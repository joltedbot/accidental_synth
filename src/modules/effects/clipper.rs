use crate::defaults::MAX_SAMPLE_VALUE;
use crate::modules::effects::{AudioEffect, EffectParameters};
use crate::synthesizer::midi_value_converters::normal_value_to_bool;

pub struct Clipper {}

impl Clipper {
    pub fn new() -> Self {
        Self {}
    }
}

impl AudioEffect for Clipper {
    fn process_samples(&self, samples: (f32, f32), effect: &EffectParameters) -> (f32, f32) {
        if !effect.is_enabled {
            return samples;
        }

        (
            clip_sample(
                samples.0,
                effect.parameters[0],
                effect.parameters[1],
                effect.parameters[2],
                normal_value_to_bool(effect.parameters[3]),
            ),
            clip_sample(
                samples.1,
                effect.parameters[0],
                effect.parameters[1],
                effect.parameters[2],
                normal_value_to_bool(effect.parameters[3]),
            ),
        )
    }
}

fn clip_sample(sample: f32, mut threshold: f32, pre_gain: f32, post_gain: f32, notch: bool) -> f32 {
    threshold = threshold.min(MAX_SAMPLE_VALUE);

    let mut boosted_sample = sample * (1.0 + pre_gain.abs());

    if boosted_sample.abs() > threshold {
        if notch {
            boosted_sample = 0.0;
        } else {
            boosted_sample = threshold * sample.signum();
        }
    }

    (boosted_sample * (1.0 + post_gain.abs())).clamp(-MAX_SAMPLE_VALUE, MAX_SAMPLE_VALUE)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::math::f32s_are_equal;

    #[test]
    fn clipper_process_samples_returns_original_when_disabled() {
        let clipper = Clipper::new();
        let effect = EffectParameters {
            is_enabled: false,
            parameters: vec![0.5, 0.0, 0.0, 0.0],
        };
        let input = (0.8, -0.6);

        let result = clipper.process_samples(input, &effect);

        assert!(f32s_are_equal(result.0, 0.8));
        assert!(f32s_are_equal(result.1, -0.6));
    }

    #[test]
    fn clip_sample_passes_sample_below_threshold() {
        let sample = 0.3;
        let threshold = 0.5;
        let pre_gain = 0.0;
        let post_gain = 0.0;
        let notch = false;

        let result = clip_sample(sample, threshold, pre_gain, post_gain, notch);

        assert!(f32s_are_equal(result, 0.3));
    }

    #[test]
    fn clip_sample_clips_sample_above_threshold_without_notch() {
        let sample = 0.8;
        let threshold = 0.5;
        let pre_gain = 0.0;
        let post_gain = 0.0;
        let notch = false;

        let result = clip_sample(sample, threshold, pre_gain, post_gain, notch);

        assert!(f32s_are_equal(result, 0.5));
    }

    #[test]
    fn clip_sample_clips_negative_sample_above_threshold_without_notch() {
        let sample = -0.8;
        let threshold = 0.5;
        let pre_gain = 0.0;
        let post_gain = 0.0;
        let notch = false;

        let result = clip_sample(sample, threshold, pre_gain, post_gain, notch);

        assert!(f32s_are_equal(result, -0.5));
    }

    #[test]
    fn clip_sample_zeroes_sample_above_threshold_with_notch() {
        let sample = 0.8;
        let threshold = 0.5;
        let pre_gain = 0.0;
        let post_gain = 0.0;
        let notch = true;

        let result = clip_sample(sample, threshold, pre_gain, post_gain, notch);

        assert!(f32s_are_equal(result, 0.0));
    }

    #[test]
    fn clip_sample_zeroes_negative_sample_above_threshold_with_notch() {
        let sample = -0.8;
        let threshold = 0.5;
        let pre_gain = 0.0;
        let post_gain = 0.0;
        let notch = true;

        let result = clip_sample(sample, threshold, pre_gain, post_gain, notch);

        assert!(f32s_are_equal(result, 0.0));
    }

    #[test]
    fn clip_sample_handles_sample_exactly_at_threshold() {
        let sample = 0.5;
        let threshold = 0.5;
        let pre_gain = 0.0;
        let post_gain = 0.0;
        let notch = false;

        let result = clip_sample(sample, threshold, pre_gain, post_gain, notch);

        assert!(f32s_are_equal(result, 0.5));
    }

    #[test]
    fn clip_sample_clamps_threshold_to_max_sample_value() {
        let sample = 0.5;
        let threshold = 2.0; // Above MAX_SAMPLE_VALUE
        let pre_gain = 0.0;
        let post_gain = 0.0;
        let notch = false;

        let result = clip_sample(sample, threshold, pre_gain, post_gain, notch);

        // Threshold clamped to 1.0, sample is below it
        assert!(f32s_are_equal(result, 0.5));
    }

    #[test]
    fn clip_sample_applies_pre_gain() {
        let sample = 0.4;
        let threshold = 0.5;
        let pre_gain = 1.0; // 2x boost
        let post_gain = 0.0;
        let notch = false;

        let result = clip_sample(sample, threshold, pre_gain, post_gain, notch);

        // Boosted: 0.4 * 2.0 = 0.8, which exceeds 0.5 threshold, clipped to 0.5
        assert!(f32s_are_equal(result, 0.5));
    }

    #[test]
    fn clip_sample_applies_post_gain() {
        let sample = 0.2;
        let threshold = 0.5;
        let pre_gain = 0.0;
        let post_gain = 1.0; // 2x boost
        let notch = false;

        let result = clip_sample(sample, threshold, pre_gain, post_gain, notch);

        // Sample passes through, then boosted: 0.2 * 2.0 = 0.4
        assert!(f32s_are_equal(result, 0.4));
    }

    #[test]
    fn clip_sample_clamps_output_to_max_sample_value() {
        let sample = 0.8;
        let threshold = 0.9;
        let pre_gain = 0.0;
        let post_gain = 1.0; // 2x boost
        let notch = false;

        let result = clip_sample(sample, threshold, pre_gain, post_gain, notch);

        // Sample passes through (0.8 < 0.9), then boosted: 0.8 * 2.0 = 1.6
        // Clamped to MAX_SAMPLE_VALUE (1.0)
        assert!(f32s_are_equal(result, 1.0));
    }

    #[test]
    fn clip_sample_clamps_negative_output_to_min_sample_value() {
        let sample = -0.8;
        let threshold = 0.9;
        let pre_gain = 0.0;
        let post_gain = 1.0; // 2x boost
        let notch = false;

        let result = clip_sample(sample, threshold, pre_gain, post_gain, notch);

        // Sample passes through (-0.8 < 0.9), then boosted: -0.8 * 2.0 = -1.6
        // Clamped to -MAX_SAMPLE_VALUE (-1.0)
        assert!(f32s_are_equal(result, -1.0));
    }

    #[test]
    fn clip_sample_handles_negative_pre_gain() {
        let sample = 0.4;
        let threshold = 0.5;
        let pre_gain = -1.0; // abs() = 1.0, so 2x boost
        let post_gain = 0.0;
        let notch = false;

        let result = clip_sample(sample, threshold, pre_gain, post_gain, notch);

        // Boosted: 0.4 * 2.0 = 0.8, exceeds threshold, clipped to 0.5
        assert!(f32s_are_equal(result, 0.5));
    }

    #[test]
    fn clip_sample_handles_negative_post_gain() {
        let sample = 0.2;
        let threshold = 0.5;
        let pre_gain = 0.0;
        let post_gain = -1.0; // abs() = 1.0, so 2x boost
        let notch = false;

        let result = clip_sample(sample, threshold, pre_gain, post_gain, notch);

        // Sample passes through, then boosted: 0.2 * 2.0 = 0.4
        assert!(f32s_are_equal(result, 0.4));
    }
}
