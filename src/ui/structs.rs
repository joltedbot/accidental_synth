#![allow(dead_code)]

use crate::defaults::Defaults;
use crate::modules::envelope::{
    DEFAULT_ENVELOPE_MILLISECONDS, DEFAULT_ENVELOPE_SUSTAIN_LEVEL, MAX_ATTACK_MILLISECONDS,
    MAX_DECAY_MILLISECONDS, MAX_RELEASE_MILLISECONDS, MIN_ATTACK_MILLISECONDS,
    MIN_DECAY_MILLISECONDS, MIN_RELEASE_MILLISECONDS,
};
use crate::modules::lfo::DEFAULT_LFO_FREQUENCY;
use crate::modules::oscillator::constants::{
    DEFAULT_HARD_SYNC_ENABLED, DEFAULT_KEY_SYNC_ENABLED, DEFAULT_POLARITY_FLIPPED,
};
use crate::ui::constants::DEFAULT_FINE_TUNE_NORMAL_VALUE;

#[derive(Clone)]
pub struct UIAudioDevice {
    pub output_device_index: i32,
    pub left_channel_index: i32,
    pub right_channel_index: i32,
    pub sample_rate_index: i32,
    pub buffer_size_index: i32,
    pub output_devices: Vec<String>,
    pub left_channels: Vec<String>,
    pub right_channels: Vec<String>,
    pub sample_rates: Vec<String>,
    pub buffer_sizes: Vec<String>,
}

impl Default for UIAudioDevice {
    fn default() -> Self {
        let sample_rates = Defaults::SUPPORTED_SAMPLE_RATES
            .iter()
            .map(ToString::to_string)
            .collect();
        let buffer_sizes = Defaults::SUPPORTED_BUFFER_SIZES
            .iter()
            .map(ToString::to_string)
            .collect();
        Self {
            output_device_index: Defaults::AUDIO_DEVICE_INDEX,
            left_channel_index: Defaults::LEFT_CHANNEL_INDEX,
            right_channel_index: Defaults::LEFT_CHANNEL_INDEX,
            sample_rate_index: Defaults::SAMPLE_RATE_INDEX as i32,
            buffer_size_index: Defaults::BUFFER_SIZE_INDEX as i32,
            output_devices: Vec::new(),
            left_channels: Vec::new(),
            right_channels: Vec::new(),
            sample_rates,
            buffer_sizes,
        }
    }
}

#[derive(Clone, Default)]
pub struct UIMidiPort {
    pub input_ports: Vec<String>,
    pub input_port_index: i32,
    pub channels: Vec<String>,
    pub channel_index: i32,
}

#[derive(Clone, Copy)]
pub struct UIOscillator {
    pub wave_shape_index: i32,
    pub fine_tune: f32,
    pub course_tune: i32,
    pub clipper_boost: f32,
    pub parameter1: f32,
    pub parameter2: f32,
}

impl Default for UIOscillator {
    fn default() -> Self {
        Self {
            wave_shape_index: 0,
            fine_tune: DEFAULT_FINE_TUNE_NORMAL_VALUE,
            course_tune: 0,
            clipper_boost: 0.0,
            parameter1: 0.0,
            parameter2: 0.0,
        }
    }
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
    pub envelope_amount: f32,
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

#[derive(Debug, Clone, Copy)]
pub struct UIGlobalOptions {
    pub portamento_time: f32,
    pub portamento_is_enabled: bool,
    pub pitch_bend_range: i32,
    pub velocity_curve_slope: f32,
    pub hard_sync_is_enabled: bool,
    pub key_sync_is_enabled: bool,
    pub polarity_is_flipped: bool,
}

impl Default for UIGlobalOptions {
    fn default() -> Self {
        Self {
            portamento_time: Defaults::PORTAMENTO_TIME_NORMAL_VALUE,
            portamento_is_enabled: false,
            pitch_bend_range: i32::from(Defaults::PITCH_BEND_RANGE),
            velocity_curve_slope: Defaults::VELOCITY_CURVE_NORMAL_VALUE,
            hard_sync_is_enabled: DEFAULT_HARD_SYNC_ENABLED,
            key_sync_is_enabled: DEFAULT_KEY_SYNC_ENABLED,
            polarity_is_flipped: DEFAULT_POLARITY_FLIPPED,
        }
    }
}
