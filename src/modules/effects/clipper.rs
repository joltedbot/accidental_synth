use crate::defaults::MAX_SAMPLE_VALUE;
use crate::modules::effects::{AudioEffect, EffectParameters};

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
            clip_signal(
                samples.0,
                effect.parameters[0],
                effect.parameters[1],
                effect.parameters[2],
            ),
            clip_signal(
                samples.1,
                effect.parameters[0],
                effect.parameters[1],
                effect.parameters[2],
            ),
        )
    }
}

fn clip_signal(signal: f32, mut threshold: f32, pre_gain: f32, post_gain: f32) -> f32 {
    threshold = threshold.min(MAX_SAMPLE_VALUE);

    let mut boosted_signal = signal * (1.0 + pre_gain.abs());

    if boosted_signal.abs() > threshold {
        boosted_signal = threshold * signal.signum();
    }

    (boosted_signal * (1.0 + post_gain.abs())).clamp(-MAX_SAMPLE_VALUE, MAX_SAMPLE_VALUE)
}
