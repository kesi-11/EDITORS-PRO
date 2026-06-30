//! Batch export queue — manages multiple export jobs in sequence.
//!
//! ## video: debt markers
//!
//! - Sequential queue (one job at a time), upgrade to parallel jobs if the hardware supports concurrent encodes
//! - No persistence, upgrade to drift database backing if queue should survive app restart
//! - No retry logic, upgrade to auto-retry with backoff if a job fails transiently

use serde::{Deserialize, Serialize};
use std::collections::VecDeque;
use std::sync::{Arc, Mutex};
use std::time::{SystemTime, UNIX_EPOCH};

/// Status of an export job.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum JobStatus {
    Queued,
    Running,
    Completed,
    Failed,
    Cancelled,
}

/// A single export job in the batch queue.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExportJob {
    pub id: String,
    pub name: String,
    /// Project file path to export.
    pub project_path: String,
    /// Output file path.
    pub output_path: String,
    /// Export preset name (from `get_export_presets`).
    pub preset: String,
    pub status: JobStatus,
    /// 0.0–1.0 progress when running.
    pub progress: f32,
    /// Error message if failed.
    pub error: Option<String>,
    /// Submitted at (Unix epoch ms).
    pub submitted_at_ms: u64,
    /// Started at (Unix epoch ms), if running/completed.
    pub started_at_ms: Option<u64>,
    /// Completed at (Unix epoch ms), if completed/failed.
    pub completed_at_ms: Option<u64>,
}

/// The batch export queue. Wrapped in `Arc<Mutex>` for shared access.
#[derive(Debug, Default)]
pub struct BatchExportQueue {
    jobs: VecDeque<ExportJob>,
    /// Set to true to stop the queue after the current job.
    stop_requested: bool,
}

impl BatchExportQueue {
    pub fn new() -> Self {
        Self::default()
    }

    /// Add a job to the end of the queue.
    pub fn enqueue(&mut self, mut job: ExportJob) -> String {
        job.id = uuid::Uuid::new_v4().to_string();
        job.status = JobStatus::Queued;
        job.progress = 0.0;
        job.submitted_at_ms = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis() as u64;
        let id = job.id.clone();
        self.jobs.push_back(job);
        id
    }

    /// Get the next queued job, marking it as running.
    pub fn dequeue_next(&mut self) -> Option<ExportJob> {
        if self.stop_requested {
            return None;
        }
        for job in self.jobs.iter_mut() {
            if job.status == JobStatus::Queued {
                job.status = JobStatus::Running;
                job.started_at_ms = Some(SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .unwrap_or_default()
                    .as_millis() as u64);
                return Some(job.clone());
            }
        }
        None
    }

    /// Update a running job's progress.
    pub fn update_progress(&mut self, job_id: &str, progress: f32) {
        for job in self.jobs.iter_mut() {
            if job.id == job_id && job.status == JobStatus::Running {
                job.progress = progress.clamp(0.0, 1.0);
                break;
            }
        }
    }

    /// Mark a running job as completed.
    pub fn complete(&mut self, job_id: &str) {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis() as u64;
        for job in self.jobs.iter_mut() {
            if job.id == job_id {
                job.status = JobStatus::Completed;
                job.progress = 1.0;
                job.completed_at_ms = Some(now);
                break;
            }
        }
    }

    /// Mark a running job as failed.
    pub fn fail(&mut self, job_id: &str, error: String) {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis() as u64;
        for job in self.jobs.iter_mut() {
            if job.id == job_id {
                job.status = JobStatus::Failed;
                job.error = Some(error);
                job.completed_at_ms = Some(now);
                break;
            }
        }
    }

    /// Cancel a queued or running job.
    pub fn cancel(&mut self, job_id: &str) {
        for job in self.jobs.iter_mut() {
            if job.id == job_id
                && (job.status == JobStatus::Queued || job.status == JobStatus::Running)
            {
                job.status = JobStatus::Cancelled;
                job.completed_at_ms = Some(SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .unwrap_or_default()
                    .as_millis() as u64);
                break;
            }
        }
    }

    /// Request the queue to stop after the current job.
    pub fn request_stop(&mut self) {
        self.stop_requested = true;
    }

    /// Clear the stop request (resume queue).
    pub fn resume(&mut self) {
        self.stop_requested = false;
    }

    /// Get a snapshot of all jobs (in queue order).
    pub fn jobs(&self) -> Vec<ExportJob> {
        self.jobs.iter().cloned().collect()
    }

    /// Get a snapshot of one job.
    pub fn job(&self, job_id: &str) -> Option<ExportJob> {
        self.jobs.iter().find(|j| j.id == job_id).cloned()
    }

    /// Count jobs by status.
    pub fn count_by_status(&self, status: JobStatus) -> usize {
        self.jobs.iter().filter(|j| j.status == status).count()
    }

    /// Remove completed/failed/cancelled jobs from the queue.
    pub fn clear_finished(&mut self) {
        self.jobs.retain(|j| {
            j.status != JobStatus::Completed
                && j.status != JobStatus::Failed
                && j.status != JobStatus::Cancelled
        });
    }
}

/// Shared, thread-safe batch export queue (the typical usage pattern).
pub type SharedBatchExportQueue = Arc<Mutex<BatchExportQueue>>;

pub fn shared_queue() -> SharedBatchExportQueue {
    Arc::new(Mutex::new(BatchExportQueue::new()))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_job(name: &str) -> ExportJob {
        ExportJob {
            id: String::new(),
            name: name.into(),
            project_path: "/tmp/proj.epp".into(),
            output_path: format!("/tmp/{}.mp4", name),
            preset: "1080p_h264".into(),
            status: JobStatus::Queued,
            progress: 0.0,
            error: None,
            submitted_at_ms: 0,
            started_at_ms: None,
            completed_at_ms: None,
        }
    }

    #[test]
    fn enqueue_assigns_id_and_status() {
        let mut q = BatchExportQueue::new();
        let id = q.enqueue(sample_job("job1"));
        assert!(!id.is_empty());
        assert_eq!(q.job(&id).unwrap().status, JobStatus::Queued);
    }

    #[test]
    fn dequeue_returns_oldest_queued() {
        let mut q = BatchExportQueue::new();
        let id1 = q.enqueue(sample_job("job1"));
        let id2 = q.enqueue(sample_job("job2"));
        let next = q.dequeue_next().unwrap();
        assert_eq!(next.id, id1);
        assert_eq!(next.status, JobStatus::Running);
        // Second dequeue gets job2
        let next = q.dequeue_next().unwrap();
        assert_eq!(next.id, id2);
    }

    #[test]
    fn dequeue_returns_none_when_empty() {
        let mut q = BatchExportQueue::new();
        assert!(q.dequeue_next().is_none());
    }

    #[test]
    fn complete_marks_job_done() {
        let mut q = BatchExportQueue::new();
        let id = q.enqueue(sample_job("job1"));
        q.dequeue_next();
        q.complete(&id);
        let job = q.job(&id).unwrap();
        assert_eq!(job.status, JobStatus::Completed);
        assert_eq!(job.progress, 1.0);
        assert!(job.completed_at_ms.is_some());
    }

    #[test]
    fn fail_records_error() {
        let mut q = BatchExportQueue::new();
        let id = q.enqueue(sample_job("job1"));
        q.dequeue_next();
        q.fail(&id, "encoder failed".into());
        let job = q.job(&id).unwrap();
        assert_eq!(job.status, JobStatus::Failed);
        assert_eq!(job.error.as_deref(), Some("encoder failed"));
    }

    #[test]
    fn cancel_queued_job() {
        let mut q = BatchExportQueue::new();
        let id = q.enqueue(sample_job("job1"));
        q.cancel(&id);
        assert_eq!(q.job(&id).unwrap().status, JobStatus::Cancelled);
    }

    #[test]
    fn stop_request_blocks_dequeue() {
        let mut q = BatchExportQueue::new();
        q.enqueue(sample_job("job1"));
        q.request_stop();
        assert!(q.dequeue_next().is_none());
        q.resume();
        assert!(q.dequeue_next().is_some());
    }

    #[test]
    fn clear_finished_removes_terminal_jobs() {
        let mut q = BatchExportQueue::new();
        let id1 = q.enqueue(sample_job("job1"));
        let id2 = q.enqueue(sample_job("job2"));
        q.complete(&id1);
        assert_eq!(q.jobs().len(), 2);
        q.clear_finished();
        assert_eq!(q.jobs().len(), 1);
        assert_eq!(q.jobs()[0].id, id2);
    }

    #[test]
    fn count_by_status() {
        let mut q = BatchExportQueue::new();
        let id1 = q.enqueue(sample_job("job1"));
        let _id2 = q.enqueue(sample_job("job2"));
        let _id3 = q.enqueue(sample_job("job3"));
        q.complete(&id1);
        assert_eq!(q.count_by_status(JobStatus::Completed), 1);
        assert_eq!(q.count_by_status(JobStatus::Queued), 2);
    }
}
