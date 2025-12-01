#![allow(dead_code)]

use crate::modules::envelope::{
    DEFAULT_ENVELOPE_MILLISECONDS, DEFAULT_ENVELOPE_SUSTAIN_LEVEL, MAX_ATTACK_MILLISECONDS,
    MAX_DECAY_MILLISECONDS, MAX_RELEASE_MILLISECONDS, MIN_ATTACK_MILLISECONDS,
    MIN_DECAY_MILLISECONDS, MIN_RELEASE_MILLISECONDS,
};
use crate::modules::lfo::DEFAULT_LFO_FREQUENCY;

#[derive(Clone, Default)]
pub struct UIAudioDevice {
    pub output_device_index: i32,
    pub left_channel_index: i32,
    pub right_channel_index: i32,
    pub sample_rate_index: i32,
    pub output_devices: Vec<String>,
    pub left_channels: Vec<String>,
    pub right_channels: Vec<String>,
    pub sample_rates: Vec<String>,
}

#[derive(Clone, Default)]
pub struct UIMidiPort {
    pub input_ports: Vec<String>,
    pub input_port_index: i32,
    pub channels: Vec<String>,
    pub channel_index: i32,
}

#[derive(Clone, Default)]
pub struct UIOscillator {
    pub wave_shape_index: i32,
    pub fine_tune: f32,
    pub fine_tune_cents: i32,
    pub course_tune: i32,
    pub clipper_boost: f32,
    pub parameter1: f32,
    pub parameter2: f32,
}

#[derive(Clone, Default)]
pub struct UIFilterCutoff {
    pub cutoff: f32,
    pub resonance: f32,
}

#[derive(Clone, Default)]
pub struct UIFilterOptions {
    pub poles: i32,
    pub key_track: f32,
    pub eg_amount: f32,
    pub lfo_amount: f32,
}

#[derive(Clone)]
pub struct UILfo {
    pub frequency: f32,
    pub phase: f32,
    pub wave_shape_index: i32,
}

impl Default for UILfo {
    fn default() -> Self {
        Self {
            frequency: DEFAULT_LFO_FREQUENCY,
            phase: 0.0,
            wave_shape_index: 0,
        }
    }
}

#[derive(Clone)]
pub struct UIEnvelope {
    pub attack: f32,
    pub decay: f32,
    pub sustain: f32,
    pub release: f32,
    pub inverted: bool,
}

impl Default for UIEnvelope {
    fn default() -> Self {
        Self {
            attack: DEFAULT_ENVELOPE_MILLISECONDS as f32
                / (MAX_ATTACK_MILLISECONDS - MIN_ATTACK_MILLISECONDS) as f32,
            decay: DEFAULT_ENVELOPE_MILLISECONDS as f32
                / (MAX_DECAY_MILLISECONDS - MIN_DECAY_MILLISECONDS) as f32,
            sustain: DEFAULT_ENVELOPE_SUSTAIN_LEVEL,
            release: DEFAULT_ENVELOPE_MILLISECONDS as f32
                / (MAX_RELEASE_MILLISECONDS - MIN_RELEASE_MILLISECONDS) as f32,
            inverted: false,
        }
    }
}

#[derive(Clone, Default)]
pub struct UIMixer {
    pub level: f32,
    pub balance: f32,
    pub is_muted: bool,
}
