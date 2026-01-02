use crate::math::load_f32_from_atomic_u32;
use crate::modules::effects::{AudioEffect, AudioEffectParameters, EffectParameters};
use crate::modules::effects::constants::MAX_WAVEFOLDER_THRESHOLD;

pub struct WaveFolder {}
impl WaveFolder {
    pub fn new() -> Self {
        Self {}
    }
}

impl AudioEffect for WaveFolder {
    fn process_samples(&self, samples: (f32, f32), effect: &EffectParameters) -> (f32, f32) {
        if !effect.is_enabled {
            return samples;
        }
        
        if effect.parameters[0].is_sign_negative() {
            return samples;
        }
        
        let positive_amount = MAX_WAVEFOLDER_THRESHOLD - effect.parameters[0];
        let negative_amount = if effect.parameters[1].is_sign_positive() {
            MAX_WAVEFOLDER_THRESHOLD - effect.parameters[1]
        } else {
            positive_amount
        };

        fold(samples, positive_amount, negative_amount)
    }
}

fn fold(
    samples: (f32, f32),
    positive_amount: f32,
    mut negative_amount: f32,
) -> (f32, f32) {
    if negative_amount > 0.0 {
        negative_amount = -negative_amount;
    }
    (
        asymmetrical_fold_wave(samples.0, positive_amount.abs().min(1.0), negative_amount.max(-1.0)),
        asymmetrical_fold_wave(samples.1, positive_amount.abs().min(1.0), negative_amount.max(-1.0)),
    )
}

fn asymmetrical_fold_wave(sample: f32, positive_threshold: f32, negative_threshold: f32) -> f32 {

    if sample <= positive_threshold && sample >= negative_threshold {
        return sample
    }

    if sample > positive_threshold {
        return positive_threshold - (sample - positive_threshold)
    }

    negative_threshold + (negative_threshold - sample)

}



