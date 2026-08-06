use crate::modules::effects::wet_dry_blend;
use crate::modules::lfo::Lfo;
use crate::modules::oscillator::WaveShape;
use crate::modules::phase_shift_all_pass::PhaseShiftAllPass;
use accsyn_core::effects::{AudioEffect, EffectParameters};
use accsyn_core::math::{
    EXPONENTIAL_PHASER_COEFFICIENT, exponential_curve_from_normal_value_and_coefficient,
};

const ALL_PASS_DRY_MIX_RATIO: f32 = 0.5;

pub struct Phaser {
    stages: [PhaseShiftAllPass; 4],
    lfo: Lfo,
}

impl Phaser {
    pub fn new(sample_rate: u32) -> Self {
        let stage1 = PhaseShiftAllPass::new(sample_rate);
        let stage2 = PhaseShiftAllPass::new(sample_rate);
        let stage3 = PhaseShiftAllPass::new(sample_rate);
        let stage4 = PhaseShiftAllPass::new(sample_rate);

        let mut lfo = Lfo::new(sample_rate);
        lfo.set_frequency(0.5);
        lfo.set_wave_shape(WaveShape::Triangle as u8);

        Self {
            stages: [stage1, stage2, stage3, stage4],
            lfo,
        }
    }
}

impl AudioEffect for Phaser {
    fn process_samples(&mut self, samples: (f32, f32), effect: &EffectParameters) -> (f32, f32) {
        if !effect.is_enabled {
            return samples;
        }

        let rate = calculate_rate(effect.parameters[0]);

        self.lfo.set_frequency(rate);
        let lfo_value = self.lfo.generate(None);

        let (mut left, mut right) = self.stages[0].process(samples, lfo_value);
        (left, right) = self.stages[1].process((left, right), lfo_value);
        (left, right) = self.stages[2].process((left, right), lfo_value);
        (left, right) = self.stages[3].process((left, right), lfo_value);

        let blend_amount = effect.parameters[1];
        let phased_samples = wet_dry_blend(samples, (left, right), ALL_PASS_DRY_MIX_RATIO);
        wet_dry_blend(samples, phased_samples, blend_amount)
    }
}

fn calculate_rate(normal_value: f32) -> f32 {
    exponential_curve_from_normal_value_and_coefficient(
        normal_value,
        EXPONENTIAL_PHASER_COEFFICIENT,
    ) / 100.0
}
