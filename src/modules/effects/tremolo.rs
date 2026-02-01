use crate::math::f32s_are_equal;
use crate::modules::effects::constants::{EFFECTS_LFO_CENTER_VALUE, TREMOLO_MAX_DEPTH};
use crate::modules::effects::{AudioEffect, EffectParameters};
use crate::modules::lfo::Lfo;
use crate::synthesizer::midi_value_converters::exponential_curve_lfo_frequency_from_normal_value;

#[derive(Debug, Default)]
struct LfoParameters {
    frequency: f32,
    depth: f32,
    oscillator_index: f32,
}

pub struct Tremolo {
    lfo: Lfo,
    lfo_parameters: LfoParameters,
}

impl Tremolo {
    pub fn new(sample_rate: u32) -> Self {
        log::debug!("Constructing Tremolo Effect Module");

        let mut lfo = Lfo::new(sample_rate);
        lfo.set_center_value(EFFECTS_LFO_CENTER_VALUE);

        let lfo_parameters = LfoParameters {
            depth: TREMOLO_MAX_DEPTH,
            ..Default::default()
        };

        Self {
            lfo,
            lfo_parameters,
        }
    }
}

impl AudioEffect for Tremolo {
    fn process_samples(&mut self, samples: (f32, f32), effect: &EffectParameters) -> (f32, f32) {
        if !effect.is_enabled {
            return samples;
        }

        let new_frequency = effect.parameters[0];
        let new_depth = effect.parameters[1];
        let new_shape = effect.parameters[2];

        if !f32s_are_equal(new_frequency, self.lfo_parameters.frequency) {
            self.lfo_parameters.frequency = new_frequency;
            self.lfo
                .set_frequency(exponential_curve_lfo_frequency_from_normal_value(
                    new_frequency,
                ));
        }
        if !f32s_are_equal(new_depth, self.lfo_parameters.depth) {
            self.lfo_parameters.depth = new_depth;
            self.lfo.set_range(new_depth);
        }

        if !f32s_are_equal(new_shape, self.lfo_parameters.oscillator_index) {
            self.lfo_parameters.oscillator_index = new_shape;
            self.lfo.set_wave_shape(new_shape as u8);
        }

        let lfo_value = self.lfo.generate(None);
        let tremolo = 1.0 - lfo_value;
        (samples.0 * tremolo, samples.1 * tremolo)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tremolo_process_samples_returns_original_when_disabled() {
        let mut tremolo = Tremolo::new(44100);
        let effect = EffectParameters {
            is_enabled: false,
            parameters: vec![0.0, 0.0, 0.0, 0.0],
        };
        let input = (0.5, -0.3);

        let result = tremolo.process_samples(input, &effect);

        assert!(f32s_are_equal(result.0, 0.5));
        assert!(f32s_are_equal(result.1, -0.3));
    }

    #[test]
    fn tremolo_process_samples_updates_frequency_when_changed() {
        let mut tremolo = Tremolo::new(44100);
        let effect = EffectParameters {
            is_enabled: true,
            parameters: vec![0.5, 0.5, 0.0, 0.0],
        };

        // Process once to set initial frequency
        tremolo.process_samples((0.5, 0.5), &effect);
        let initial_frequency = tremolo.lfo_parameters.frequency;

        // Change frequency parameter
        let new_effect = EffectParameters {
            is_enabled: true,
            parameters: vec![0.8, 0.5, 0.0, 0.0],
        };
        tremolo.process_samples((0.5, 0.5), &new_effect);

        assert!(!f32s_are_equal(
            initial_frequency,
            tremolo.lfo_parameters.frequency
        ));
        assert!(f32s_are_equal(tremolo.lfo_parameters.frequency, 0.8));
    }

    #[test]
    fn tremolo_process_samples_updates_depth_when_changed() {
        let mut tremolo = Tremolo::new(44100);
        let effect = EffectParameters {
            is_enabled: true,
            parameters: vec![0.5, 0.3, 0.0, 0.0],
        };

        // Process once to set initial depth
        tremolo.process_samples((0.5, 0.5), &effect);
        let initial_depth = tremolo.lfo_parameters.depth;

        // Change depth parameter
        let new_effect = EffectParameters {
            is_enabled: true,
            parameters: vec![0.5, 0.7, 0.0, 0.0],
        };
        tremolo.process_samples((0.5, 0.5), &new_effect);

        assert!(!f32s_are_equal(initial_depth, tremolo.lfo_parameters.depth));
        assert!(f32s_are_equal(tremolo.lfo_parameters.depth, 0.7));
    }

    #[test]
    fn tremolo_process_samples_updates_shape_when_changed() {
        let mut tremolo = Tremolo::new(44100);
        let effect = EffectParameters {
            is_enabled: true,
            parameters: vec![0.5, 0.5, 0.0, 0.0],
        };

        // Process once to set initial shape
        tremolo.process_samples((0.5, 0.5), &effect);
        let initial_shape = tremolo.lfo_parameters.oscillator_index;

        // Change shape parameter
        let new_effect = EffectParameters {
            is_enabled: true,
            parameters: vec![0.5, 0.5, 1.0, 0.0],
        };
        tremolo.process_samples((0.5, 0.5), &new_effect);

        assert!(!f32s_are_equal(
            initial_shape,
            tremolo.lfo_parameters.oscillator_index
        ));
        assert!(f32s_are_equal(tremolo.lfo_parameters.oscillator_index, 1.0));
    }

    #[test]
    fn tremolo_process_samples_does_not_update_when_parameters_unchanged() {
        let mut tremolo = Tremolo::new(44100);
        let effect = EffectParameters {
            is_enabled: true,
            parameters: vec![0.5, 0.6, 2.0, 0.0],
        };

        // Process once to set parameters
        tremolo.process_samples((0.5, 0.5), &effect);
        let initial_frequency = tremolo.lfo_parameters.frequency;
        let initial_depth = tremolo.lfo_parameters.depth;
        let initial_shape = tremolo.lfo_parameters.oscillator_index;

        // Process again with same parameters
        tremolo.process_samples((0.5, 0.5), &effect);

        // Parameters should remain the same
        assert!(f32s_are_equal(
            initial_frequency,
            tremolo.lfo_parameters.frequency
        ));
        assert!(f32s_are_equal(initial_depth, tremolo.lfo_parameters.depth));
        assert!(f32s_are_equal(
            initial_shape,
            tremolo.lfo_parameters.oscillator_index
        ));
    }
}
