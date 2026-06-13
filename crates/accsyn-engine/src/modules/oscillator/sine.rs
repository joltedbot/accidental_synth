use super::WaveShape;
use super::constants::{DEFAULT_PHASE, MAX_PHASE, MIN_PHASE, RADS_PER_CYCLE};
use crate::modules::oscillator::generate_wave_trait::GenerateWave;

/// Sine wave oscillator using phase accumulation.
pub struct Sine {
    shape: WaveShape,
    phase: f64,
    phase_coefficient: f64,
}

impl Sine {
    pub(crate) fn new(sample_rate: u32) -> Self {
        log::debug!(target: "synth::oscillator", shape = "Sine"; "Constructing wave generator");
        let phase = DEFAULT_PHASE;
        let seconds_per_sample = 1.0 / sample_rate as f64;
        let phase_increment = RADS_PER_CYCLE * seconds_per_sample;

        Self {
            shape: WaveShape::Sine,
            phase,
            phase_coefficient: phase_increment,
        }
    }
}
impl GenerateWave for Sine {
    fn next_sample(&mut self, tone_frequency: f32, modulation: Option<f32>) -> f32 {
        let new_frequency = tone_frequency * modulation.unwrap_or(1.0);
        self.phase += self.phase_coefficient * new_frequency as f64;

        if self.phase >= RADS_PER_CYCLE {
            self.phase -= RADS_PER_CYCLE;
        }

        self.phase.sin() as f32
    }

    fn set_shape_parameter1(&mut self, _parameters: f32) {}
    fn set_shape_parameter2(&mut self, _parameter: f32) {}

    fn set_phase(&mut self, phase: f32) {
        let internal_phase = phase as f64;
        self.phase = RADS_PER_CYCLE * internal_phase.clamp(MIN_PHASE, MAX_PHASE);
    }

    fn shape(&self) -> WaveShape {
        self.shape
    }

    fn reset(&mut self) {
        self.phase = DEFAULT_PHASE;
    }
}
