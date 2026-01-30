use crate::math::load_f32_from_atomic_u32;
use crate::modules::effects::bitshifter::BitShifter;
use crate::modules::effects::clipper::Clipper;
use crate::modules::effects::constants::{MAX_GATE_CUT, MAX_THRESHOLD, PARAMETERS_PER_EFFECT};
use crate::modules::effects::gate::Gate;
use crate::modules::effects::rectifier::Rectifier;
use crate::modules::effects::wavefolder::WaveFolder;
use std::sync::atomic::Ordering::Relaxed;
use std::sync::atomic::{AtomicBool, AtomicU32};
use strum::{EnumCount, EnumIter, FromRepr, IntoEnumIterator};

mod autopan;
mod bitshifter;
mod clipper;
mod compressor;
mod constants;
mod gate;
mod rectifier;
mod saturation;
mod tremolo;
mod wavefolder;

pub trait AudioEffect {
    fn process_samples(&mut self, samples: (f32, f32), effect: &EffectParameters) -> (f32, f32);
}

#[derive(Debug, Clone, Copy, EnumCount, EnumIter, FromRepr)]
#[repr(i32)]
pub enum EffectIndex {
    WaveFolder,
    Clipper,
    Gate,
    Rectifier,
    BitShifter,
    Saturation,
    Compressor,
    AutoPan,
    Tremolo,
}

impl EffectIndex {
    pub fn from_i32(index: i32) -> Option<Self> {
        Self::from_repr(index)
    }
}

#[derive(Debug)]
pub struct AudioEffectParameters {
    pub is_enabled: AtomicBool,
    pub parameters: [AtomicU32; PARAMETERS_PER_EFFECT],
}

impl Default for AudioEffectParameters {
    fn default() -> Self {
        Self {
            is_enabled: AtomicBool::new(false),
            parameters: [
                AtomicU32::new(0.0_f32.to_bits()),
                AtomicU32::new(0.0_f32.to_bits()),
                AtomicU32::new(0.0_f32.to_bits()),
                AtomicU32::new(0.0_f32.to_bits()),
            ],
        }
    }
}

#[derive(Debug)]
pub struct EffectParameters {
    is_enabled: bool,
    parameters: Vec<f32>,
}

impl Default for EffectParameters {
    fn default() -> Self {
        let parameters = vec![0.0; PARAMETERS_PER_EFFECT];
        Self {
            is_enabled: false,
            parameters,
        }
    }
}

pub struct Effects {
    effects: Vec<Box<dyn AudioEffect>>,
    parameters: [EffectParameters; EffectIndex::COUNT],
}

impl Effects {
    pub fn new(sample_rate: u32) -> Self {
        let wavefolder = Box::new(WaveFolder::new());
        let wavefolder_parameters = EffectParameters::default();

        let clipper = Box::new(Clipper::new());
        let mut clipper_parameters = EffectParameters::default();
        clipper_parameters.parameters[0] = MAX_THRESHOLD;

        let gate = Box::new(Gate::new());
        let mut gate_parameters = EffectParameters::default();
        gate_parameters.parameters[1] = MAX_GATE_CUT;

        let rectifier = Box::new(Rectifier::new());
        let rectifier_parameters = EffectParameters::default();

        let bitshifter = Box::new(BitShifter::new());
        let bitshifter_parameters = EffectParameters::default();

        let saturation = Box::new(saturation::Saturation::new());
        let saturation_parameters = EffectParameters::default();

        let compressor = Box::new(compressor::Compressor::new());
        let mut compressor_parameters = EffectParameters::default();
        compressor_parameters.parameters[0] = MAX_THRESHOLD;

        let autopan = Box::new(autopan::AutoPan::new(sample_rate));
        let mut autopan_parameters = EffectParameters::default();
        autopan_parameters.parameters[0] = MAX_THRESHOLD;

        let tremolo = Box::new(tremolo::Tremolo::new(sample_rate));
        let mut tremolo_parameters = EffectParameters::default();
        tremolo_parameters.parameters[0] = MAX_THRESHOLD;

        Self {
            effects: vec![
                wavefolder, clipper, gate, rectifier, bitshifter, saturation, compressor, autopan,
                tremolo,
            ],
            parameters: [
                wavefolder_parameters,
                clipper_parameters,
                gate_parameters,
                rectifier_parameters,
                bitshifter_parameters,
                saturation_parameters,
                compressor_parameters,
                autopan_parameters,
                tremolo_parameters,
            ],
        }
    }

    pub fn set_parameters(&mut self, parameters: &[AudioEffectParameters]) {
        parameters
            .iter()
            .enumerate()
            .for_each(|(index, effect_parameters)| {
                self.parameters[index] = extract_parameters(effect_parameters);
            });
    }

    pub fn process(&mut self, mut samples: (f32, f32)) -> (f32, f32) {
        for (index, parameter) in self.parameters.iter().enumerate() {
            if parameter.is_enabled {
                samples = self.effects[index].process_samples(samples, parameter);
            }
        }

        samples
    }
}

fn extract_parameters(source: &AudioEffectParameters) -> EffectParameters {
    EffectParameters {
        is_enabled: source.is_enabled.load(Relaxed),
        parameters: source
            .parameters
            .iter()
            .map(load_f32_from_atomic_u32)
            .collect(),
    }
}

pub fn default_audio_effect_parameters() -> Vec<AudioEffectParameters> {
    let mut audio_effect_parameters = Vec::new();

    for effect in EffectIndex::iter() {
        match effect {
            EffectIndex::WaveFolder
            | EffectIndex::Rectifier
            | EffectIndex::BitShifter
            | EffectIndex::Saturation => {
                audio_effect_parameters.push(AudioEffectParameters::default());
            }
            EffectIndex::Clipper | EffectIndex::Compressor => {
                let mut effects_parameters = AudioEffectParameters::default();
                effects_parameters.parameters[0] = AtomicU32::new(MAX_THRESHOLD.to_bits());
                audio_effect_parameters.push(effects_parameters);
            }
            EffectIndex::Gate | EffectIndex::AutoPan | EffectIndex::Tremolo => {
                let mut effects_parameters = AudioEffectParameters::default();
                effects_parameters.parameters[1] = AtomicU32::new(MAX_GATE_CUT.to_bits());
                audio_effect_parameters.push(effects_parameters);
            }
        }
    }

    audio_effect_parameters
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::math::f32s_are_equal;

    #[test]
    fn effect_index_from_i32_returns_wavefolder_for_0() {
        let result = EffectIndex::from_i32(0);

        assert!(matches!(result, Some(EffectIndex::WaveFolder)));
    }

    #[test]
    fn effect_index_from_i32_returns_none_for_negative() {
        let result = EffectIndex::from_i32(-1);

        assert!(result.is_none());
    }

    #[test]
    fn effect_index_from_i32_returns_none_for_out_of_range() {
        let result = EffectIndex::from_i32(14);

        assert!(result.is_none());
    }

    #[test]
    fn extract_parameters_copies_is_enabled() {
        let source = AudioEffectParameters {
            is_enabled: AtomicBool::new(true),
            parameters: [
                AtomicU32::new(0.5_f32.to_bits()),
                AtomicU32::new(0.0_f32.to_bits()),
                AtomicU32::new(0.0_f32.to_bits()),
                AtomicU32::new(0.0_f32.to_bits()),
            ],
        };

        let result = extract_parameters(&source);

        assert!(result.is_enabled);
    }

    #[test]
    fn extract_parameters_copies_parameters() {
        let source = AudioEffectParameters {
            is_enabled: AtomicBool::new(false),
            parameters: [
                AtomicU32::new(0.5_f32.to_bits()),
                AtomicU32::new(0.3_f32.to_bits()),
                AtomicU32::new(0.7_f32.to_bits()),
                AtomicU32::new(0.1_f32.to_bits()),
            ],
        };

        let result = extract_parameters(&source);

        assert!(f32s_are_equal(result.parameters[0], 0.5));
        assert!(f32s_are_equal(result.parameters[1], 0.3));
        assert!(f32s_are_equal(result.parameters[2], 0.7));
        assert!(f32s_are_equal(result.parameters[3], 0.1));
    }

    #[test]
    fn effects_process_returns_original_when_all_disabled() {
        let mut effects = Effects::new(48000);
        let params = vec![
            AudioEffectParameters::default(),
            AudioEffectParameters::default(),
            AudioEffectParameters::default(),
            AudioEffectParameters::default(),
        ];
        effects.set_parameters(&params);
        let input = (0.7, -0.5);

        let result = effects.process(input);

        assert!(f32s_are_equal(result.0, 0.7));
        assert!(f32s_are_equal(result.1, -0.5));
    }

    #[test]
    fn effects_process_applies_enabled_effect() {
        let mut effects = Effects::new(48000);
        let mut params = vec![
            AudioEffectParameters::default(),
            AudioEffectParameters::default(),
            AudioEffectParameters::default(),
            AudioEffectParameters::default(),
        ];
        // Enable rectifier with half-wave mode
        params[EffectIndex::Rectifier as usize].is_enabled = AtomicBool::new(true);
        params[EffectIndex::Rectifier as usize].parameters[0] = AtomicU32::new(0.0_f32.to_bits()); // half-wave
        effects.set_parameters(&params);
        let input = (0.5, -0.3);

        let result = effects.process(input);

        // Half-wave rectifier: positive passes, negative becomes 0
        assert!(f32s_are_equal(result.0, 0.5));
        assert!(f32s_are_equal(result.1, 0.0));
    }

    #[test]
    fn effects_process_chains_multiple_effects() {
        let mut effects = Effects::new(48000);
        let mut params = vec![
            AudioEffectParameters::default(),
            AudioEffectParameters::default(),
            AudioEffectParameters::default(),
            AudioEffectParameters::default(),
            AudioEffectParameters::default(),
        ];
        // Enable rectifier with full-wave mode
        params[EffectIndex::Rectifier as usize].is_enabled = AtomicBool::new(true);
        params[EffectIndex::Rectifier as usize].parameters[0] = AtomicU32::new(1.0_f32.to_bits()); // full-wave
        // Enable clipper with low threshold and no gains
        params[EffectIndex::Clipper as usize].is_enabled = AtomicBool::new(true);
        params[EffectIndex::Clipper as usize].parameters[0] = AtomicU32::new(0.4_f32.to_bits()); // threshold
        params[EffectIndex::Clipper as usize].parameters[1] = AtomicU32::new(0.0_f32.to_bits()); // pre_gain
        params[EffectIndex::Clipper as usize].parameters[2] = AtomicU32::new(0.0_f32.to_bits()); // post_gain
        params[EffectIndex::Clipper as usize].parameters[3] = AtomicU32::new(0.0_f32.to_bits()); // notch
        effects.set_parameters(&params);
        let input = (0.6, -0.6);

        let result = effects.process(input);

        // First wavefolder (disabled), then clipper (enabled), then rectifier (enabled), then bitshifter (disabled)
        // Clipper: 0.6 > 0.4 -> clipped to 0.4, -0.6 -> clipped to -0.4
        // Rectifier full-wave: 0.4 stays 0.4, -0.4 becomes 0.4
        assert!(f32s_are_equal(result.0, 0.4));
        assert!(f32s_are_equal(result.1, 0.4));
    }

    #[test]
    fn default_audio_effect_parameters_returns_correct_count() {
        let params = default_audio_effect_parameters();

        assert_eq!(params.len(), EffectIndex::COUNT);
    }

    #[test]
    fn default_audio_effect_parameters_clipper_has_max_threshold() {
        let params = default_audio_effect_parameters();

        // Clipper
        let clipper_threshold =
            load_f32_from_atomic_u32(&params[EffectIndex::Clipper as usize].parameters[0]);
        assert!(f32s_are_equal(clipper_threshold, MAX_THRESHOLD));
    }

    #[test]
    fn default_audio_effect_parameters_all_effects_disabled() {
        let params = default_audio_effect_parameters();

        for param in &params {
            assert!(!param.is_enabled.load(Relaxed));
        }
    }
}
