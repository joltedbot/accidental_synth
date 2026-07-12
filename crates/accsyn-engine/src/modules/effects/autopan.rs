use crate::modules::effects::constants::{
    AUTO_PAN_MAX_WIDTH, DEFAULT_LFO_WAVESHAPE_INDEX, EFFECTS_LFO_CENTER_VALUE,
};
use crate::modules::lfo::Lfo;
use crate::synthesizer::midi_value_converters::{
    exponential_curve_lfo_frequency_from_normal_value, normal_value_to_wave_shape_index,
};
use accsyn_core::casting::f32_to_u8_clamped;
use accsyn_core::defaults::Defaults;
use accsyn_core::effects::{AudioEffect, EffectParameters};
use accsyn_core::math::f32s_are_equal;
use std::f32::consts::PI;

#[derive(Debug, Default)]
struct LfoParameters {
    frequency: f32,
    width: f32,
    waveshape_index: f32,
}

pub struct AutoPan {
    lfo: Lfo,
    lfo_parameters: LfoParameters,
}

impl AutoPan {
    pub fn new(sample_rate: u32) -> Self {
        log::debug!(target: "synthesizer::effects::autopan", "Constructing AutoPan Effect Module");

        let mut lfo = Lfo::new(sample_rate);
        lfo.set_center_value(EFFECTS_LFO_CENTER_VALUE);

        let lfo_parameters = LfoParameters {
            frequency: Defaults::LFO_FREQUENCY,
            width: AUTO_PAN_MAX_WIDTH,
            waveshape_index: DEFAULT_LFO_WAVESHAPE_INDEX,
        };

        Self {
            lfo,
            lfo_parameters,
        }
    }
}

impl AudioEffect for AutoPan {
    fn process_samples(&mut self, samples: (f32, f32), effect: &EffectParameters) -> (f32, f32) {
        if !effect.is_enabled {
            return samples;
        }

        let new_frequency = effect.parameters[0];
        let new_width = effect.parameters[1];
        let new_shape = f32::from(normal_value_to_wave_shape_index(effect.parameters[2]));

        if !f32s_are_equal(new_frequency, self.lfo_parameters.frequency) {
            self.lfo_parameters.frequency = new_frequency;
            self.lfo
                .set_frequency(exponential_curve_lfo_frequency_from_normal_value(
                    new_frequency,
                ));
        }
        if !f32s_are_equal(new_width, self.lfo_parameters.width) {
            self.lfo_parameters.width = new_width;
            self.lfo.set_range(new_width);
        }

        if !f32s_are_equal(new_shape, self.lfo_parameters.waveshape_index) {
            self.lfo_parameters.waveshape_index = new_shape;
            self.lfo.set_wave_shape(f32_to_u8_clamped(new_shape));
        }

        let lfo_value = self.lfo.generate(None);
        autopan_samples(samples, lfo_value)
    }
}

fn autopan_samples(samples: (f32, f32), pan: f32) -> (f32, f32) {
    let left_out = (pan * PI / 2.0).cos() * samples.0;
    let right_out = (pan * PI / 2.0).sin() * samples.1;
    (left_out, right_out)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::modules::oscillator::WaveShape;
    use accsyn_core::math::f32s_are_equal;
    use strum::EnumCount;

    #[test]
    fn autopan_process_samples_returns_original_when_disabled() {
        let mut autopan = AutoPan::new(44100);
        let effect = EffectParameters {
            name: String::from("Auto Pan"),
            is_enabled: false,
            parameters: vec![0.0, 0.0, 0.0, 0.0],
        };
        let input = (0.5, -0.3);

        let result = autopan.process_samples(input, &effect);

        assert!(f32s_are_equal(result.0, 0.5));
        assert!(f32s_are_equal(result.1, -0.3));
    }

    #[test]
    fn autopan_process_samples_updates_frequency_when_changed() {
        let mut autopan = AutoPan::new(44100);
        let effect = EffectParameters {
            name: String::from("Auto Pan"),
            is_enabled: true,
            parameters: vec![0.5, 0.5, 0.0, 0.0],
        };

        // Process once to set initial frequency
        autopan.process_samples((0.5, 0.5), &effect);
        let initial_frequency = autopan.lfo_parameters.frequency;

        // Change frequency parameter
        let new_effect = EffectParameters {
            name: String::from("Auto Pan"),
            is_enabled: true,
            parameters: vec![0.8, 0.5, 0.0, 0.0],
        };
        autopan.process_samples((0.5, 0.5), &new_effect);

        assert!(!f32s_are_equal(
            initial_frequency,
            autopan.lfo_parameters.frequency
        ));
        assert!(f32s_are_equal(autopan.lfo_parameters.frequency, 0.8));
    }

    #[test]
    fn autopan_process_samples_updates_width_when_changed() {
        let mut autopan = AutoPan::new(44100);
        let effect = EffectParameters {
            name: String::from("Auto Pan"),
            is_enabled: true,
            parameters: vec![0.5, 0.3, 0.0, 0.0],
        };

        // Process once to set initial width
        autopan.process_samples((0.5, 0.5), &effect);
        let initial_width = autopan.lfo_parameters.width;

        // Change width parameter
        let new_effect = EffectParameters {
            name: String::from("Auto Pan"),
            is_enabled: true,
            parameters: vec![0.5, 0.7, 0.0, 0.0],
        };
        autopan.process_samples((0.5, 0.5), &new_effect);

        assert!(!f32s_are_equal(initial_width, autopan.lfo_parameters.width));
        assert!(f32s_are_equal(autopan.lfo_parameters.width, 0.7));
    }

    #[test]
    fn autopan_process_samples_updates_shape_when_changed() {
        let mut autopan = AutoPan::new(44100);

        let test_shape_normalized: f32 = 1.0;
        // Hand coded above with test values. Not dynamically created
        #[allow(clippy::cast_precision_loss)]
        let test_shape_index = (WaveShape::COUNT - 1) as f32;

        let effect = EffectParameters {
            name: String::from("Auto Pan"),
            is_enabled: true,
            parameters: vec![0.5, 0.5, 0.0, 0.0],
        };

        // Process once to set initial shape
        autopan.process_samples((0.5, 0.5), &effect);
        let initial_shape = autopan.lfo_parameters.waveshape_index;

        // Change shape parameter
        let new_effect = EffectParameters {
            name: String::from("Auto Pan"),
            is_enabled: true,
            parameters: vec![0.5, 0.5, test_shape_normalized, 0.0],
        };
        autopan.process_samples((0.5, 0.5), &new_effect);

        assert!(!f32s_are_equal(
            initial_shape,
            autopan.lfo_parameters.waveshape_index
        ));

        assert!(
            f32s_are_equal(autopan.lfo_parameters.waveshape_index, test_shape_index),
            "{} and {} are not equal",
            autopan.lfo_parameters.waveshape_index,
            test_shape_index
        );
    }

    #[test]
    fn autopan_process_samples_does_not_update_when_parameters_unchanged() {
        let mut autopan = AutoPan::new(44100);
        let effect = EffectParameters {
            name: String::from("Auto Pan"),
            is_enabled: true,
            parameters: vec![0.5, 0.6, 2.0, 0.0],
        };

        // Process once to set parameters
        autopan.process_samples((0.5, 0.5), &effect);
        let initial_frequency = autopan.lfo_parameters.frequency;
        let initial_width = autopan.lfo_parameters.width;
        let initial_shape = autopan.lfo_parameters.waveshape_index;

        // Process again with same parameters
        autopan.process_samples((0.5, 0.5), &effect);

        // Parameters should remain the same
        assert!(f32s_are_equal(
            initial_frequency,
            autopan.lfo_parameters.frequency
        ));
        assert!(f32s_are_equal(initial_width, autopan.lfo_parameters.width));
        assert!(f32s_are_equal(
            initial_shape,
            autopan.lfo_parameters.waveshape_index
        ));
    }

    #[test]
    fn autopan_samples_at_center_pan() {
        let samples = (1.0, 1.0);
        let pan = 0.0; // Center pan
        let expected_left = (0.0 * PI / 2.0).cos(); // cos(0) = 1.0
        let expected_right = (0.0 * PI / 2.0).sin(); // sin(0) = 0.0

        let result = autopan_samples(samples, pan);

        assert!(
            f32s_are_equal(result.0, expected_left),
            "Expected: {expected_left}, got: {}",
            result.0
        );
        assert!(
            f32s_are_equal(result.1, expected_right),
            "Expected: {expected_right}, got: {}",
            result.1
        );
    }

    #[test]
    fn autopan_samples_at_full_right_pan() {
        let samples = (1.0, 1.0);
        let pan = 1.0; // Full right
        let expected_left = (1.0 * PI / 2.0).cos(); // cos(π/2) ≈ 0.0
        let expected_right = (1.0 * PI / 2.0).sin(); // sin(π/2) = 1.0

        let result = autopan_samples(samples, pan);

        assert!(
            f32s_are_equal(result.0, expected_left),
            "Expected: {expected_left}, got: {}",
            result.0
        );
        assert!(
            f32s_are_equal(result.1, expected_right),
            "Expected: {expected_right}, got: {}",
            result.1
        );
    }

    #[test]
    fn autopan_samples_at_full_left_pan() {
        let samples = (1.0, 1.0);
        let pan = -1.0; // Full left
        let expected_left = (-PI / 2.0).cos(); // cos(-π/2) ≈ 0.0
        let expected_right = (-PI / 2.0).sin(); // sin(-π/2) = -1.0

        let result = autopan_samples(samples, pan);

        assert!(
            f32s_are_equal(result.0, expected_left),
            "Expected: {expected_left}, got: {}",
            result.0
        );
        assert!(
            f32s_are_equal(result.1, expected_right),
            "Expected: {expected_right}, got: {}",
            result.1
        );
    }
}
