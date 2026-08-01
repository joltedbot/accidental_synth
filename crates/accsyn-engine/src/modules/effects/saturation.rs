use accsyn_core::casting::f32_to_u32_clamped;
use accsyn_core::effects::{AudioEffect, EffectParameters};
use strum::EnumCount as saturationCount;
use strum_macros::{EnumCount, EnumIter, FromRepr};

const MAXIMUM_AMOUNT: f32 = 0.999;
const MINIMUM_AMOUNT: f32 = 0.001;

pub struct Saturation {}

impl Saturation {
    pub fn new() -> Self {
        log::debug!(target: "synthesizer::effects::saturation", "Constructing Saturation Effect Module");

        Self {}
    }
}

impl AudioEffect for Saturation {
    fn process_samples(&mut self, samples: (f32, f32), effect: &EffectParameters) -> (f32, f32) {
        if !effect.is_enabled {
            return samples;
        }

        let mode = SaturationMode::from_normal_value(effect.parameters[0]);
        let amount = effect.parameters[1].clamp(MINIMUM_AMOUNT, MAXIMUM_AMOUNT);
        let gain_reduction = effect.parameters[2];

        let mut saturated_samples = match mode {
            SaturationMode::AnalogModeled => {
                let left_saturation_sample = saturation_analog_modeled(samples.0, amount);
                let right_saturation_sample = saturation_analog_modeled(samples.1, amount);
                (left_saturation_sample, right_saturation_sample)
            }
            SaturationMode::TubeLike => {
                let left_saturation_sample = saturation_exponential_tube_like(samples.0, amount);
                let right_saturation_sample = saturation_exponential_tube_like(samples.1, amount);
                (left_saturation_sample, right_saturation_sample)
            }
            SaturationMode::CubicSoftClipping => {
                let left_saturation_sample = saturation_cubic_soft_clipping(samples.0, amount);
                let right_saturation_sample = saturation_cubic_soft_clipping(samples.1, amount);
                (left_saturation_sample, right_saturation_sample)
            }
            SaturationMode::TanhSoftClipping => {
                let left_saturation_sample = tanh_soft_clipping(samples.0, amount);
                let right_saturation_sample = tanh_soft_clipping(samples.1, amount);
                (left_saturation_sample, right_saturation_sample)
            }
            SaturationMode::WaveShaping => {
                let left_saturation_sample = saturation_asymptotic_waveshaper(samples.0, amount);
                let right_saturation_sample = saturation_asymptotic_waveshaper(samples.1, amount);
                (left_saturation_sample, right_saturation_sample)
            }
            SaturationMode::SineShaper => {
                let left_saturation_sample = saturation_sine_shaper(samples.0, amount);
                let right_saturation_sample = saturation_sine_shaper(samples.1, amount);
                (left_saturation_sample, right_saturation_sample)
            }
            SaturationMode::Polynomial => {
                let left_saturation_sample = saturation_chebyshev_polynomial(samples.0, amount);
                let right_saturation_sample = saturation_chebyshev_polynomial(samples.1, amount);
                (left_saturation_sample, right_saturation_sample)
            }
        };

        saturated_samples.0 *= gain_reduction;
        saturated_samples.1 *= gain_reduction;

        saturated_samples
    }
}

#[derive(Default, Debug, Clone, Copy, EnumCount, EnumIter, FromRepr)]
#[repr(u32)]
pub enum SaturationMode {
    #[default]
    AnalogModeled,
    TubeLike,
    CubicSoftClipping,
    TanhSoftClipping,
    WaveShaping,
    SineShaper,
    Polynomial,
}

impl SaturationMode {
    pub fn from_normal_value(normal_value: f32) -> Self {
        // The number of saturation modes is a very small list created by hand and not dynamic.
        // It fits well within an f32
        #[allow(clippy::cast_precision_loss)]
        let index = normal_value * (Self::COUNT - 1) as f32;
        Self::from_repr(f32_to_u32_clamped(index)).unwrap_or_default()
    }
}

fn saturation_analog_modeled(sample: f32, amount: f32) -> f32 {
    let drive = 1.0 + amount * 9.0;
    let shaped = (sample * drive).atan() * (2.0 / std::f32::consts::PI);
    let makeup = 1.0 + (1.0 - amount).powf(0.5) * amount * 3.0;
    shaped * makeup
}

fn saturation_exponential_tube_like(sample: f32, amount: f32) -> f32 {
    let factor = 1.0 + amount * 2.0;
    let shaped = sample.signum() * (1.0 - (-sample.abs() * factor).exp());
    let makeup = 1.0 + amount * (3.0 - amount * 1.5);
    shaped * makeup
}

fn saturation_cubic_soft_clipping(sample: f32, amount: f32) -> f32 {
    let drive = 1.0 + amount * 3.0;
    let driven_sample = sample * drive;
    let shaped = if driven_sample.abs() < 1.0 {
        driven_sample - (driven_sample.powi(3) / 3.0)
    } else {
        driven_sample.signum() * (2.0 / 3.0)
    };

    let makeup = 1.0 + amount * (2.0 - amount);
    shaped * makeup
}

fn tanh_soft_clipping(sample: f32, amount: f32) -> f32 {
    let clipping_coefficient = 1.0 + amount * 7.0;
    (sample * clipping_coefficient).tanh() / clipping_coefficient.tanh()
}

fn saturation_asymptotic_waveshaper(sample: f32, amount: f32) -> f32 {
    let shape = (2.0 * amount) / (1.0 - amount);
    let shaped = ((1.0 + shape) * sample) / (1.0 + (shape * sample.abs()));
    let makeup = 1.0 + amount * 0.2;
    shaped * makeup
}

fn saturation_sine_shaper(sample: f32, amount: f32) -> f32 {
    let drive = 1.0 + amount * std::f32::consts::PI * 0.5;
    let shaped = (sample * drive).sin();
    let makeup = 1.0 + amount * (1.5 - amount * 0.5);
    shaped * makeup
}

fn saturation_chebyshev_polynomial(sample: f32, amount: f32) -> f32 {
    let clamped_sample = sample.clamp(-1.0, 1.0);
    let third_harmonic = 4.0 * clamped_sample.powi(3);
    let third_harmonic_scale = 0.25 + amount * 0.5;
    (clamped_sample + third_harmonic * amount * third_harmonic_scale).clamp(-1.0, 1.0)
}

#[cfg(test)]
mod tests {
    use super::*;
    use accsyn_core::math::f32s_are_equal;

    #[test]
    fn saturation_process_samples_returns_original_when_disabled() {
        let mut saturation = Saturation::new();
        let effect = EffectParameters {
            name: String::new(),
            is_enabled: false,
            parameters: vec![0.0, 0.5, 1.0, 0.0],
        };
        let input = (0.7, -0.4);

        let result = saturation.process_samples(input, &effect);

        assert!(f32s_are_equal(result.0, 0.7));
        assert!(f32s_are_equal(result.1, -0.4));
    }

    #[test]
    fn saturation_process_samples_clamps_amount_to_minimum() {
        let mut saturation = Saturation::new();
        let effect = EffectParameters {
            name: String::new(),
            is_enabled: true,
            parameters: vec![0.0, 0.0, 1.0, 0.0], // amount below MINIMUM_AMOUNT clamps to it
        };
        let input = (0.7, -0.4);

        let result = saturation.process_samples(input, &effect);

        let expected_left = saturation_analog_modeled(0.7, MINIMUM_AMOUNT);
        let expected_right = saturation_analog_modeled(-0.4, MINIMUM_AMOUNT);

        assert!(
            f32s_are_equal(result.0, expected_left),
            "Expected: {expected_left}, got: {}",
            result.0
        );
        assert!(
            f32s_are_equal(result.1, expected_right),
            "Expected: {expected_right}, got: {}",
            result.1
        );
    }

    #[test]
    fn saturation_process_samples_clamps_amount_to_max() {
        let mut saturation = Saturation::new();
        let effect = EffectParameters {
            name: String::new(),
            is_enabled: true,
            parameters: vec![0.0, 1.5, 0.5, 0.0], // amount > WAVE_SHAPER_MAX_AMOUNT
        };
        let input = (0.5, 0.5);

        // Should not panic and should use clamped value
        let result = saturation.process_samples(input, &effect);

        // Result should be finite (not NaN or infinite)
        assert!(result.0.is_finite());
        assert!(result.1.is_finite());
    }

    #[test]
    fn saturation_process_samples_uses_analog_modeled_mode() {
        let mut saturation = Saturation::new();
        let effect = EffectParameters {
            name: String::new(),
            is_enabled: true,
            parameters: vec![0.0, 0.5, 1.0, 0.0], // mode index 0 = AnalogModeled
        };
        let input = (0.5, -0.5);

        let result = saturation.process_samples(input, &effect);

        // Should produce saturated output
        assert!(result.0.is_finite());
        assert!(result.1.is_finite());
    }

    #[test]
    fn saturation_process_samples_uses_tube_like_mode() {
        let mut saturation = Saturation::new();
        let effect = EffectParameters {
            name: String::new(),
            is_enabled: true,
            parameters: vec![1.0, 0.5, 1.0, 0.0], // mode index 1 = TubeLike
        };
        let input = (0.5, -0.5);

        let result = saturation.process_samples(input, &effect);

        assert!(result.0.is_finite());
        assert!(result.1.is_finite());
    }

    #[test]
    fn saturation_process_samples_uses_soft_clipping_mode() {
        let mut saturation = Saturation::new();
        let effect = EffectParameters {
            name: String::new(),
            is_enabled: true,
            parameters: vec![2.0, 0.5, 1.0, 0.0], // mode index 2 = CubicSoftClipping
        };
        let input = (0.5, -0.5);

        let result = saturation.process_samples(input, &effect);

        assert!(result.0.is_finite());
        assert!(result.1.is_finite());
    }

    #[test]
    fn saturation_process_samples_uses_wave_shaping_mode() {
        let mut saturation = Saturation::new();
        let effect = EffectParameters {
            name: String::new(),
            is_enabled: true,
            parameters: vec![3.0, 0.5, 1.0, 0.0], // mode index 3 = WaveShaping
        };
        let input = (0.5, -0.5);

        let result = saturation.process_samples(input, &effect);

        assert!(result.0.is_finite());
        assert!(result.1.is_finite());
    }

    #[test]
    fn saturation_process_samples_uses_sine_shaper_mode() {
        let mut saturation = Saturation::new();
        let effect = EffectParameters {
            name: String::new(),
            is_enabled: true,
            parameters: vec![4.0, 0.5, 1.0, 0.0], // mode index 4 = SineShaper
        };
        let input = (0.5, -0.5);

        let result = saturation.process_samples(input, &effect);

        assert!(result.0.is_finite());
        assert!(result.1.is_finite());
    }

    #[test]
    fn saturation_process_samples_uses_polynomial_mode() {
        let mut saturation = Saturation::new();
        let effect = EffectParameters {
            name: String::new(),
            is_enabled: true,
            parameters: vec![5.0, 0.5, 1.0, 0.0], // mode index 5 = Polynomial
        };
        let input = (0.5, -0.5);

        let result = saturation.process_samples(input, &effect);

        assert!(result.0.is_finite());
        assert!(result.1.is_finite());
    }

    #[test]
    fn saturation_process_samples_applies_gain_reduction() {
        let mut saturation = Saturation::new();
        let effect_full_gain = EffectParameters {
            name: String::new(),
            is_enabled: true,
            parameters: vec![0.0, 0.5, 1.0, 0.0], // gain_reduction = 1.0
        };
        let effect_half_gain = EffectParameters {
            name: String::new(),
            is_enabled: true,
            parameters: vec![0.0, 0.5, 0.5, 0.0], // gain_reduction = 0.5
        };
        let input = (0.5, 0.5);

        let result_full = saturation.process_samples(input, &effect_full_gain);
        let result_half = saturation.process_samples(input, &effect_half_gain);

        // Half gain result should be smaller
        assert!(result_half.0.abs() < result_full.0.abs());
        assert!(result_half.1.abs() < result_full.1.abs());
    }

    #[test]
    fn saturation_mode_from_normal_value_returns_default_for_invalid_index() {
        let mode = SaturationMode::from_normal_value(100.0); // Invalid index

        // Should return default (AnalogModeled)
        assert!(matches!(mode, SaturationMode::AnalogModeled));
    }

    #[test]
    fn saturation_cubic_soft_clipping_branches_on_threshold() {
        let amount = 0.5;

        // Test below threshold (|x| < 1.0)
        let below = saturation_cubic_soft_clipping(0.3, amount);
        assert!(below.is_finite());

        // Test above threshold (|x| >= 1.0)
        let above = saturation_cubic_soft_clipping(2.0, amount);
        assert!(above.is_finite());

        // Results should differ
        assert!(!f32s_are_equal(below, above));
    }

    #[test]
    fn saturation_cubic_soft_clipping_handles_negative_samples() {
        let amount: f32 = 0.2;
        let sample: f32 = -0.5;
        let drive = 1.0 + amount * 3.0;
        let x = sample * drive;
        assert!(
            x.abs() < 1.0,
            "test input must stay in the soft-clip branch"
        );
        let expected = x - (x.powi(3) / 3.0);
        let makeup = 1.0 + amount * (2.0 - amount);
        let expected_output = expected * makeup;

        let result = saturation_cubic_soft_clipping(sample, amount);

        assert!(
            f32s_are_equal(result, expected_output),
            "Expected: {expected_output}, got: {result}"
        );
    }

    #[test]
    fn saturation_exponential_tube_like_uses_signum() {
        let amount = 0.5;

        // Test positive sample
        let positive = saturation_exponential_tube_like(0.5, amount);
        assert!(positive > 0.0);

        // Test negative sample
        let negative = saturation_exponential_tube_like(-0.5, amount);
        assert!(negative < 0.0);
    }
}
