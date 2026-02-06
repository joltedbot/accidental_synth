use crate::modules::effects::constants::{MAX_BITSHIFT_BITS, MIN_BITSHIFT_BITS};
use crate::modules::effects::{AudioEffect, EffectParameters};
use crate::synthesizer::midi_value_converters::normal_value_to_unsigned_integer_range;

pub struct BitShifter {}

impl BitShifter {
    pub fn new() -> Self {
        log::debug!("Constructing BitShifter Effect Module");

        Self {}
    }
}

impl AudioEffect for BitShifter {
    fn process_samples(&mut self, samples: (f32, f32), effect: &EffectParameters) -> (f32, f32) {
        if !effect.is_enabled {
            return samples;
        }

        let bits = normal_value_to_unsigned_integer_range(
            1.0 - effect.parameters[0],
            MIN_BITSHIFT_BITS,
            MAX_BITSHIFT_BITS,
        );

        (shift_sample(samples.0, bits), shift_sample(samples.1, bits))
    }
}

fn shift_sample(sample: f32, new_bit_depth: u32) -> f32 {
    let bits = (2_u32.pow(new_bit_depth) / 2) as f32;

    let quantized_sample = (sample.abs() * bits).ceil();
    let mut bitcrushed_sample = quantized_sample / bits;

    if sample.is_sign_negative() {
        bitcrushed_sample *= -1.0;
    }

    bitcrushed_sample
}

#[cfg(test)]
mod tests {
    use super::*;
    use accsyn_types::math::f32s_are_equal;

    #[test]
    fn bitshifter_process_samples_returns_original_when_disabled() {
        let mut bitshifter = BitShifter::new();
        let effect = EffectParameters {
            is_enabled: false,
            parameters: vec![0.5, 0.0, 0.0, 0.0],
        };
        let input = (0.6, -0.4);

        let result = bitshifter.process_samples(input, &effect);

        assert!(f32s_are_equal(result.0, 0.6));
        assert!(f32s_are_equal(result.1, -0.4));
    }

    #[test]
    fn shift_sample_quantizes_positive_sample() {
        let sample = 0.5;
        let bit_depth = 4; // 2^4 / 2 = 8 levels

        let result = shift_sample(sample, bit_depth);

        // 0.5 * 8 = 4.0, ceil = 4.0, 4.0 / 8 = 0.5
        assert!(f32s_are_equal(result, 0.5));
    }

    #[test]
    fn shift_sample_quantizes_negative_sample() {
        let sample = -0.5;
        let bit_depth = 4; // 2^4 / 2 = 8 levels

        let result = shift_sample(sample, bit_depth);

        // abs(-0.5) * 8 = 4.0, ceil = 4.0, 4.0 / 8 = 0.5, then negated
        assert!(f32s_are_equal(result, -0.5));
    }

    #[test]
    fn shift_sample_handles_zero() {
        let sample = 0.0;
        let bit_depth = 4;

        let result = shift_sample(sample, bit_depth);

        assert!(f32s_are_equal(result, 0.0));
    }

    #[test]
    fn shift_sample_handles_min_bit_depth() {
        let sample = 0.7;
        let bit_depth = MIN_BITSHIFT_BITS; // 1 bit: 2^1 / 2 = 1 level

        let result = shift_sample(sample, bit_depth);

        // 0.7 * 1 = 0.7, ceil = 1.0, 1.0 / 1 = 1.0
        assert!(f32s_are_equal(result, 1.0));
    }

    #[test]
    fn shift_sample_handles_max_bit_depth() {
        let sample = 0.5;
        let bit_depth = MAX_BITSHIFT_BITS; // 16 bits: 2^16 / 2 = 32768 levels

        let result = shift_sample(sample, bit_depth);

        // 0.5 * 32768 = 16384.0, ceil = 16384.0, 16384.0 / 32768 = 0.5
        assert!(f32s_are_equal(result, 0.5));
    }

    #[test]
    fn shift_sample_quantizes_low_bit_depth() {
        let sample = 0.6;
        let bit_depth = 2; // 2^2 / 2 = 2 levels

        let result = shift_sample(sample, bit_depth);

        // 0.6 * 2 = 1.2, ceil = 2.0, 2.0 / 2 = 1.0
        assert!(f32s_are_equal(result, 1.0));
    }

    #[test]
    fn shift_sample_preserves_sign_for_negative() {
        let sample = -0.7;
        let bit_depth = 3; // 2^3 / 2 = 4 levels

        let result = shift_sample(sample, bit_depth);

        // abs(-0.7) * 4 = 2.8, ceil = 3.0, 3.0 / 4 = 0.75, then negated
        assert!(f32s_are_equal(result, -0.75));
    }

    #[test]
    fn shift_sample_quantizes_small_positive_value() {
        let sample = 0.1;
        let bit_depth = 3; // 2^3 / 2 = 4 levels

        let result = shift_sample(sample, bit_depth);

        // 0.1 * 4 = 0.4, ceil = 1.0, 1.0 / 4 = 0.25
        assert!(f32s_are_equal(result, 0.25));
    }

    #[test]
    fn shift_sample_quantizes_small_negative_value() {
        let sample = -0.1;
        let bit_depth = 3; // 2^3 / 2 = 4 levels

        let result = shift_sample(sample, bit_depth);

        // abs(-0.1) * 4 = 0.4, ceil = 1.0, 1.0 / 4 = 0.25, then negated
        assert!(f32s_are_equal(result, -0.25));
    }
}
