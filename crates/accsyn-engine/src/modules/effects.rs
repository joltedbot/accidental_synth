use crate::modules::effects::bitshifter::BitShifter;
use crate::modules::effects::clipper::Clipper;
use crate::modules::effects::gate::Gate;
use crate::modules::effects::rectifier::Rectifier;
use crate::modules::effects::wavefolder::WaveFolder;
pub use accsyn_types::effects::EffectIndex;
use accsyn_types::effects::{AudioEffect, EffectParameters, PARAMETERS_PER_EFFECT};
use accsyn_types::parameter_types::NormalizedValue;
use serde::{Deserialize, Serialize};
use std::sync::atomic::AtomicBool;
use std::sync::atomic::Ordering::Relaxed;

mod autopan;
mod bitshifter;
mod clipper;
mod compressor;
mod constants;
mod delay;
mod gate;
mod rectifier;
mod saturation;
mod tremolo;
mod wavefolder;

/// Shared atomic parameters for controlling a single audio effect from the UI thread.
#[derive(Debug, Serialize, Deserialize)]
pub struct AudioEffectParameters {
    /// Whether this effect is active in the processing chain.
    pub is_enabled: AtomicBool,
    /// Effect-specific parameter values.
    pub parameters: [NormalizedValue; PARAMETERS_PER_EFFECT],
}

impl AudioEffectParameters {
    /// Replace all the values in this `AudioEffectParameters` with the values from the provided `AudioEffectParameters`.
    pub fn assign_from(&self, effects_parameters: &AudioEffectParameters) {
        self.is_enabled
            .store(effects_parameters.is_enabled.load(Relaxed), Relaxed);
        self.parameters
            .iter()
            .enumerate()
            .for_each(|(index, parameter)| {
                parameter.store(effects_parameters.parameters[index].load());
            });
    }
}

impl Default for AudioEffectParameters {
    fn default() -> Self {
        Self {
            is_enabled: AtomicBool::new(false),
            parameters: [
                NormalizedValue::default(),
                NormalizedValue::default(),
                NormalizedValue::default(),
                NormalizedValue::default(),
            ],
        }
    }
}

/// Audio effects processing chain that applies enabled effects in sequence.
pub struct Effects {
    effects: Vec<Box<dyn AudioEffect>>,
    parameters: Vec<EffectParameters>,
}

impl Effects {
    /// Creates a new effects chain with all available effects initialized.
    pub fn new(sample_rate: u32) -> Self {
        let saturation = Box::new(saturation::Saturation::new());
        let compressor = Box::new(compressor::Compressor::new());
        let wavefolder = Box::new(WaveFolder::new());
        let clipper = Box::new(Clipper::new());
        let gate = Box::new(Gate::new());
        let rectifier = Box::new(Rectifier::new());
        let bitshifter = Box::new(BitShifter::new());
        let delay = Box::new(delay::Delay::new());
        let autopan = Box::new(autopan::AutoPan::new(sample_rate));
        let tremolo = Box::new(tremolo::Tremolo::new(sample_rate));

        Self {
            effects: vec![
                saturation, compressor, wavefolder, clipper, gate, rectifier, bitshifter, delay,
                autopan, tremolo,
            ],
            parameters: EffectParameters::default_all(),
        }
    }

    /// Updates all effect parameters from the shared parameter array.
    pub fn set_parameters(&mut self, parameters: &[AudioEffectParameters]) {
        parameters
            .iter()
            .enumerate()
            .for_each(|(index, effect_parameters)| {
                self.parameters[index] = extract_parameters(effect_parameters);
            });
    }

    /// Processes a stereo sample pair through all enabled effects in order.
    pub fn process(&mut self, mut samples: (f32, f32)) -> (f32, f32) {
        for (effect, parameter) in self.effects.iter_mut().zip(self.parameters.iter()) {
            samples = effect.process_samples(samples, parameter);
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
            .map(|parameter| parameter.load())
            .collect(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use accsyn_types::math::f32s_are_equal;

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
                NormalizedValue::new(0.5_f32),
                NormalizedValue::new(0.0_f32),
                NormalizedValue::new(0.0_f32),
                NormalizedValue::new(0.0_f32),
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
                NormalizedValue::new(0.5_f32),
                NormalizedValue::new(0.3_f32),
                NormalizedValue::new(0.7_f32),
                NormalizedValue::new(0.1_f32),
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
        params[EffectIndex::Rectifier as usize].parameters[0] = NormalizedValue::new(0.0_f32); // half-wave
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
        params[EffectIndex::Rectifier as usize].parameters[0] = NormalizedValue::new(1.0_f32); // full-wave

        // Enable clipper with low threshold and no gains
        params[EffectIndex::Clipper as usize].is_enabled = AtomicBool::new(true);
        params[EffectIndex::Clipper as usize].parameters[0] = NormalizedValue::new(0.4_f32); // threshold
        params[EffectIndex::Clipper as usize].parameters[1] = NormalizedValue::new(0.0_f32); // pre_gain
        params[EffectIndex::Clipper as usize].parameters[2] = NormalizedValue::new(0.0_f32); // post_gain
        params[EffectIndex::Clipper as usize].parameters[3] = NormalizedValue::new(0.0_f32); // notch
        effects.set_parameters(&params);
        let input = (0.6, -0.6);

        let result = effects.process(input);

        // First wavefolder (disabled), then clipper (enabled), then rectifier (enabled), then bitshifter (disabled)
        // Clipper: 0.6 > 0.4 -> clipped to 0.4, -0.6 -> clipped to -0.4
        // Rectifier full-wave: 0.4 stays 0.4, -0.4 becomes 0.4
        assert!(f32s_are_equal(result.0, 0.4));
        assert!(f32s_are_equal(result.1, 0.4));
    }
}
