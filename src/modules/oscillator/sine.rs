use super::WaveShape;
use super::constants::{DEFAULT_PHASE, MAX_PHASE, MIN_PHASE, RADS_PER_CYCLE};
use crate::modules::oscillator::generate_wave_trait::GenerateWave;
use std::f32::consts::PI;

pub struct Sine {
    shape: WaveShape,
    phase: f32,
    phase_increment: f32,
}

impl Sine {
    pub fn new(sample_rate: u32) -> Self {
        log::debug!(target: "synth::oscillator", shape = "Sine"; "Constructing wave generator");
        let phase: f32 = DEFAULT_PHASE;
        let seconds_per_sample = 1.0 / sample_rate as f32;
        let phase_increment = RADS_PER_CYCLE * seconds_per_sample;

        Self {
            shape: WaveShape::Sine,
            phase,
            phase_increment,
        }
    }
}
impl GenerateWave for Sine {
    fn next_sample(&mut self, tone_frequency: f32, modulation: Option<f32>) -> f32 {
        let new_frequency = tone_frequency * modulation.unwrap_or(1.0);
        self.phase += self.phase_increment * new_frequency;

        if self.phase >= RADS_PER_CYCLE {
            self.reset();
        }

        self.phase.sin()
    }

    fn set_shape_parameter1(&mut self, _parameters: f32) {}
    fn set_shape_parameter2(&mut self, _parameter: f32) {}

    fn set_phase(&mut self, phase: f32) {
        self.phase = (2.0 * PI) * phase.clamp(MIN_PHASE, MAX_PHASE);
    }

    fn shape(&self) -> WaveShape {
        self.shape
    }

    fn reset(&mut self) {
        self.phase = DEFAULT_PHASE;
    }
}
