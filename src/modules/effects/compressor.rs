use crate::defaults::MAX_SAMPLE_VALUE;
use crate::modules::effects::constants::{
    MAX_MAKEUP_GAIN_FACTOR, MAX_RATIO, MIN_MAKEUP_GAIN_FACTOR, MIN_RATIO,
};
use crate::modules::effects::{AudioEffect, EffectParameters};
use crate::synthesizer::midi_value_converters::{
    normal_value_to_f32_range, normal_value_to_unsigned_integer_range,
};

pub struct Compressor {}

impl Compressor {
    pub fn new() -> Self {
        Self {}
    }
}

impl AudioEffect for Compressor {
    fn process_samples(&self, samples: (f32, f32), effect: &EffectParameters) -> (f32, f32) {
        if !effect.is_enabled {
            return samples;
        }

        (
            compress_sample(
                samples.0,
                effect.parameters[0],
                effect.parameters[1],
                effect.parameters[2],
            ),
            compress_sample(
                samples.1,
                effect.parameters[0],
                effect.parameters[1],
                effect.parameters[2],
            ),
        )
    }
}

fn compress_sample(
    sample: f32,
    mut threshold: f32,
    normalized_ratio: f32,
    makeup_gain: f32,
) -> f32 {
    threshold = threshold.min(MAX_SAMPLE_VALUE);

    let gain_factor =
        normal_value_to_f32_range(makeup_gain, MIN_MAKEUP_GAIN_FACTOR, MAX_MAKEUP_GAIN_FACTOR);

    if sample.abs() <= threshold {
        return sample * gain_factor;
    }

    let delta = sample.abs() - threshold;
    let ratio = normal_value_to_unsigned_integer_range(
        normalized_ratio,
        u32::from(MIN_RATIO),
        u32::from(MAX_RATIO),
    );

    (threshold + (delta / ratio as f32)) * gain_factor * sample.signum()
}
