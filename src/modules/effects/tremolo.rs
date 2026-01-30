use crate::modules::effects::{AudioEffect, EffectParameters};


pub struct Tremolo {}

impl Tremolo {
    pub fn new(sample_rate: u32) -> Self {
        Self {}
    }
}

impl AudioEffect for Tremolo {
    fn process_samples(&mut self, samples: (f32, f32), effect: &EffectParameters) -> (f32, f32) {
        if !effect.is_enabled {
            return samples;
        }

        (
            tremolo_sample(
                samples.0,
                effect.parameters[0],
                effect.parameters[1],
                effect.parameters[2],
            ),
            tremolo_sample(
                samples.1,
                effect.parameters[0],
                effect.parameters[1],
                effect.parameters[2],
            ),
        )
    }
}

fn tremolo_sample(sample: f32, mut rate: f32, depth: f32, shape: f32) -> f32 {
    0.0
}