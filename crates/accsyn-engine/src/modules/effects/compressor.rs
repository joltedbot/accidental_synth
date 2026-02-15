use crate::modules::effects::constants::{
    MAX_MAKEUP_GAIN_FACTOR, MAX_RATIO, MIN_MAKEUP_GAIN_FACTOR, MIN_RATIO,
};
use crate::synthesizer::midi_value_converters::{
    normal_value_to_f32_range, normal_value_to_unsigned_integer_range,
};
use accsyn_types::defaults::MAX_SAMPLE_VALUE;
use accsyn_types::effects::{AudioEffect, EffectParameters};

pub struct Compressor {}

impl Compressor {
    pub fn new() -> Self {
        log::debug!("Constructing Compressor Effect Module");

        Self {}
    }
}

impl AudioEffect for Compressor {
    fn process_samples(&mut self, samples: (f32, f32), effect: &EffectParameters) -> (f32, f32) {
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

#[cfg(test)]
mod tests {
    use super::*;
    use accsyn_types::math::f32s_are_equal;

    #[test]
    fn compressor_process_samples_returns_original_when_disabled() {
        let mut compressor = Compressor::new();
        let effect = EffectParameters {
            is_enabled: false,
            parameters: vec![0.5, 0.5, 0.5, 0.0],
        };
        let input = (0.7, -0.4);

        let result = compressor.process_samples(input, &effect);

        assert!(f32s_are_equal(result.0, 0.7));
        assert!(f32s_are_equal(result.1, -0.4));
    }

    #[test]
    fn compress_sample_clamps_threshold_to_max_sample_value() {
        let sample = 0.5;
        let threshold = MAX_SAMPLE_VALUE + 1.0; // Above max
        let normalized_ratio = 0.5;
        let makeup_gain = 0.5;

        // Should not panic and should clamp threshold
        let result = compress_sample(sample, threshold, normalized_ratio, makeup_gain);

        assert!(result.is_finite());
    }

    #[test]
    fn compress_sample_returns_sample_with_gain_when_below_threshold() {
        let sample = 0.3;
        let threshold = 0.5;
        let normalized_ratio = 0.5;
        let makeup_gain = 0.5;
        let gain_factor =
            normal_value_to_f32_range(makeup_gain, MIN_MAKEUP_GAIN_FACTOR, MAX_MAKEUP_GAIN_FACTOR);
        let expected = sample * gain_factor;

        let result = compress_sample(sample, threshold, normalized_ratio, makeup_gain);

        assert!(
            f32s_are_equal(result, expected),
            "Expected: {expected}, got: {result}"
        );
    }

    #[test]
    fn compress_sample_returns_sample_with_gain_when_at_threshold() {
        let threshold = 0.5;
        let sample = threshold; // Exactly at threshold
        let normalized_ratio = 0.5;
        let makeup_gain = 0.5;
        let gain_factor =
            normal_value_to_f32_range(makeup_gain, MIN_MAKEUP_GAIN_FACTOR, MAX_MAKEUP_GAIN_FACTOR);
        let expected = sample * gain_factor;

        let result = compress_sample(sample, threshold, normalized_ratio, makeup_gain);

        assert!(
            f32s_are_equal(result, expected),
            "Expected: {expected}, got: {result}"
        );
    }

    #[test]
    fn compress_sample_compresses_when_above_threshold() {
        let sample = 0.8;
        let threshold = 0.5;
        let normalized_ratio = 0.5;
        let makeup_gain = 0.5;

        let result = compress_sample(sample, threshold, normalized_ratio, makeup_gain);

        // Should be compressed (result should be less than input)
        assert!(result.abs() < sample);
        assert!(result > 0.0); // Should maintain sign
    }

    #[test]
    fn compress_sample_handles_negative_samples_below_threshold() {
        let sample = -0.3;
        let threshold = 0.5;
        let normalized_ratio = 0.5;
        let makeup_gain = 0.5;
        let gain_factor =
            normal_value_to_f32_range(makeup_gain, MIN_MAKEUP_GAIN_FACTOR, MAX_MAKEUP_GAIN_FACTOR);
        let expected = sample * gain_factor;

        let result = compress_sample(sample, threshold, normalized_ratio, makeup_gain);

        assert!(
            f32s_are_equal(result, expected),
            "Expected: {expected}, got: {result}"
        );
    }

    #[test]
    fn compress_sample_handles_negative_samples_above_threshold() {
        let sample = -0.8;
        let threshold = 0.5;
        let normalized_ratio = 0.5;
        let makeup_gain = 0.5;

        let result = compress_sample(sample, threshold, normalized_ratio, makeup_gain);

        // Should be compressed and maintain negative sign
        assert!(result.abs() < sample.abs());
        assert!(result < 0.0);
    }

    #[test]
    fn compress_sample_uses_ratio_in_compression() {
        let sample = 0.8;
        let threshold = 0.4;
        let low_ratio = 0.1; // Lower compression
        let high_ratio = 0.9; // Higher compression
        let makeup_gain = 0.5;

        let result_low_ratio = compress_sample(sample, threshold, low_ratio, makeup_gain);
        let result_high_ratio = compress_sample(sample, threshold, high_ratio, makeup_gain);

        // Higher ratio should compress more (smaller result)
        assert!(result_high_ratio.abs() < result_low_ratio.abs());
    }

    #[test]
    fn compress_sample_applies_makeup_gain() {
        let sample = 0.8;
        let threshold = 0.5;
        let normalized_ratio = 0.5;
        let low_gain = 0.2;
        let high_gain = 0.8;

        let result_low_gain = compress_sample(sample, threshold, normalized_ratio, low_gain);
        let result_high_gain = compress_sample(sample, threshold, normalized_ratio, high_gain);

        // Higher makeup gain should produce larger result
        assert!(result_high_gain.abs() > result_low_gain.abs());
    }

    #[test]
    fn compress_sample_preserves_sign_with_signum() {
        let threshold = 0.3;
        let normalized_ratio = 0.5;
        let makeup_gain = 0.5;

        // Test positive sample
        let positive_result = compress_sample(0.8, threshold, normalized_ratio, makeup_gain);
        assert!(positive_result > 0.0);

        // Test negative sample
        let negative_result = compress_sample(-0.8, threshold, normalized_ratio, makeup_gain);
        assert!(negative_result < 0.0);
    }
}
