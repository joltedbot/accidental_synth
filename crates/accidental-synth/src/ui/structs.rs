#![allow(dead_code)]

use crate::ui::constants::{DEFAULT_FINE_TUNE_NORMAL_VALUE, MIDI_CHANNEL_LIST};
use accsyn_engine::modules::envelope::{
    DEFAULT_ENVELOPE_MILLISECONDS, DEFAULT_ENVELOPE_SUSTAIN_LEVEL, EnvelopeParameters,
    MAX_ATTACK_MILLISECONDS, MAX_DECAY_MILLISECONDS, MAX_RELEASE_MILLISECONDS,
    MIN_ATTACK_MILLISECONDS, MIN_DECAY_MILLISECONDS, MIN_RELEASE_MILLISECONDS,
};
use accsyn_engine::modules::filter::FilterParameters;
use accsyn_engine::modules::lfo::{
    DEFAULT_LFO_FREQUENCY, LfoParameters, MAX_LFO_FREQUENCY, MIN_LFO_FREQUENCY,
};
use accsyn_engine::modules::oscillator::OscillatorParameters;
use accsyn_engine::modules::oscillator::constants::{
    DEFAULT_HARD_SYNC_ENABLED, DEFAULT_KEY_SYNC_ENABLED, DEFAULT_POLARITY_FLIPPED, MAX_CLIP_BOOST,
    MIN_CLIP_BOOST,
};
use accsyn_engine::synthesizer::{KeyboardParameters, MixerParameters};
use accsyn_types::defaults::{
    Defaults, MAX_FILTER_CUTOFF, MAX_FILTER_RESONANCE, MIN_FILTER_CUTOFF, MIN_FILTER_RESONANCE,
    OSCILLATOR_FINE_TUNE_MAX_CENTS, OSCILLATOR_FINE_TUNE_MIN_CENTS,
};
use accsyn_types::math::{
    EXPONENTIAL_PORTAMENTO_COEFFICIENT, normalize_float_range,
    normalize_unsigned_integer_range,
};
use accsyn_types::math::{
    normal_value_from_exponential_curve_and_coefficient, normalize_signed_integer_range,
};
use std::sync::atomic::Ordering::Relaxed;

#[derive(Clone, Debug)]
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

#[derive(Clone, Debug)]
pub struct UIMidiPort {
    pub input_ports: Vec<String>,
    pub input_port_index: i32,
    pub channels: Vec<String>,
    pub channel_index: i32,
}

impl Default for UIMidiPort {
    fn default() -> Self {
        Self {
            channels: MIDI_CHANNEL_LIST.iter().map(ToString::to_string).collect(),
            input_ports: Vec::new(),
            input_port_index: i32::default(),
            channel_index: i32::default(),
        }
    }
}

#[derive(Clone, Copy, Debug)]
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

impl UIOscillator {
    pub fn from_synth_parameters(parameters: &OscillatorParameters) -> Self {
        Self {
            wave_shape_index: parameters.wave_shape_index.load(Relaxed) as i32,
            fine_tune: normalize_signed_integer_range(
                parameters.fine_tune.load() as i32,
                OSCILLATOR_FINE_TUNE_MIN_CENTS as i32,
                OSCILLATOR_FINE_TUNE_MAX_CENTS as i32,
            ),
            course_tune: parameters.course_tune.load() as i32,
            clipper_boost: normalize_unsigned_integer_range(
                parameters.clipper_boost.load(Relaxed) as u32,
                MIN_CLIP_BOOST as u32,
                MAX_CLIP_BOOST as u32,
            ),
            parameter1: parameters.shape_parameter1.load(),
            parameter2: parameters.shape_parameter2.load(),
        }
    }
}

#[derive(Clone, Default, Debug)]
pub struct UIFilterCutoff {
    pub cutoff: f32,
    pub resonance: f32,
}

impl UIFilterCutoff {
    pub fn from_synth_parameters(parameters: &FilterParameters) -> Self {
        let cutoff = normalize_float_range(
            parameters.cutoff_frequency.load(),
            MIN_FILTER_CUTOFF,
            MAX_FILTER_CUTOFF,
        );
        Self {
            cutoff,
            resonance: normalize_float_range(
                parameters.resonance.load(),
                MIN_FILTER_RESONANCE,
                MAX_FILTER_RESONANCE,
            ),
        }
    }
}

#[derive(Clone, Default, Debug)]
pub struct UIFilterOptions {
    pub poles: i32,
    pub key_track: f32,
    pub envelope_amount: f32,
    pub lfo_amount: f32,
}

impl UIFilterOptions {
    pub fn from_synth_parameters(
        parameters: &FilterParameters,
        envelope: &EnvelopeParameters,
        lfo: &LfoParameters,
    ) -> Self {
        Self {
            poles: parameters.filter_poles.load(Relaxed) as i32,
            key_track: parameters.key_tracking_amount.load(),
            envelope_amount: envelope.amount.load(),
            lfo_amount: lfo.range.load(),
        }
    }
}

#[derive(Clone, Debug)]
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

impl UILfo {
    pub fn from_synth_parameters(parameters: &LfoParameters) -> Self {
        Self {
            frequency: normalize_float_range(
                parameters.frequency.load(),
                MIN_LFO_FREQUENCY,
                MAX_LFO_FREQUENCY,
            ),
            phase: parameters.phase.load(),
            wave_shape_index: parameters.wave_shape.load(Relaxed) as i32,
        }
    }
}

#[derive(Clone, Debug)]
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

impl UIEnvelope {
    pub fn from_synth_parameters(parameters: &EnvelopeParameters) -> Self {
        Self {
            attack: normalize_unsigned_integer_range(
                parameters.attack_ms.load(),
                MIN_ATTACK_MILLISECONDS,
                MAX_ATTACK_MILLISECONDS,
            ),
            decay: normalize_unsigned_integer_range(
                parameters.release_ms.load(),
                MIN_RELEASE_MILLISECONDS,
                MAX_RELEASE_MILLISECONDS,
            ),
            sustain: parameters.sustain_level.load(),
            release: normalize_unsigned_integer_range(
                parameters.release_ms.load(),
                MIN_RELEASE_MILLISECONDS,
                MAX_RELEASE_MILLISECONDS,
            ),
            inverted: parameters.is_inverted.load(Relaxed),
        }
    }
}

#[derive(Clone, Debug)]
pub struct UIMixer {
    pub level: f32,
    pub balance: f32,
    pub is_muted: bool,
}
impl UIMixer {
    fn output_default() -> Self {
        Self {
            level: Defaults::OUTPUT_MIXER_LEVEL,
            balance: Defaults::OUTPUT_MIXER_BALANCE,
            is_muted: Defaults::OUTPUT_MIXER_IS_MUTED,
        }
    }

    fn quad_default() -> Self {
        Self {
            level: Defaults::QUAD_MIXER_LEVEL,
            balance: Defaults::QUAD_MIXER_BALANCE,
            is_muted: Defaults::QUAD_MIXER_IS_MUTED,
        }
    }

    pub fn from_synth_parameters(parameters: &MixerParameters) -> Self {
        Self {
            level: parameters.level.load(),
            balance: parameters.balance.load(),
            is_muted: parameters.is_muted.load(Relaxed),
        }
    }
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

impl UIGlobalOptions {
    pub fn from_synth_parameters(
        keyboard_parameters: &KeyboardParameters,
        oscillator_parameters: &OscillatorParameters,
    ) -> Self {
        Self {
            portamento_time: normal_value_from_exponential_curve_and_coefficient(
                oscillator_parameters.portamento_time.load() as f32,
                EXPONENTIAL_PORTAMENTO_COEFFICIENT,
            ),
            portamento_is_enabled: oscillator_parameters.portamento_enabled.load(Relaxed),
            pitch_bend_range: keyboard_parameters.pitch_bend_range.load(Relaxed) as i32,
            velocity_curve_slope: keyboard_parameters.velocity_curve.load(),
            hard_sync_is_enabled: oscillator_parameters.hard_sync_enabled.load(Relaxed),
            key_sync_is_enabled: oscillator_parameters.key_sync_enabled.load(Relaxed),
            polarity_is_flipped: keyboard_parameters.polarity_flipped.load(Relaxed),
        }
    }
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
