use crate::modules::effects::bitcrusher::BitCrusher;
use crate::modules::effects::clipper::Clipper;
use crate::modules::effects::gate::Gate;
use crate::modules::effects::rectifier::Rectifier;
use crate::modules::effects::wavefolder::WaveFolder;
pub use accsyn_core::effects::EffectIndex;
use accsyn_core::effects::{AudioEffect, EffectParameters, PARAMETERS_PER_EFFECT};
use accsyn_core::parameter_types::NormalizedValue;
use serde::{Deserialize, Serialize};
use std::f32::consts::FRAC_PI_2;
use std::sync::atomic::AtomicBool;
use std::sync::atomic::Ordering::Relaxed;

mod autopan;
mod bitcrusher;
mod chorus;
mod clipper;
mod compressor;
mod constants;
mod delay;
mod flanger;
mod gate;
mod rectifier;
mod saturation;
mod tremolo;
mod wavefolder;

/// Shared atomic parameters for controlling a single audio effect from the UI thread.
#[derive(Debug, Serialize, Deserialize)]
pub struct AudioEffectParameters {
    /// Effect name
    pub name: String,
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
            name: String::new(),
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
    pub(crate) fn new(sample_rate: u32) -> Self {
        let saturation = Box::new(saturation::Saturation::new());
        let compressor = Box::new(compressor::Compressor::new());
        let wavefolder = Box::new(WaveFolder::new());
        let clipper = Box::new(Clipper::new());
        let gate = Box::new(Gate::new());
        let rectifier = Box::new(Rectifier::new());
        let bitcrusher = Box::new(BitCrusher::new());
        let delay = Box::new(delay::Delay::new());
        let autopan = Box::new(autopan::AutoPan::new(sample_rate));
        let tremolo = Box::new(tremolo::Tremolo::new(sample_rate));
        let chorus = Box::new(chorus::Chorus::new(sample_rate));
        let flanger = Box::new(flanger::Flanger::new(sample_rate));

        Self {
            effects: vec![
                saturation, compressor, wavefolder, bitcrusher, clipper, gate, rectifier, chorus,
                flanger, autopan, tremolo, delay,
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
        name: source.name.clone(),
        is_enabled: source.is_enabled.load(Relaxed),
        parameters: source
            .parameters
            .iter()
            .map(NormalizedValue::load)
            .collect(),
    }
}

fn wet_dry_blend(
    dry_samples: (f32, f32),
    wet_samples: (f32, f32),
    blend_amount: f32,
) -> (f32, f32) {
    let angle = blend_amount * FRAC_PI_2; // 0 → π/2
    let dry_gain = angle.cos();
    let wet_gain = angle.sin();

    let left_sample = dry_samples.0 * dry_gain + wet_samples.0 * wet_gain;
    let right_sample = dry_samples.1 * dry_gain + wet_samples.1 * wet_gain;

    (left_sample, right_sample)
}

fn interpolate_samples(
    buffer_samples_a: (f32, f32),
    buffer_samples_b: (f32, f32),
    fractional_index: f32,
) -> (f32, f32) {
    let mut interpolated_left =
        buffer_samples_a.0 * (1.0 - fractional_index) + buffer_samples_b.0 * fractional_index;
    let mut interpolated_right =
        buffer_samples_a.1 * (1.0 - fractional_index) + buffer_samples_b.1 * fractional_index;

    if interpolated_left.is_nan() || interpolated_right.is_nan() {
        interpolated_left = buffer_samples_a.0;
        interpolated_right = buffer_samples_a.1;
    }

    if interpolated_left.is_infinite() || interpolated_right.is_infinite() {
        interpolated_left = buffer_samples_a.0;
        interpolated_right = buffer_samples_a.1;
    }
    (interpolated_left, interpolated_right)
}

fn interpolate_single_sample(
    buffer_samples_a: f32,
    buffer_samples_b: f32,
    fractional_index: f32,
) -> f32 {
    let interpolated_sample =
        buffer_samples_a * (1.0 - fractional_index) + buffer_samples_b * fractional_index;

    if interpolated_sample.is_nan() {
        return buffer_samples_a;
    }

    if interpolated_sample.is_infinite() {
        return buffer_samples_a;
    }

    interpolated_sample
}

#[cfg(test)]
mod tests {
    use super::*;
    use accsyn_core::math::f32s_are_equal;

    #[test]
    fn effect_index_from_i32_returns_saturation_for_0() {
        let result = EffectIndex::from_i32(0);

        assert!(matches!(result, Some(EffectIndex::Saturation)));
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
            name: String::new(),
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
    fn extract_parameters_copies_name() {
        let test_name = "Test Effect";
        let source = AudioEffectParameters {
            name: String::from(test_name),
            is_enabled: AtomicBool::new(false),
            parameters: [
                NormalizedValue::new(0.5_f32),
                NormalizedValue::new(0.3_f32),
                NormalizedValue::new(0.7_f32),
                NormalizedValue::new(0.1_f32),
            ],
        };

        let result = extract_parameters(&source);

        assert_eq!(result.name, test_name);
    }

    #[test]
    fn extract_parameters_copies_parameters() {
        let source = AudioEffectParameters {
            name: String::new(),
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
            AudioEffectParameters::default(),
            AudioEffectParameters::default(),
            AudioEffectParameters::default(),
            AudioEffectParameters::default(),
            AudioEffectParameters::default(),
            AudioEffectParameters::default(),
        ];
        // Enable rectifier with half-wave mode
        params[EffectIndex::Rectifier as usize].is_enabled = AtomicBool::new(true);
        params[EffectIndex::Rectifier as usize].parameters[0] = NormalizedValue::new(0.0_f32); // half-wave
        params[EffectIndex::Rectifier as usize].parameters[1] = NormalizedValue::new(1.0_f32); // 100% blend
        effects.set_parameters(&params);
        let input = (0.5, -0.3);

        let result = effects.process(input);

        // Half-wave rectifier: positive passes, negative becomes 0
        assert!(
            f32s_are_equal(result.0, 0.5),
            "Expected 0.5, Recieved {}",
            result.0
        );
        assert!(
            f32s_are_equal(result.1, 0.0),
            "Expected 0.0, Recieved {}",
            result.1
        );
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
            AudioEffectParameters::default(),
            AudioEffectParameters::default(),
        ];
        // Enable rectifier with full-wave mode
        params[EffectIndex::Rectifier as usize].is_enabled = AtomicBool::new(true);
        params[EffectIndex::Rectifier as usize].parameters[0] = NormalizedValue::new(1.0_f32); // full-wave
        params[EffectIndex::Rectifier as usize].parameters[1] = NormalizedValue::new(1.0_f32); // 100% blend

        // Enable clipper with low threshold and no gains
        params[EffectIndex::Clipper as usize].is_enabled = AtomicBool::new(true);
        params[EffectIndex::Clipper as usize].parameters[0] = NormalizedValue::new(0.4_f32); // threshold
        params[EffectIndex::Clipper as usize].parameters[1] = NormalizedValue::new(0.0_f32); // pre_gain
        params[EffectIndex::Clipper as usize].parameters[2] = NormalizedValue::new(0.0_f32); // post_gain
        params[EffectIndex::Clipper as usize].parameters[3] = NormalizedValue::new(0.0_f32); // notch
        effects.set_parameters(&params);
        let input = (0.6, -0.6);

        let result = effects.process(input);

        // First wavefolder (disabled), then clipper (enabled), then rectifier (enabled), then bitcrusher (disabled)
        // Clipper: 0.6 > 0.4 -> clipped to 0.4, -0.6 -> clipped to -0.4
        // Rectifier full-wave: 0.4 stays 0.4, -0.4 becomes 0.4
        assert!(f32s_are_equal(result.0, 0.4));
        assert!(f32s_are_equal(result.1, 0.4));
    }

    #[test]
    fn dry_wet_blend_returns_correct_blend_for_arbitrary_values() {
        let samples = (0.1234, -0.9876);
        let rectified_samples = (1.0, -1.0);
        let blend_amount = 0.75;
        let expected_values = (0.971103, -1.301818);

        let result = wet_dry_blend(samples, rectified_samples, blend_amount);

        assert!(
            f32s_are_equal(result.0, expected_values.0),
            "Expected {}, Recieved {}",
            expected_values.0,
            result.0
        );
        assert!(
            f32s_are_equal(result.1, expected_values.1),
            "Expected {}, Recieved {}",
            expected_values.1,
            result.1
        );
    }

    #[test]
    fn dry_wet_blend_returns_correct_blend_for_max_min_values() {
        let samples = (1.0, -1.0);
        let rectified_samples = (-1.0, 1.0);
        let blend_amount = 1.0;
        let expected_values = (-1.0, 1.0);

        let result = wet_dry_blend(samples, rectified_samples, blend_amount);

        assert!(
            f32s_are_equal(result.0, expected_values.0),
            "Expected {}, Recieved {}",
            expected_values.0,
            result.0
        );
        assert!(
            f32s_are_equal(result.1, expected_values.1),
            "Expected {}, Recieved {}",
            expected_values.1,
            result.1
        );
    }

    #[test]
    fn dry_wet_blend_returns_correctly_handles_zeros() {
        let samples = (1.0, -1.0);
        let rectified_samples = (0.0, 0.0);
        let blend_amount = 0.0;
        let expected_values = (1.0, -1.0);

        let result = wet_dry_blend(samples, rectified_samples, blend_amount);

        assert!(
            f32s_are_equal(result.0, expected_values.0),
            "Expected {}, Recieved {}",
            expected_values.0,
            result.0
        );
        assert!(
            f32s_are_equal(result.1, expected_values.1),
            "Expected {}, Recieved {}",
            expected_values.1,
            result.1
        );
    }

    #[test]
    fn interpolate_samples_blends_at_fractional_index() {
        let buffer_samples_a = (0.0, 1.0);
        let buffer_samples_b = (1.0, 0.0);

        let result_quarter = interpolate_samples(buffer_samples_a, buffer_samples_b, 0.25);
        let result_half = interpolate_samples(buffer_samples_a, buffer_samples_b, 0.5);
        let result_three_quarter = interpolate_samples(buffer_samples_a, buffer_samples_b, 0.75);

        let expected_quarter = (0.25, 0.75);
        assert!(
            f32s_are_equal(result_quarter.0, expected_quarter.0),
            "Expected {}, got {:?}",
            expected_quarter.0,
            result_quarter.0
        );
        assert!(
            f32s_are_equal(result_quarter.1, expected_quarter.1),
            "Expected {}, got {:?}",
            expected_quarter.1,
            result_quarter.1
        );
        let expected_half = (0.5, 0.5);
        assert!(
            f32s_are_equal(result_half.0, expected_half.0),
            "Expected {}, got {:?}",
            expected_half.0,
            result_half.0
        );
        assert!(
            f32s_are_equal(result_half.1, expected_half.1),
            "Expected {}, got {:?}",
            expected_half.1,
            result_half.1
        );
        let expected_three_quarter = (0.75, 0.25);
        assert!(
            f32s_are_equal(result_three_quarter.0, expected_three_quarter.0),
            "Expected {}, got {:?}",
            expected_three_quarter.0,
            result_three_quarter.0
        );
        assert!(
            f32s_are_equal(result_three_quarter.1, expected_three_quarter.1),
            "Expected {}, got {:?}",
            expected_three_quarter.1,
            result_three_quarter.1
        );
    }

    #[test]
    fn interpolate_samples_returns_first_sample_at_zero_fractional_index() {
        let buffer_samples_a = (0.3, -0.6);
        let buffer_samples_b = (0.9, 0.2);

        let result = interpolate_samples(buffer_samples_a, buffer_samples_b, 0.0);

        assert!(f32s_are_equal(result.0, buffer_samples_a.0));
        assert!(f32s_are_equal(result.1, buffer_samples_a.1));
    }

    #[test]
    fn interpolate_samples_returns_second_sample_at_one_fractional_index() {
        let buffer_samples_a = (0.3, -0.6);
        let buffer_samples_b = (0.9, 0.2);

        let result = interpolate_samples(buffer_samples_a, buffer_samples_b, 1.0);

        assert!(f32s_are_equal(result.0, buffer_samples_b.0));
        assert!(f32s_are_equal(result.1, buffer_samples_b.1));
    }

    #[test]
    fn interpolate_samples_falls_back_to_first_sample_when_result_would_be_nan() {
        let buffer_samples_a = (0.3, -0.6);
        let buffer_samples_b = (f32::NAN, f32::NAN);

        let result = interpolate_samples(buffer_samples_a, buffer_samples_b, 0.5);

        assert!(f32s_are_equal(result.0, buffer_samples_a.0));
        assert!(f32s_are_equal(result.1, buffer_samples_a.1));
    }

    #[test]
    fn interpolate_samples_falls_back_to_first_sample_when_result_would_be_infinite() {
        let buffer_samples_a = (0.3, -0.6);
        let buffer_samples_b = (f32::INFINITY, f32::NEG_INFINITY);

        let result = interpolate_samples(buffer_samples_a, buffer_samples_b, 0.5);

        assert!(f32s_are_equal(result.0, buffer_samples_a.0));
        assert!(f32s_are_equal(result.1, buffer_samples_a.1));
    }
}
