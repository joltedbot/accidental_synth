use accsyn_core::defaults::Defaults;
use accsyn_core::effects::{AudioEffect, EffectParameters};

const THRESHOLD_INDEX: usize = 0;
const PREGAIN_INDEX: usize = 1;
const POSTGAIN_INDEX: usize = 2;
const NOTCH_INDEX: usize = 3;

pub struct Gate {}

impl Gate {
    pub fn new() -> Self {
        log::debug!(target: "synthesizer::effects::gate", "Constructing Gate Effect Module");

        Self {}
    }
}

impl AudioEffect for Gate {
    fn process_samples(&mut self, samples: (f32, f32), effect: &EffectParameters) -> (f32, f32) {
        if !effect.is_enabled || effect.parameters[0] == 0.0 {
            return samples;
        }

        (
            gate_sample(samples.0, &effect.parameters),
            gate_sample(samples.1, &effect.parameters),
        )
    }
}

fn gate_sample(sample: f32, parameters: &[f32]) -> f32 {
    let sample_sign = sample.signum();
    let sample_magnitude = sample.abs();
    let threshold = parameters[THRESHOLD_INDEX];

    let mut boosted_sample_magnitude = sample_magnitude * parameters[PREGAIN_INDEX];
    let is_below_threshold = boosted_sample_magnitude < threshold;

    let gated_magnitude = if parameters[NOTCH_INDEX] > 0.0 {
        0.0
    } else {
        threshold
    };

    if is_below_threshold {
        boosted_sample_magnitude = gated_magnitude;
    }

    (boosted_sample_magnitude * sample_sign * (1.0 + parameters[POSTGAIN_INDEX]))
        .clamp(-Defaults::MAX_SAMPLE_VALUE, Defaults::MAX_SAMPLE_VALUE)
}

#[cfg(test)]
mod tests {
    use super::*;
    use accsyn_core::math::f32s_are_equal;

    #[test]
    fn gate_process_samples_returns_original_when_disabled() {
        let mut gate = Gate::new();
        let effect = EffectParameters {
            name: String::new(),
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
    fn gate_process_samples_returns_original_when_threshold_is_zero() {
        let mut gate = Gate::new();
        let effect = EffectParameters {
            name: String::new(),
            is_enabled: true,
            parameters: vec![0.0, 1.0, 0.0, 1.0],
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
    fn gate_process_samples_gates_both_channels_with_notch() {
        let mut gate = Gate::new();
        let effect = EffectParameters {
            name: String::new(),
            is_enabled: true,
            parameters: vec![0.5, 1.0, 0.0, 1.0],
        };
        let input = (0.8, 0.3);
        let expected_left = 0.8; // 0.8 is not below the 0.5 threshold, so it passes
        let expected_right = 0.0; // 0.3 is below the threshold, and notch zeroes it

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
        // threshold, pre-gain (direct multiplier), post-gain, notch
        let parameters = vec![0.5, 1.0, 0.0, 1.0];
        let expected_result = 0.8;

        let result = gate_sample(sample, &parameters);

        assert!(
            f32s_are_equal(result, expected_result),
            "Expected: {expected_result}, got: {result:?}"
        );
    }

    #[test]
    fn gate_sample_zeroes_sample_below_threshold_with_notch() {
        let sample = 0.3;
        let parameters = vec![0.5, 1.0, 0.0, 1.0]; // notch on
        let expected_result = 0.0; // 0.3 is below the 0.5 threshold, so notch zeroes it

        let result = gate_sample(sample, &parameters);

        assert!(
            f32s_are_equal(result, expected_result),
            "Expected: {expected_result}, got: {result:?}"
        );
    }

    #[test]
    fn gate_sample_holds_sample_below_threshold_at_threshold_without_notch() {
        let sample = 0.3;
        let parameters = vec![0.5, 1.0, 0.0, 0.0]; // notch off
        let expected_result = 0.5; // Without notch a below-threshold sample is held at the threshold

        let result = gate_sample(sample, &parameters);

        assert!(
            f32s_are_equal(result, expected_result),
            "Expected: {expected_result}, got: {result:?}"
        );
    }

    #[test]
    fn gate_sample_zeroes_negative_sample_with_notch() {
        let sample = -0.3;
        let parameters = vec![0.5, 1.0, 0.0, 1.0]; // notch on
        let expected_result = 0.0; // Magnitude 0.3 sits inside the +/-0.5 band, so notch zeroes it

        let result = gate_sample(sample, &parameters);

        assert!(
            f32s_are_equal(result, expected_result),
            "Expected: {expected_result}, got: {result:?}"
        );
    }

    #[test]
    fn gate_sample_passes_negative_sample_louder_than_threshold() {
        let sample = -0.8;
        let parameters = vec![0.5, 1.0, 0.0, 1.0];
        let expected_result = -0.8; // Magnitude 0.8 is outside the +/-0.5 band, so it passes through untouched

        let result = gate_sample(sample, &parameters);

        assert!(
            f32s_are_equal(result, expected_result),
            "Expected: {expected_result}, got: {result:?}"
        );
    }

    #[test]
    fn gate_sample_handles_negative_sample_exactly_at_threshold() {
        let sample = -0.5;
        let parameters = vec![0.5, 1.0, 0.0, 1.0];
        let expected_result = -0.5; // Mirrors the positive boundary case: the band is exclusive at both edges

        let result = gate_sample(sample, &parameters);

        assert!(
            f32s_are_equal(result, expected_result),
            "Expected: {expected_result}, got: {result:?}"
        );
    }

    #[test]
    fn gate_sample_zeroes_silence_with_notch() {
        let sample = 0.0;
        let parameters = vec![0.5, 1.0, 0.0, 1.0]; // notch on
        let expected_result = 0.0; // Silence is inside the band, so notch keeps it silent

        let result = gate_sample(sample, &parameters);

        assert!(
            f32s_are_equal(result, expected_result),
            "Expected: {expected_result}, got: {result:?}"
        );
    }

    #[test]
    fn gate_sample_holds_silence_at_threshold_without_notch() {
        let sample = 0.0;
        let parameters = vec![0.5, 1.0, 0.0, 0.0]; // notch off
        // Silence is inside the band, so it is held at the threshold, leaving a DC offset on a
        // silent input. This is intended — the harsh edge is the effect.
        let expected_result = 0.5;

        let result = gate_sample(sample, &parameters);

        assert!(
            f32s_are_equal(result, expected_result),
            "Expected: {expected_result}, got: {result:?}"
        );
    }

    #[test]
    fn gate_sample_holds_negative_silence_at_negative_threshold_without_notch() {
        let sample = -0.0;
        let parameters = vec![0.5, 1.0, 0.0, 0.0]; // notch off
        // Negative zero carries a sign: (-0.0).signum() is -1.0, so the held DC offset is inverted
        // relative to the +0.0 case above. Reachable whenever an upstream stage scales a negative
        // sample to zero.
        let expected_result = -0.5;

        let result = gate_sample(sample, &parameters);

        assert!(
            f32s_are_equal(result, expected_result),
            "Expected: {expected_result}, got: {result:?}"
        );
    }

    #[test]
    fn gate_sample_clamps_negative_output_to_min_sample_value() {
        let sample = -0.8;
        let parameters = vec![0.5, 1.0, 1.0, 1.0]; // 2x post-gain
        let expected_result = -1.0; // Passes the band, then boosted: -0.8 * 2.0 = -1.6, clamped to -MAX_SAMPLE_VALUE

        let result = gate_sample(sample, &parameters);

        assert!(
            f32s_are_equal(result, expected_result),
            "Expected: {expected_result}, got: {result:?}"
        );
    }

    #[test]
    fn gate_sample_holds_negative_sample_at_threshold_without_notch() {
        let sample = -0.3;
        let parameters = vec![0.5, 1.0, 0.0, 0.0]; // notch off
        let expected_result = -0.5; // Held at the threshold, with the input's sign reapplied

        let result = gate_sample(sample, &parameters);

        assert!(
            f32s_are_equal(result, expected_result),
            "Expected: {expected_result}, got: {result:?}"
        );
    }

    #[test]
    fn gate_sample_handles_sample_exactly_at_threshold() {
        let sample = 0.5;
        let parameters = vec![0.5, 1.0, 0.0, 1.0];
        let expected_result = 0.5; // The comparison is strictly less-than, so the sample passes

        let result = gate_sample(sample, &parameters);

        assert!(
            f32s_are_equal(result, expected_result),
            "Expected: {expected_result}, got: {result:?}"
        );
    }

    #[test]
    fn gate_sample_applies_pre_gain() {
        let sample = 0.8;
        let parameters = vec![0.2, 0.5, 0.0, 1.0]; // Pre-gain is a direct multiplier, so 0.5 halves the sample
        let expected_result = 0.4; // Cut: 0.8 * 0.5 = 0.4, still above the 0.2 threshold, so it passes

        let result = gate_sample(sample, &parameters);

        assert!(
            f32s_are_equal(result, expected_result),
            "Expected: {expected_result}, got: {result:?}"
        );
    }

    #[test]
    fn gate_sample_pre_gain_causes_gate_to_close() {
        let sample = 0.8;
        let parameters = vec![0.5, 0.5, 0.0, 1.0]; // Cutting the level pushes the sample under the threshold
        let expected_result = 0.0; // Cut: 0.8 * 0.5 = 0.4, now below the 0.5 threshold, so notch zeroes it

        let result = gate_sample(sample, &parameters);

        assert!(
            f32s_are_equal(result, expected_result),
            "Expected: {expected_result}, got: {result:?}"
        );
    }

    #[test]
    fn gate_sample_zero_pre_gain_closes_gate_entirely() {
        let sample = 0.8;
        let parameters = vec![0.5, 0.0, 0.0, 1.0]; // Pre-gain at the bottom of its range
        let expected_result = 0.0; // Cut: 0.8 * 0.0 = 0.0, below any positive threshold, so the gate never opens

        let result = gate_sample(sample, &parameters);

        assert!(
            f32s_are_equal(result, expected_result),
            "Expected: {expected_result}, got: {result:?}"
        );
    }

    #[test]
    fn gate_sample_applies_post_gain_to_passed_sample() {
        let sample = 0.3;
        let parameters = vec![0.2, 1.0, 1.0, 1.0]; // 2x post-gain (1.0 + 1.0)
        let expected_result = 0.6; // Sample passes (0.3 > 0.2), then boosted: 0.3 * 2.0 = 0.6

        let result = gate_sample(sample, &parameters);

        assert!(
            f32s_are_equal(result, expected_result),
            "Expected: {expected_result}, got: {result:?}"
        );
    }

    #[test]
    fn gate_sample_applies_post_gain_to_held_threshold_without_notch() {
        let sample = 0.1;
        let parameters = vec![0.3, 1.0, 1.0, 0.0]; // notch off, 2x post-gain
        let expected_result = 0.6; // Held at the 0.3 threshold, then boosted: 0.3 * 2.0 = 0.6

        let result = gate_sample(sample, &parameters);

        assert!(
            f32s_are_equal(result, expected_result),
            "Expected: {expected_result}, got: {result:?}"
        );
    }

    #[test]
    fn gate_sample_clamps_output_to_max_sample_value() {
        let sample = 0.8;
        let parameters = vec![0.5, 1.0, 1.0, 1.0]; // 2x post-gain
        let expected_result = 1.0; // Sample passes, then boosted: 0.8 * 2.0 = 1.6, clamped to MAX_SAMPLE_VALUE

        let result = gate_sample(sample, &parameters);

        assert!(
            f32s_are_equal(result, expected_result),
            "Expected: {expected_result}, got: {result:?}"
        );
    }
}
