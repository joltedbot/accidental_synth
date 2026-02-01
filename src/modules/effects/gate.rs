use crate::defaults::MAX_SAMPLE_VALUE;
use crate::modules::effects::{AudioEffect, EffectParameters};

pub struct Gate {}

impl Gate {
    pub fn new() -> Self {
        log::debug!("Constructing Gate Effect Module");

        Self {}
    }
}

impl AudioEffect for Gate {
    fn process_samples(&mut self, samples: (f32, f32), effect: &EffectParameters) -> (f32, f32) {
        if !effect.is_enabled || effect.parameters[0] == 0.0 {
            return samples;
        }

        (
            gate_sample(
                samples.0,
                effect.parameters[0],
                effect.parameters[1],
                effect.parameters[2],
            ),
            gate_sample(
                samples.1,
                effect.parameters[0],
                effect.parameters[1],
                effect.parameters[2],
            ),
        )
    }
}

fn gate_sample(sample: f32, threshold: f32, pre_gain: f32, post_gain: f32) -> f32 {
    let mut boosted_sample = sample * pre_gain.abs();

    if boosted_sample.abs() < threshold {
        boosted_sample = 0.0;
    }

    (boosted_sample * (1.0 + post_gain.abs())).clamp(-MAX_SAMPLE_VALUE, MAX_SAMPLE_VALUE)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::math::f32s_are_equal;

    #[test]
    fn gate_process_samples_returns_original_when_disabled() {
        let mut gate = Gate::new();
        let effect = EffectParameters {
            is_enabled: false,
            parameters: vec![0.5, 0.0, 0.0, 0.0],
        };
        let input = (0.8, -0.6);
        let expected_left = 0.8;
        let expected_right = -0.6;

        let result = gate.process_samples(input, &effect);

        assert!(
            f32s_are_equal(result.0, expected_left),
            "Left channel: Expected: {expected_left}, got: {result_left:?}",
            result_left = result.0
        );
        assert!(
            f32s_are_equal(result.1, expected_right),
            "Right channel: Expected: {expected_right}, got: {result_right:?}",
            result_right = result.1
        );
    }

    #[test]
    fn gate_sample_passes_sample_above_threshold() {
        let sample = 0.8;
        let threshold = 0.5;
        let pre_gain = 1.0; // No boost (multiply by 1.0)
        let post_gain = 0.0; // No boost (multiply by 1.0)
        let expected_result = 0.8;

        let result = gate_sample(sample, threshold, pre_gain, post_gain);

        assert!(
            f32s_are_equal(result, expected_result),
            "Expected: {expected_result}, got: {result:?}"
        );
    }

    #[test]
    fn gate_sample_zeroes_sample_below_threshold() {
        let sample = 0.3;
        let threshold = 0.5;
        let pre_gain = 1.0;
        let post_gain = 0.0;
        let expected_result = 0.0;

        let result = gate_sample(sample, threshold, pre_gain, post_gain);

        assert!(
            f32s_are_equal(result, expected_result),
            "Expected: {expected_result}, got: {result:?}"
        );
    }

    #[test]
    fn gate_sample_zeroes_negative_sample_below_threshold() {
        let sample = -0.3;
        let threshold = 0.5;
        let pre_gain = 1.0;
        let post_gain = 0.0;
        let expected_result = 0.0;

        let result = gate_sample(sample, threshold, pre_gain, post_gain);

        assert!(
            f32s_are_equal(result, expected_result),
            "Expected: {expected_result}, got: {result:?}"
        );
    }

    #[test]
    fn gate_sample_passes_negative_sample_above_threshold() {
        let sample = -0.8;
        let threshold = 0.5;
        let pre_gain = 1.0;
        let post_gain = 0.0;
        let expected_result = -0.8;

        let result = gate_sample(sample, threshold, pre_gain, post_gain);

        assert!(
            f32s_are_equal(result, expected_result),
            "Expected: {expected_result}, got: {result:?}"
        );
    }

    #[test]
    fn gate_sample_handles_sample_exactly_at_threshold() {
        let sample = 0.5;
        let threshold = 0.5;
        let pre_gain = 1.0;
        let post_gain = 0.0;
        let expected_result = 0.5;

        let result = gate_sample(sample, threshold, pre_gain, post_gain);

        assert!(
            f32s_are_equal(result, expected_result),
            "Expected: {expected_result}, got: {result:?}"
        );
    }

    #[test]
    fn gate_sample_applies_pre_gain() {
        let sample = 0.3;
        let threshold = 0.5;
        let pre_gain = 2.0; // 2x boost
        let post_gain = 0.0;
        let expected_result = 0.6; // Boosted: 0.3 * 2.0 = 0.6, which is above threshold, passes through

        let result = gate_sample(sample, threshold, pre_gain, post_gain);

        assert!(
            f32s_are_equal(result, expected_result),
            "Expected: {expected_result}, got: {result:?}"
        );
    }

    #[test]
    fn gate_sample_pre_gain_causes_gate_to_open() {
        let sample = 0.4;
        let threshold = 0.5;
        let pre_gain = 2.0; // 2x boost
        let post_gain = 0.0;
        let expected_result = 0.8; // Boosted: 0.4 * 2.0 = 0.8, which is above threshold, passes through

        let result = gate_sample(sample, threshold, pre_gain, post_gain);

        assert!(
            f32s_are_equal(result, expected_result),
            "Expected: {expected_result}, got: {result:?}"
        );
    }

    #[test]
    fn gate_sample_applies_post_gain_to_passed_sample() {
        let sample = 0.6;
        let threshold = 0.5;
        let pre_gain = 1.0;
        let post_gain = 1.0; // 2x boost (1.0 + 1.0)
        let expected_result = 1.0; // Sample passes through (0.6 > 0.5), then boosted: 0.6 * 2.0 = 1.2, clamped to 1.0

        let result = gate_sample(sample, threshold, pre_gain, post_gain);

        assert!(
            f32s_are_equal(result, expected_result),
            "Expected: {expected_result}, got: {result:?}"
        );
    }

    #[test]
    fn gate_sample_clamps_output_to_max_sample_value() {
        let sample = 0.8;
        let threshold = 0.5;
        let pre_gain = 1.0;
        let post_gain = 1.0; // 2x boost
        let expected_result = 1.0; // Sample passes through (0.8 > 0.5), then boosted: 0.8 * 2.0 = 1.6, clamped to MAX_SAMPLE_VALUE (1.0)

        let result = gate_sample(sample, threshold, pre_gain, post_gain);

        assert!(
            f32s_are_equal(result, expected_result),
            "Expected: {expected_result}, got: {result:?}"
        );
    }

    #[test]
    fn gate_sample_clamps_negative_output_to_min_sample_value() {
        let sample = -0.8;
        let threshold = 0.5;
        let pre_gain = 1.0;
        let post_gain = 1.0; // 2x boost
        let expected_result = -1.0; // Sample passes through (abs(-0.8) = 0.8 > 0.5), then boosted: -0.8 * 2.0 = -1.6, clamped to -MAX_SAMPLE_VALUE (-1.0)

        let result = gate_sample(sample, threshold, pre_gain, post_gain);

        assert!(
            f32s_are_equal(result, expected_result),
            "Expected: {expected_result}, got: {result:?}"
        );
    }

    #[test]
    fn gate_sample_handles_negative_pre_gain() {
        let sample = 0.3;
        let threshold = 0.5;
        let pre_gain = -2.0; // abs() = 2.0
        let post_gain = 0.0;
        let expected_result = 0.6; // Boosted: 0.3 * 2.0 = 0.6, which is above threshold, passes through

        let result = gate_sample(sample, threshold, pre_gain, post_gain);

        assert!(
            f32s_are_equal(result, expected_result),
            "Expected: {expected_result}, got: {result:?}"
        );
    }

    #[test]
    fn gate_sample_handles_negative_post_gain() {
        let sample = 0.6;
        let threshold = 0.5;
        let pre_gain = 1.0;
        let post_gain = -1.0; // abs() = 1.0, so 2x boost
        let expected_result = 1.0; // Sample passes through (0.6 > 0.5), then boosted: 0.6 * 2.0 = 1.2, clamped to 1.0

        let result = gate_sample(sample, threshold, pre_gain, post_gain);

        assert!(
            f32s_are_equal(result, expected_result),
            "Expected: {expected_result}, got: {result:?}"
        );
    }
}
