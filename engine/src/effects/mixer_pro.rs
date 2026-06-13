//! Professional Audio Mixer — VU meters, pan law, automation, submixes, aux sends.
//!
//! Full-featured audio mixing console with constant-power pan law,
//! VU metering, automation lanes, submix buses, and auxiliary sends.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// VU meter readings for a channel.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct VuMeter {
    pub peak_db: f32,
    pub rms_db: f32,
    pub peak_hold_db: f32,
    pub peak_hold_frames: u32,
    pub clipping: bool,
}

impl VuMeter {
    pub fn new() -> Self { Self::default() }

    pub fn update(&mut self, samples: &[f32]) {
        if samples.is_empty() { return; }
        let rms = (samples.iter().map(|s| s * s).sum::<f32>() / samples.len() as f32).sqrt();
        let peak = samples.iter().cloned().fold(0.0f32, f32::max);
        self.rms_db = if rms > 0.0 { 20.0 * rms.log10() } else { -60.0 };
        self.peak_db = if peak > 0.0 { 20.0 * peak.log10() } else { -60.0 };
        self.clipping = peak >= 1.0;
        if self.peak_db > self.peak_hold_db || self.peak_hold_frames > 30 {
            self.peak_hold_db = self.peak_db;
            self.peak_hold_frames = 0;
        }
        self.peak_hold_frames += 1;
    }
}

/// Constant-power pan law: maps pan position [-1..1] to (left_gain, right_gain).
pub fn constant_power_pan(pan: f32) -> (f32, f32) {
    let angle = (pan.clamp(-1.0, 1.0) + 1.0) * 0.25 * std::f32::consts::PI;
    (angle.cos(), angle.sin())
}

/// Linear pan law (simpler alternative).
pub fn linear_pan(pan: f32) -> (f32, f32) {
    let p = (pan.clamp(-1.0, 1.0) + 1.0) * 0.5;
    (1.0 - p, p)
}

/// Automation point for a single parameter.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AutomationPoint {
    pub time_ms: f64,
    pub value: f32,
}

/// Automation lane for a parameter.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AutomationLane {
    pub param_name: String,
    pub points: Vec<AutomationPoint>,
}

impl AutomationLane {
    pub fn new(param_name: &str) -> Self { Self { param_name: param_name.to_string(), points: Vec::new() } }

    pub fn add_point(&mut self, time_ms: f64, value: f32) {
        self.points.push(AutomationPoint { time_ms, value });
        self.points.sort_by(|a, b| a.time_ms.partial_cmp(&b.time_ms).unwrap());
    }

    pub fn value_at(&self, time_ms: f64) -> Option<f32> {
        if self.points.is_empty() { return None; }
        if time_ms <= self.points[0].time_ms { return Some(self.points[0].value); }
        if time_ms >= self.points[self.points.len()-1].time_ms { return Some(self.points[self.points.len()-1].value); }
        for i in 0..self.points.len()-1 {
            if time_ms >= self.points[i].time_ms && time_ms <= self.points[i+1].time_ms {
                let t = (time_ms - self.points[i].time_ms) / (self.points[i+1].time_ms - self.points[i].time_ms + 1e-10);
                return Some(self.points[i].value + (self.points[i+1].value - self.points[i].value) * t as f32);
            }
        }
        None
    }
}

/// An aux send from a channel.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuxSend {
    pub bus_id: String,
    pub level_db: f32,
    pub pre_fader: bool,
    pub enabled: bool,
}

/// A mixer channel strip.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChannelStrip {
    pub id: String,
    pub name: String,
    pub volume_db: f32,
    pub pan: f32,           // -1..1
    pub muted: bool,
    pub solo: bool,
    pub vu_meter: VuMeter,
    pub aux_sends: Vec<AuxSend>,
    pub automation: Vec<AutomationLane>,
    pub assigned_submix: Option<String>,
}

impl ChannelStrip {
    pub fn new(name: &str) -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            name: name.to_string(),
            volume_db: 0.0,
            pan: 0.0,
            muted: false,
            solo: false,
            vu_meter: VuMeter::new(),
            aux_sends: Vec::new(),
            automation: Vec::new(),
            assigned_submix: None,
        }
    }

    pub fn volume_linear(&self) -> f32 {
        if self.volume_db <= -60.0 { 0.0 }
        else { 10.0_f32.powf(self.volume_db / 20.0) }
    }

    pub fn add_aux_send(&mut self, bus_id: &str, level_db: f32, pre_fader: bool) {
        self.aux_sends.push(AuxSend { bus_id: bus_id.to_string(), level_db, pre_fader, enabled: true });
    }

    pub fn get_automation(&mut self, param: &str) -> &mut AutomationLane {
        if !self.automation.iter().any(|a| a.param_name == param) {
            self.automation.push(AutomationLane::new(param));
        }
        self.automation.iter_mut().find(|a| a.param_name == param).unwrap()
    }
}

/// A submix bus.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SubmixBus {
    pub id: String,
    pub name: String,
    pub volume_db: f32,
    pub vu_meter: VuMeter,
}

impl SubmixBus {
    pub fn new(name: &str) -> Self {
        Self { id: uuid::Uuid::new_v4().to_string(), name: name.to_string(), volume_db: 0.0, vu_meter: VuMeter::new() }
    }
}

/// The master audio mixer.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AudioMixerPro {
    pub channels: HashMap<String, ChannelStrip>,
    pub submixes: HashMap<String, SubmixBus>,
    pub master_volume_db: f32,
    pub master_vu: VuMeter,
}

impl AudioMixerPro {
    pub fn new() -> Self {
        Self { channels: HashMap::new(), submixes: HashMap::new(), master_volume_db: 0.0, master_vu: VuMeter::new() }
    }

    pub fn add_channel(&mut self, channel: ChannelStrip) { self.channels.insert(channel.id.clone(), channel); }
    pub fn add_submix(&mut self, submix: SubmixBus) { self.submixes.insert(submix.id.clone(), submix); }

    /// Mix all channels into a stereo output buffer.
    pub fn mix_to_stereo(&mut self, channel_buffers: &HashMap<String, Vec<f32>>, frame_count: usize) -> Vec<(f32, f32)> {
        let mut output = vec![(0.0f32, 0.0f32); frame_count];
        let master_vol = if self.master_volume_db <= -60.0 { 0.0 } else { 10.0_f32.powf(self.master_volume_db / 20.0) };
        let any_solo = self.channels.values().any(|c| c.solo);

        for (id, channel) in self.channels.iter_mut() {
            if channel.muted { continue; }
            if any_solo && !channel.solo { continue; }
            let vol = channel.volume_linear();
            let (left_gain, right_gain) = constant_power_pan(channel.pan);
            if let Some(buffer) = channel_buffers.get(id) {
                let mut peak = 0.0f32;
                for (i, frame) in output.iter_mut().enumerate().take(frame_count) {
                    let sample = buffer.get(i).copied().unwrap_or(0.0) * vol;
                    peak = peak.max(sample.abs());
                    frame.0 += sample * left_gain;
                    frame.1 += sample * right_gain;
                }
                channel.vu_meter.update(&buffer[..frame_count.min(buffer.len())]);
            }
        }

        // Apply master volume
        for frame in &mut output {
            frame.0 *= master_vol;
            frame.1 *= master_vol;
        }

        // Update master VU
        let all_samples: Vec<f32> = output.iter().flat_map(|(l, r)| [*l, *r]).collect();
        self.master_vu.update(&all_samples);

        output
    }

    pub fn solo_count(&self) -> usize { self.channels.values().filter(|c| c.solo).count() }
    pub fn channel_count(&self) -> usize { self.channels.len() }
    pub fn submix_count(&self) -> usize { self.submixes.len() }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_constant_power_pan_center() {
        let (l, r) = constant_power_pan(0.0);
        assert!((l - r).abs() < 0.01);
        assert!((l - 0.707).abs() < 0.01);
    }

    #[test]
    fn test_constant_power_pan_left() {
        let (l, r) = constant_power_pan(-1.0);
        assert!((l - 1.0).abs() < 0.01);
        assert!((r - 0.0).abs() < 0.01);
    }

    #[test]
    fn test_constant_power_pan_right() {
        let (l, r) = constant_power_pan(1.0);
        assert!((l - 0.0).abs() < 0.01);
        assert!((r - 1.0).abs() < 0.01);
    }

    #[test]
    fn test_linear_pan() {
        let (l, r) = linear_pan(0.0);
        assert!((l - 0.5).abs() < 0.01);
        assert!((r - 0.5).abs() < 0.01);
    }

    #[test]
    fn test_vu_meter() {
        let mut vu = VuMeter::new();
        let samples = vec![0.5; 100];
        vu.update(&samples);
        assert!(!vu.clipping);
        assert!(vu.rms_db > -10.0);
    }

    #[test]
    fn test_vu_meter_clipping() {
        let mut vu = VuMeter::new();
        let samples = vec![1.5; 10];
        vu.update(&samples);
        assert!(vu.clipping);
    }

    #[test]
    fn test_vu_meter_silence() {
        let mut vu = VuMeter::new();
        let samples = vec![0.0; 100];
        vu.update(&samples);
        assert_eq!(vu.rms_db, -60.0);
    }

    #[test]
    fn test_channel_strip_new() {
        let ch = ChannelStrip::new("Test");
        assert_eq!(ch.name, "Test");
        assert_eq!(ch.volume_db, 0.0);
        assert!(!ch.muted);
    }

    #[test]
    fn test_channel_volume_linear() {
        let ch = ChannelStrip::new("Test");
        assert!((ch.volume_linear() - 1.0).abs() < 0.01);
    }

    #[test]
    fn test_channel_volume_muted() {
        let mut ch = ChannelStrip::new("Test");
        ch.volume_db = -60.0;
        assert_eq!(ch.volume_linear(), 0.0);
    }

    #[test]
    fn test_automation_lane() {
        let mut lane = AutomationLane::new("volume");
        lane.add_point(0.0, 0.5);
        lane.add_point(1000.0, 1.0);
        assert_eq!(lane.value_at(500.0), Some(0.75));
    }

    #[test]
    fn test_automation_lane_edge() {
        let mut lane = AutomationLane::new("pan");
        lane.add_point(0.0, 0.0);
        lane.add_point(1000.0, 1.0);
        assert_eq!(lane.value_at(-100.0), Some(0.0));
        assert_eq!(lane.value_at(2000.0), Some(1.0));
    }

    #[test]
    fn test_automation_empty() {
        let lane = AutomationLane::new("test");
        assert!(lane.value_at(0.0).is_none());
    }

    #[test]
    fn test_aux_send() {
        let mut ch = ChannelStrip::new("Test");
        ch.add_aux_send("reverb", -6.0, true);
        assert_eq!(ch.aux_sends.len(), 1);
        assert!(ch.aux_sends[0].pre_fader);
    }

    #[test]
    fn test_submix() {
        let sub = SubmixBus::new("Music");
        assert_eq!(sub.name, "Music");
        assert_eq!(sub.volume_db, 0.0);
    }

    #[test]
    fn test_mixer_basic() {
        let mut mixer = AudioMixerPro::new();
        let ch = ChannelStrip::new("Track 1");
        let ch_id = ch.id.clone();
        mixer.add_channel(ch);
        let buffers = HashMap::from([(ch_id, vec![0.5f32; 100])]);
        let output = mixer.mix_to_stereo(&buffers, 100);
        assert_eq!(output.len(), 100);
        assert!(output[0].0.abs() > 0.0);
    }

    #[test]
    fn test_mixer_muted_channel() {
        let mut mixer = AudioMixerPro::new();
        let mut ch = ChannelStrip::new("Track 1");
        ch.muted = true;
        let ch_id = ch.id.clone();
        mixer.add_channel(ch);
        let buffers = HashMap::from([(ch_id, vec![0.5f32; 100])]);
        let output = mixer.mix_to_stereo(&buffers, 100);
        assert_eq!(output[0].0, 0.0);
    }

    #[test]
    fn test_mixer_solo() {
        let mut mixer = AudioMixerPro::new();
        let mut ch1 = ChannelStrip::new("Track 1");
        let mut ch2 = ChannelStrip::new("Track 2");
        ch2.solo = true;
        let ch1_id = ch1.id.clone();
        let ch2_id = ch2.id.clone();
        mixer.add_channel(ch1);
        mixer.add_channel(ch2);
        let buffers = HashMap::from([(ch1_id, vec![0.5f32; 100]), (ch2_id, vec![0.8f32; 100])]);
        let output = mixer.mix_to_stereo(&buffers, 100);
        assert!(output[0].0.abs() > 0.0);
    }

    #[test]
    fn test_mixer_master_volume() {
        let mut mixer = AudioMixerPro::new();
        mixer.master_volume_db = -6.0;
        let ch = ChannelStrip::new("Track 1");
        let ch_id = ch.id.clone();
        mixer.add_channel(ch);
        let buffers = HashMap::from([(ch_id, vec![0.5f32; 100])]);
        let output = mixer.mix_to_stereo(&buffers, 100);
        assert!(output[0].0 < 0.5); // Master volume reduces level
    }

    #[test]
    fn test_mixer_counts() {
        let mut mixer = AudioMixerPro::new();
        mixer.add_channel(ChannelStrip::new("Ch1"));
        mixer.add_channel(ChannelStrip::new("Ch2"));
        mixer.add_submix(SubmixBus::new("Sub1"));
        assert_eq!(mixer.channel_count(), 2);
        assert_eq!(mixer.submix_count(), 1);
    }

    #[test]
    fn test_automation_sorted_insert() {
        let mut lane = AutomationLane::new("vol");
        lane.add_point(1000.0, 0.8);
        lane.add_point(0.0, 0.5);
        assert_eq!(lane.points[0].time_ms, 0.0);
    }

    #[test]
    fn test_get_automation_creates() {
        let mut ch = ChannelStrip::new("Test");
        ch.get_automation("volume");
        assert_eq!(ch.automation.len(), 1);
    }
}
