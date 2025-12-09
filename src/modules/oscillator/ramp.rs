use super::WaveShape;
use super::constants::{DEFAULT_X_COORDINATE, DEFAULT_X_INCREMENT};
use crate::modules::oscillator::generate_wave_trait::GenerateWave;
use std::f32::consts::PI;

pub struct Ramp {
    shape: WaveShape,
    x_coordinate: f32,
    sample_rate: u32,
    phase: Option<f32>,
}

impl Ramp {
    pub fn new(sample_rate: u32) -> Self {
        log::debug!(target: "synth::oscillator", shape = "Ramp"; "Constructing wave generator");
        let x_coordinate = DEFAULT_X_COORDINATE;

        Self {
            shape: WaveShape::Ramp,
            x_coordinate,
            sample_rate,
            phase: None,
        }
    }
}

impl GenerateWave for Ramp {
    fn next_sample(&mut self, tone_frequency: f32, modulation: Option<f32>) -> f32 {
        if let Some(phase) = self.phase {
            self.x_coordinate = (phase / PI) * (self.sample_rate as f32 / tone_frequency);
            self.phase = None;
        }

        let y_coordinate: f32 = (2.0 / PI)
            * (1.0f32
                / (tone_frequency * PI * (self.x_coordinate / self.sample_rate as f32)).tan())
            .atan();

        self.x_coordinate += DEFAULT_X_INCREMENT * modulation.unwrap_or(1.0);
        y_coordinate
    }

    fn set_shape_parameter1(&mut self, _parameter: f32) {}

    fn set_shape_parameter2(&mut self, _parameter: f32) {}

    fn set_phase(&mut self, phase: f32) {
        self.phase = Some(phase);
    }

    fn shape(&self) -> WaveShape {
        self.shape
    }

    fn reset(&mut self) {
        self.x_coordinate = DEFAULT_X_COORDINATE;
    }
}
