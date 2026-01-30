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

        if new_frequency != self.lfo_parameters.frequency {
            self.lfo_parameters.frequency = new_frequency;
            self.lfo
                .set_frequency(exponential_curve_lfo_frequency_from_normal_value(
                    new_frequency,
                ));
        }
        if new_depth != self.lfo_parameters.depth {
            self.lfo_parameters.depth = new_depth;
            self.lfo.set_range(new_depth);
        }

        if new_shape != self.lfo_parameters.oscillator_index {
            self.lfo_parameters.oscillator_index = new_shape;
            self.lfo.set_wave_shape(new_shape as u8);
        }

        let lfo_value = self.lfo.generate(None);
        let tremolo = 1.0 - lfo_value;
        (samples.0 * tremolo, samples.1 * tremolo)
    }
}
