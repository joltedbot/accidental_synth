use crate::modules::effects::{AudioEffect, EffectParameters};
use strum_macros::{EnumCount, EnumIter, FromRepr};

const WAVE_SHAPER_MAX_AMOUNT: f32 = 0.99;

pub struct Saturation {}

impl Saturation {
    pub fn new() -> Self {
        Self {}
    }
}

impl AudioEffect for Saturation {
    fn process_samples(&self, samples: (f32, f32), effect: &EffectParameters) -> (f32, f32) {
        let mode_index = effect.parameters[0];
        let mode = SaturationMode::from_f32(mode_index);
        let mut amount = effect.parameters[1];
        let gain_reduction = effect.parameters[2];

        if !effect.is_enabled || amount == 0.0 {
            return samples;
        }

        if amount > WAVE_SHAPER_MAX_AMOUNT {
            amount = WAVE_SHAPER_MAX_AMOUNT;
        }

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
            SaturationMode::SoftClipping => {
                let left_saturation_sample = saturation_cubic_soft_clipping(samples.0, amount);
                let right_saturation_sample = saturation_cubic_soft_clipping(samples.1, amount);
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
    SoftClipping,
    WaveShaping,
    SineShaper,
    Polynomial,
}

impl SaturationMode {
    pub fn from_f32(index: f32) -> Self {
        Self::from_repr(index.trunc() as u32).unwrap_or_default()
    }
}

fn saturation_analog_modeled(sample: f32, amount: f32) -> f32 {
    let drive = 1.0 + amount * 9.0;
    let shaped = (sample * drive).atan() * (2.0 / std::f32::consts::PI);
    let makeup = 1.0 + (1.0 - amount).powf(0.5) * amount * 3.0;
    shaped * makeup
}

fn saturation_exponential_tube_like(sample: f32, amount: f32) -> f32 {
    let factor = amount * 2.0;
    let shaped = sample.signum() * (1.0 - (-sample.abs() * factor).exp());
    let makeup = 1.0 + amount * (3.0 - amount * 1.5);
    shaped * makeup
}

fn saturation_cubic_soft_clipping(sample: f32, amount: f32) -> f32 {
    let drive = amount * 3.0;
    let x = sample * drive;
    let shaped = if x.abs() < 1.0 {
        x - (x.powi(3) / 3.0)
    } else {
        x.signum() * (2.0 / 3.0)
    };

    let makeup = 1.0 + amount * (2.0 - amount);
    shaped * makeup
}

fn saturation_asymptotic_waveshaper(sample: f32, amount: f32) -> f32 {
    let shape = (2.0 * amount) / (1.0 - amount);
    let shaped = ((1.0 + shape) * sample) / (1.0 + (shape * sample.abs()));
    let makeup = 1.0 + amount * 0.2;
    shaped * makeup
}

fn saturation_sine_shaper(sample: f32, amount: f32) -> f32 {
    let drive = amount * std::f32::consts::PI * 0.5;
    let shaped = (sample * drive).sin();
    let makeup = 1.0 + amount * (1.5 - amount * 0.5);
    shaped * makeup
}

fn saturation_chebyshev_polynomial(sample: f32, amount: f32) -> f32 {
    let x = sample.clamp(-1.0, 1.0);
    let t3 = 4.0 * x.powi(3) - 3.0 * x;
    let t3_scale = 0.25 + amount * 0.5;
    x * (1.0 - amount) + t3 * amount * t3_scale
}
