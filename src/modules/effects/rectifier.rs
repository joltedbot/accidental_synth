use crate::modules::effects::{AudioEffect, EffectParameters};
use crate::synthesizer::midi_value_converters::normal_value_to_bool;

pub struct Rectifier {}

impl Rectifier {
    pub fn new() -> Self {
        log::debug!("Constructing Rectifier Effect Module");

        Self {}
    }
}

impl AudioEffect for Rectifier {
    fn process_samples(&mut self, samples: (f32, f32), effect: &EffectParameters) -> (f32, f32) {
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::math::f32s_are_equal;

    #[test]
    fn rectifier_process_samples_returns_original_when_disabled() {
        let mut rectifier = Rectifier::new();
        let effect = EffectParameters {
            is_enabled: false,
            parameters: vec![0.0, 0.0, 0.0, 0.0],
        };
        let input = (0.5, -0.3);

        let result = rectifier.process_samples(input, &effect);

        assert!(f32s_are_equal(result.0, 0.5));
        assert!(f32s_are_equal(result.1, -0.3));
    }

    #[test]
    fn rectify_uses_full_wave_when_parameter_is_true() {
        let samples = (0.5, -0.3);
        let use_full_wave = true;

        let result = rectify(samples, use_full_wave);

        assert!(f32s_are_equal(result.0, 0.5));
        assert!(f32s_are_equal(result.1, 0.3));
    }

    #[test]
    fn rectify_uses_half_wave_when_parameter_is_false() {
        let samples = (0.5, -0.3);
        let use_full_wave = false;

        let result = rectify(samples, use_full_wave);

        assert!(f32s_are_equal(result.0, 0.5));
        assert!(f32s_are_equal(result.1, 0.0));
    }

    #[test]
    fn half_wave_rectifier_passes_positive_sample() {
        let positive_sample = 0.7;

        let result = half_wave_rectifier(positive_sample);

        assert!(f32s_are_equal(result, 0.7));
    }

    #[test]
    fn half_wave_rectifier_zeros_negative_sample() {
        let negative_sample = -0.4;

        let result = half_wave_rectifier(negative_sample);

        assert!(f32s_are_equal(result, 0.0));
    }

    #[test]
    fn half_wave_rectifier_passes_zero() {
        let zero_sample = 0.0;

        let result = half_wave_rectifier(zero_sample);

        assert!(f32s_are_equal(result, 0.0));
    }

    #[test]
    fn full_wave_rectifier_passes_positive_sample() {
        let positive_sample = 0.6;

        let result = full_wave_rectifier(positive_sample);

        assert!(f32s_are_equal(result, 0.6));
    }

    #[test]
    fn full_wave_rectifier_inverts_negative_sample() {
        let negative_sample = -0.8;

        let result = full_wave_rectifier(negative_sample);

        assert!(f32s_are_equal(result, 0.8));
    }

    #[test]
    fn full_wave_rectifier_passes_zero() {
        let zero_sample = 0.0;

        let result = full_wave_rectifier(zero_sample);

        assert!(f32s_are_equal(result, 0.0));
    }
}
