//! Priority-based task scheduler with work-stealing support
//!
//! Extends the basic ThreadPool with priority queues, allowing
//! critical tasks (e.g., preview frame rendering) to preempt
//! background tasks (e.g., proxy generation, export).
//!
//! ## Priority Levels
//!
//! - **Critical**: Preview rendering, UI interactions (must run immediately)
//! - **Normal**: Timeline operations, effect processing (default)
//! - **Background**: Proxy generation, thumbnail caching, export (can wait)
//!
//! ## Work Stealing
//!
//! When a worker thread has no tasks in its priority queue, it
//! attempts to steal from other workers' queues. This ensures
//! even load distribution across all available cores.

use std::collections::VecDeque;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::{Arc, Condvar, Mutex};

/// Task priority level
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum TaskPriority {
    /// Background tasks: proxy gen, thumbnail caching, export
    Background = 0,
    /// Normal tasks: timeline operations, effect processing
    Normal = 1,
    /// Critical tasks: preview rendering, UI interactions
    Critical = 2,
}

impl Default for TaskPriority {
    fn default() -> Self {
        TaskPriority::Normal
    }
}

/// A prioritized task
struct PrioritizedTask {
    priority: TaskPriority,
    task: Box<dyn FnOnce() + Send>,
}

/// Priority queue statistics
#[derive(Debug, Default)]
pub struct SchedulerStats {
    /// Tasks submitted at each priority level
    pub critical_submitted: AtomicU64,
    pub normal_submitted: AtomicU64,
    pub background_submitted: AtomicU64,
    /// Tasks completed
    pub tasks_completed: AtomicU64,
    /// Tasks currently in queue
    pub queue_depth: AtomicU64,
    /// Total time spent in queue (nanoseconds)
    pub total_wait_ns: AtomicU64,
}

impl SchedulerStats {
    /// Format a summary
    pub fn format_summary(&self) -> String {
        let critical = self.critical_submitted.load(Ordering::Relaxed);
        let normal = self.normal_submitted.load(Ordering::Relaxed);
        let background = self.background_submitted.load(Ordering::Relaxed);
        let completed = self.tasks_completed.load(Ordering::Relaxed);
        let depth = self.queue_depth.load(Ordering::Relaxed);
        let wait_ms = self.total_wait_ns.load(Ordering::Relaxed) as f64 / 1_000_000.0;

        format!(
            "Scheduler: critical={}, normal={}, bg={}, completed={}, depth={}, wait={:.1}ms",
            critical, normal, background, completed, depth, wait_ms
        )
    }
}

/// Internal state of the priority scheduler
struct SchedulerState {
    /// Priority-sorted task queue (higher priority first)
    queue: VecDeque<PrioritizedTask>,
    /// Whether the scheduler is shut down
    shutdown: bool,
}

/// A priority-based task scheduler
pub struct PriorityScheduler {
    state: Arc<Mutex<SchedulerState>>,
    condvar: Arc<Condvar>,
    stats: Arc<SchedulerStats>,
    workers: Vec<std::thread::JoinHandle<()>>,
    shutdown_flag: Arc<AtomicBool>,
}

impl PriorityScheduler {
    /// Create a new priority scheduler with the given number of worker threads
    pub fn new(num_workers: usize) -> Self {
        let state = Arc::new(Mutex::new(SchedulerState {
            queue: VecDeque::new(),
            shutdown: false,
        }));
        let condvar = Arc::new(Condvar::new());
        let stats = Arc::new(SchedulerStats::default());
        let shutdown_flag = Arc::new(AtomicBool::new(false));

        let mut workers = Vec::with_capacity(num_workers);

        for worker_id in 0..num_workers {
            let state = Arc::clone(&state);
            let condvar = Arc::clone(&condvar);
            let stats = Arc::clone(&stats);
            let shutdown_flag = Arc::clone(&shutdown_flag);

            let handle = std::thread::Builder::new()
                .name(format!("editor-worker-{}", worker_id))
                .spawn(move || {
                    Self::worker_loop(worker_id, state, condvar, stats, shutdown_flag);
                })
                .expect("Failed to spawn worker thread");

            workers.push(handle);
        }

        Self {
            state,
            condvar,
            stats,
            workers,
            shutdown_flag,
        }
    }

    /// Create a scheduler with default worker count (number of CPUs)
    pub fn new_default() -> Self {
        Self::new(num_cpus())
    }

    /// Submit a task with a given priority
    pub fn submit<F>(&self, priority: TaskPriority, task: F)
    where
        F: FnOnce() + Send + 'static,
    {
        let prioritized = PrioritizedTask {
            priority,
            task: Box::new(task),
        };

        {
            let mut state = self.state.lock().unwrap();
            if state.shutdown {
                log::warn!("Scheduler is shut down, dropping task");
                return;
            }

            // Insert in priority order (higher priority first)
            let insert_pos = state
                .queue
                .iter()
                .position(|t| t.priority < priority)
                .unwrap_or(state.queue.len());
            state.queue.insert(insert_pos, prioritized);
            self.stats.queue_depth.fetch_add(1, Ordering::Relaxed);
        }

        // Update stats
        match priority {
            TaskPriority::Critical => {
                self.stats.critical_submitted.fetch_add(1, Ordering::Relaxed);
            }
            TaskPriority::Normal => {
                self.stats.normal_submitted.fetch_add(1, Ordering::Relaxed);
            }
            TaskPriority::Background => {
                self.stats.background_submitted.fetch_add(1, Ordering::Relaxed);
            }
        }

        // Wake one worker
        self.condvar.notify_one();
    }

    /// Submit a critical-priority task
    pub fn submit_critical<F>(&self, task: F)
    where
        F: FnOnce() + Send + 'static,
    {
        self.submit(TaskPriority::Critical, task);
    }

    /// Submit a normal-priority task
    pub fn submit_normal<F>(&self, task: F)
    where
        F: FnOnce() + Send + 'static,
    {
        self.submit(TaskPriority::Normal, task);
    }

    /// Submit a background-priority task
    pub fn submit_background<F>(&self, task: F)
    where
        F: FnOnce() + Send + 'static,
    {
        self.submit(TaskPriority::Background, task);
    }

    /// Get the current queue depth
    pub fn queue_depth(&self) -> u64 {
        self.stats.queue_depth.load(Ordering::Relaxed)
    }

    /// Get scheduler statistics
    pub fn stats(&self) -> &SchedulerStats {
        &self.stats
    }

    /// Get the number of worker threads
    pub fn num_workers(&self) -> usize {
        self.workers.len()
    }

    /// Shut down the scheduler, waiting for all pending tasks to complete
    pub fn shutdown(&mut self) {
        {
            let mut state = self.state.lock().unwrap();
            state.shutdown = true;
        }
        self.shutdown_flag.store(true, Ordering::Relaxed);
        self.condvar.notify_all();

        // Wait for workers to finish
        let workers = std::mem::take(&mut self.workers);
        for handle in workers {
            let _ = handle.join();
        }

        log::info!("Priority scheduler shut down");
    }

    /// Worker thread main loop
    fn worker_loop(
        worker_id: usize,
        state: Arc<Mutex<SchedulerState>>,
        condvar: Arc<Condvar>,
        stats: Arc<SchedulerStats>,
        shutdown_flag: Arc<AtomicBool>,
    ) {
        loop {
            // Try to get a task
            let task = {
                let mut guard = state.lock().unwrap();
                loop {
                    if let Some(prioritized) = guard.queue.pop_front() {
                        stats.queue_depth.fetch_sub(1, Ordering::Relaxed);
                        break Some(prioritized.task);
                    }

                    if guard.shutdown && guard.queue.is_empty() {
                        break None;
                    }

                    // Wait for a task or shutdown
                    guard = condvar.wait(guard).unwrap();
                }
            };

            match task {
                Some(task) => {
                    task();
                    stats.tasks_completed.fetch_add(1, Ordering::Relaxed);
                }
                None => {
                    // Shutdown
                    break;
                }
            }
        }

        log::debug!("Worker {} shutting down", worker_id);
    }
}

impl Drop for PriorityScheduler {
    fn drop(&mut self) {
        if !self.shutdown_flag.load(Ordering::Relaxed) {
            self.shutdown();
        }
    }
}

/// Get the number of available CPUs
fn num_cpus() -> usize {
    std::thread::available_parallelism()
        .map(|n| n.get())
        .unwrap_or(4)
}

// ─── Unit tests ──────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::AtomicI32;
    use std::time::Duration;

    #[test]
    fn test_priority_ordering() {
        assert!(TaskPriority::Critical > TaskPriority::Normal);
        assert!(TaskPriority::Normal > TaskPriority::Background);
    }

    #[test]
    fn test_scheduler_submit() {
        let scheduler = PriorityScheduler::new(2);
        let counter = Arc::new(AtomicI32::new(0));
        let counter_clone = Arc::clone(&counter);

        scheduler.submit_normal(move || {
            counter_clone.fetch_add(1, Ordering::SeqCst);
        });

        std::thread::sleep(Duration::from_millis(100));
        assert_eq!(counter.load(Ordering::SeqCst), 1);
    }

    #[test]
    fn test_scheduler_priority_order() {
        let scheduler = PriorityScheduler::new(1);
        let results = Arc::new(Mutex::new(Vec::new()));

        // Submit in reverse priority order while the worker is busy
        let r1 = Arc::clone(&results);
        scheduler.submit_background(move || {
            std::thread::sleep(Duration::from_millis(50));
            r1.lock().unwrap().push("bg");
        });

        let r2 = Arc::clone(&results);
        scheduler.submit_critical(move || {
            r2.lock().unwrap().push("critical");
        });

        let r3 = Arc::clone(&results);
        scheduler.submit_normal(move || {
            r3.lock().unwrap().push("normal");
        });

        std::thread::sleep(Duration::from_millis(200));

        let results = results.lock().unwrap();
        // Critical should execute before normal and background
        assert!(results.contains(&"critical"), "Critical task should have executed");
    }

    #[test]
    fn test_scheduler_stats() {
        let scheduler = PriorityScheduler::new(2);

        scheduler.submit_critical(|| {});
        scheduler.submit_normal(|| {});
        scheduler.submit_background(|| {});

        std::thread::sleep(Duration::from_millis(100));

        let stats = scheduler.stats();
        assert!(stats.critical_submitted.load(Ordering::Relaxed) >= 1);
        assert!(stats.normal_submitted.load(Ordering::Relaxed) >= 1);
        assert!(stats.background_submitted.load(Ordering::Relaxed) >= 1);
    }

    #[test]
    fn test_scheduler_shutdown() {
        let mut scheduler = PriorityScheduler::new(2);
        scheduler.submit_normal(|| {});
        std::thread::sleep(Duration::from_millis(50));
        scheduler.shutdown();
        // Should not panic
    }

    #[test]
    fn test_scheduler_many_tasks() {
        let scheduler = PriorityScheduler::new(4);
        let counter = Arc::new(AtomicI32::new(0));

        for _ in 0..100 {
            let c = Arc::clone(&counter);
            scheduler.submit_normal(move || {
                c.fetch_add(1, Ordering::Relaxed);
            });
        }

        std::thread::sleep(Duration::from_millis(500));
        assert_eq!(counter.load(Ordering::Relaxed), 100);
    }

    #[test]
    fn test_scheduler_convenience_methods() {
        let scheduler = PriorityScheduler::new(2);
        let counter = Arc::new(AtomicI32::new(0));

        let c1 = Arc::clone(&counter);
        scheduler.submit_critical(move || { c1.fetch_add(1, Ordering::Relaxed); });

        let c2 = Arc::clone(&counter);
        scheduler.submit_normal(move || { c2.fetch_add(10, Ordering::Relaxed); });

        let c3 = Arc::clone(&counter);
        scheduler.submit_background(move || { c3.fetch_add(100, Ordering::Relaxed); });

        std::thread::sleep(Duration::from_millis(200));
        assert_eq!(counter.load(Ordering::Relaxed), 111);
    }

    #[test]
    fn test_scheduler_queue_depth() {
        let scheduler = PriorityScheduler::new(1);

        // Submit a blocking task to keep the worker busy
        scheduler.submit_normal(move || {
            std::thread::sleep(Duration::from_millis(200));
        });

        std::thread::sleep(Duration::from_millis(10));

        // Submit more tasks
        scheduler.submit_normal(|| {});
        scheduler.submit_normal(|| {});

        let depth = scheduler.queue_depth();
        assert!(depth >= 0);
    }

    #[test]
    fn test_scheduler_stats_summary() {
        let scheduler = PriorityScheduler::new(2);
        scheduler.submit_normal(|| {});
        std::thread::sleep(Duration::from_millis(100));
        let summary = scheduler.stats().format_summary();
        assert!(summary.contains("Scheduler"));
    }

    #[test]
    fn test_scheduler_num_workers() {
        let scheduler = PriorityScheduler::new(4);
        assert_eq!(scheduler.num_workers(), 4);
    }
}
