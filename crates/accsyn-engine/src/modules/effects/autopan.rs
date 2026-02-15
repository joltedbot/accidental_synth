use crate::modules::effects::constants::{
    AUTO_PAN_MAX_WIDTH, DEFAULT_LFO_WAVESHAPE_INDEX, EFFECTS_LFO_CENTER_VALUE,
};
use crate::modules::lfo::{DEFAULT_LFO_FREQUENCY, Lfo};
use crate::synthesizer::midi_value_converters::exponential_curve_lfo_frequency_from_normal_value;
use accsyn_types::effects::{AudioEffect, EffectParameters};
use accsyn_types::math::f32s_are_equal;
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
        log::debug!("Constructing AutoPan Effect Module");

        let mut lfo = Lfo::new(sample_rate);
        lfo.set_center_value(EFFECTS_LFO_CENTER_VALUE);

        let lfo_parameters = LfoParameters {
            frequency: DEFAULT_LFO_FREQUENCY,
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
        let new_shape = effect.parameters[2];

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
            self.lfo.set_wave_shape(new_shape as u8);
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
    use accsyn_types::math::f32s_are_equal;

    #[test]
    fn autopan_process_samples_returns_original_when_disabled() {
        let mut autopan = AutoPan::new(44100);
        let effect = EffectParameters {
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
            is_enabled: true,
            parameters: vec![0.5, 0.5, 0.0, 0.0],
        };

        // Process once to set initial frequency
        autopan.process_samples((0.5, 0.5), &effect);
        let initial_frequency = autopan.lfo_parameters.frequency;

        // Change frequency parameter
        let new_effect = EffectParameters {
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
            is_enabled: true,
            parameters: vec![0.5, 0.3, 0.0, 0.0],
        };

        // Process once to set initial width
        autopan.process_samples((0.5, 0.5), &effect);
        let initial_width = autopan.lfo_parameters.width;

        // Change width parameter
        let new_effect = EffectParameters {
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
        let effect = EffectParameters {
            is_enabled: true,
            parameters: vec![0.5, 0.5, 0.0, 0.0],
        };

        // Process once to set initial shape
        autopan.process_samples((0.5, 0.5), &effect);
        let initial_shape = autopan.lfo_parameters.waveshape_index;

        // Change shape parameter
        let new_effect = EffectParameters {
            is_enabled: true,
            parameters: vec![0.5, 0.5, 1.0, 0.0],
        };
        autopan.process_samples((0.5, 0.5), &new_effect);

        assert!(!f32s_are_equal(
            initial_shape,
            autopan.lfo_parameters.waveshape_index
        ));
        assert!(f32s_are_equal(autopan.lfo_parameters.waveshape_index, 1.0));
    }

    #[test]
    fn autopan_process_samples_does_not_update_when_parameters_unchanged() {
        let mut autopan = AutoPan::new(44100);
        let effect = EffectParameters {
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
