use crate::modules::effects::constants::MAX_THRESHOLD;
use accsyn_types::effects::{AudioEffect, EffectParameters};

pub struct WaveFolder {}
impl WaveFolder {
    pub fn new() -> Self {
        log::debug!("Constructing WaveFolder Effect Module");

        Self {}
    }
}

impl AudioEffect for WaveFolder {
    fn process_samples(&mut self, samples: (f32, f32), effect: &EffectParameters) -> (f32, f32) {
        if !effect.is_enabled {
            return samples;
        }

        if effect.parameters[0].is_sign_negative() {
            return samples;
        }

        let positive_amount = MAX_THRESHOLD - effect.parameters[0];
        let negative_amount = if effect.parameters[1].is_sign_positive() {
            MAX_THRESHOLD - effect.parameters[1]
        } else {
            positive_amount
        };

        fold(samples, positive_amount, negative_amount)
    }
}

fn fold(samples: (f32, f32), positive_amount: f32, mut negative_amount: f32) -> (f32, f32) {
    if negative_amount > 0.0 {
        negative_amount = -negative_amount;
    }
    (
        asymmetrical_fold_wave(
            samples.0,
            positive_amount.abs().min(1.0),
            negative_amount.max(-1.0),
        ),
        asymmetrical_fold_wave(
            samples.1,
            positive_amount.abs().min(1.0),
            negative_amount.max(-1.0),
        ),
    )
}

fn asymmetrical_fold_wave(sample: f32, positive_threshold: f32, negative_threshold: f32) -> f32 {
    if sample <= positive_threshold && sample >= negative_threshold {
        return sample;
    }

    if sample > positive_threshold {
        return positive_threshold - (sample - positive_threshold);
    }

    negative_threshold + (negative_threshold - sample)
}

#[cfg(test)]
mod tests {
    use super::*;
    use accsyn_types::math::f32s_are_equal;

    #[test]
    fn wavefolder_process_samples_returns_original_when_disabled() {
        let mut wavefolder = WaveFolder::new();
        let effect = EffectParameters {
            is_enabled: false,
            parameters: vec![0.5, 0.5, 0.0, 0.0],
        };
        let input = (0.8, -0.6);

        let result = wavefolder.process_samples(input, &effect);

        assert!(f32s_are_equal(result.0, 0.8));
        assert!(f32s_are_equal(result.1, -0.6));
    }

    #[test]
    fn wavefolder_process_samples_returns_original_when_parameter_negative() {
        let mut wavefolder = WaveFolder::new();
        let effect = EffectParameters {
            is_enabled: true,
            parameters: vec![-0.1, 0.5, 0.0, 0.0],
        };
        let input = (0.8, -0.6);

        let result = wavefolder.process_samples(input, &effect);

        assert!(f32s_are_equal(result.0, 0.8));
        assert!(f32s_are_equal(result.1, -0.6));
    }

    #[test]
    fn wavefolder_process_samples_uses_symmetrical_folding_when_parameter1_negative() {
        let mut wavefolder = WaveFolder::new();
        let effect = EffectParameters {
            is_enabled: true,
            parameters: vec![0.5, -1.0, 0.0, 0.0], // parameter[1] negative means symmetrical
        };
        let input = (0.8, -0.8);

        let result = wavefolder.process_samples(input, &effect);

        // positive_amount = MAX_THRESHOLD - 0.5 = 0.5
        // negative_amount = positive_amount = 0.5 (symmetrical)
        // So thresholds are 0.5 and -0.5
        // 0.8 > 0.5, so folded: 0.5 - (0.8 - 0.5) = 0.2
        // -0.8 < -0.5, so folded: -0.5 + (-0.5 - (-0.8)) = -0.5 + 0.3 = -0.2
        assert!(f32s_are_equal(result.0, 0.2));
        assert!(f32s_are_equal(result.1, -0.2));
    }

    #[test]
    fn wavefolder_process_samples_uses_asymmetrical_folding_when_parameter1_positive() {
        let mut wavefolder = WaveFolder::new();
        let effect = EffectParameters {
            is_enabled: true,
            parameters: vec![0.7, 0.3, 0.0, 0.0], // Different positive values for asymmetry
        };
        let input = (0.8, -0.9);

        let result = wavefolder.process_samples(input, &effect);

        // positive_amount = 1.0 - 0.7 = 0.3
        // negative_amount = 1.0 - 0.3 = 0.7 (asymmetrical)
        // So thresholds are 0.3 and -0.7
        // 0.8 > 0.3, so folded: 0.3 - (0.8 - 0.3) = -0.2
        // -0.9 < -0.7, so folded: -0.7 + (-0.7 - (-0.9)) = -0.7 + 0.2 = -0.5
        assert!(f32s_are_equal(result.0, -0.2));
        assert!(f32s_are_equal(result.1, -0.5));
    }

    #[test]
    fn asymmetrical_fold_wave_returns_sample_within_thresholds() {
        let sample = 0.3;
        let positive_threshold = 0.5;
        let negative_threshold = -0.5;

        let result = asymmetrical_fold_wave(sample, positive_threshold, negative_threshold);

        assert!(f32s_are_equal(result, 0.3));
    }

    #[test]
    fn asymmetrical_fold_wave_returns_sample_at_positive_threshold() {
        let sample = 0.5;
        let positive_threshold = 0.5;
        let negative_threshold = -0.5;

        let result = asymmetrical_fold_wave(sample, positive_threshold, negative_threshold);

        assert!(f32s_are_equal(result, 0.5));
    }

    #[test]
    fn asymmetrical_fold_wave_returns_sample_at_negative_threshold() {
        let sample = -0.5;
        let positive_threshold = 0.5;
        let negative_threshold = -0.5;

        let result = asymmetrical_fold_wave(sample, positive_threshold, negative_threshold);

        assert!(f32s_are_equal(result, -0.5));
    }

    #[test]
    fn asymmetrical_fold_wave_folds_down_when_above_positive_threshold() {
        let sample = 0.8;
        let positive_threshold = 0.5;
        let negative_threshold = -0.5;

        let result = asymmetrical_fold_wave(sample, positive_threshold, negative_threshold);

        // Folded: 0.5 - (0.8 - 0.5) = 0.2
        let expected = 0.2;
        assert!(f32s_are_equal(result, expected));
    }

    #[test]
    fn asymmetrical_fold_wave_folds_up_when_below_negative_threshold() {
        let sample = -0.9;
        let positive_threshold = 0.5;
        let negative_threshold = -0.5;

        let result = asymmetrical_fold_wave(sample, positive_threshold, negative_threshold);

        // Folded: -0.5 + (-0.5 - (-0.9)) = -0.5 + 0.4 = -0.1
        let expected = -0.1;
        assert!(f32s_are_equal(result, expected));
    }

    #[test]
    fn asymmetrical_fold_wave_handles_asymmetric_thresholds() {
        let sample = 0.7;
        let positive_threshold = 0.4;
        let negative_threshold = -0.8;

        let result = asymmetrical_fold_wave(sample, positive_threshold, negative_threshold);

        // Folded: 0.4 - (0.7 - 0.4) = 0.1
        let expected = 0.1;
        assert!(f32s_are_equal(result, expected));
    }
}
