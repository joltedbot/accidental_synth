use crate::modules::effects::{AudioEffect, EffectParameters};
use crate::synthesizer::midi_value_converters::normal_value_to_bool;

pub struct Rectifier {}

impl Rectifier {
    pub fn new() -> Self {
        Self {}
    }
}

impl AudioEffect for Rectifier {
    fn process_samples(&self, samples: (f32, f32), effect: &EffectParameters) -> (f32, f32) {
        if !effect.is_enabled {
            return samples;
        }

        rectify(samples, normal_value_to_bool(effect.parameters[0]))
    }
}

fn rectify(samples: (f32, f32), use_full_wave: bool) -> (f32, f32) {
    if use_full_wave {
        (
            full_wave_rectifier(samples.0),
            full_wave_rectifier(samples.1),
        )
    } else {
        (
            half_wave_rectifier(samples.0),
            half_wave_rectifier(samples.1),
        )
    }
}

fn half_wave_rectifier(sample: f32) -> f32 {
    if sample > 0.0 { sample } else { 0.0 }
}

fn full_wave_rectifier(sample: f32) -> f32 {
    if sample.is_sign_negative() {
        sample.abs()
    } else {
        sample
    }
}
