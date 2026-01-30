use crate::math::f32s_are_equal;
use crate::modules::effects::constants::{AUTO_PAN_MAX_WIDTH, EFFECTS_LFO_CENTER_VALUE};
use crate::modules::effects::{AudioEffect, EffectParameters};
use crate::modules::lfo::Lfo;
use crate::synthesizer::midi_value_converters::exponential_curve_lfo_frequency_from_normal_value;
use std::f32::consts::PI;

#[derive(Debug, Default)]
struct LfoParameters {
    frequency: f32,
    width: f32,
    oscillator_index: f32,
}

pub struct AutoPan {
    lfo: Lfo,
    lfo_parameters: LfoParameters,
}

impl AutoPan {
    pub fn new(sample_rate: u32) -> Self {
        let mut lfo = Lfo::new(sample_rate);
        lfo.set_center_value(EFFECTS_LFO_CENTER_VALUE);

        let lfo_parameters = LfoParameters {
            width: AUTO_PAN_MAX_WIDTH,
            ..Default::default()
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

        if !f32s_are_equal(new_shape, self.lfo_parameters.oscillator_index) {
            self.lfo_parameters.oscillator_index = new_shape;
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
