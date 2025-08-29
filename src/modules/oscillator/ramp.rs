use super::constants::*;
use super::{GenerateSamples, WaveShape};
use std::f32::consts::PI;

pub struct Ramp {
    shape: WaveShape,
    x_coordinate: f32,
    sample_rate: u32,
}

impl Ramp {
    pub fn new(sample_rate: u32) -> Self {
        log::info!("Constructing Ramp WaveShape Module");
        let x_coordinate = DEFAULT_X_COORDINATE;

        Self {
            shape: WaveShape::Ramp,
            x_coordinate,
            sample_rate,
        }
    }
}

impl GenerateSamples for Ramp {
    fn next_sample(&mut self, tone_frequency: f32, modulation: Option<f32>) -> f32 {
        let new_frequency = tone_frequency * modulation.unwrap_or(1.0);

        let y_coordinate: f32 = (2.0 / PI)
            * (1.0f32 / (new_frequency * PI * (self.x_coordinate / self.sample_rate as f32)).tan())
                .atan();

        self.x_coordinate += DEFAULT_X_INCREMENT * modulation.unwrap_or(1.0);
        y_coordinate
    }

    fn set_shape_parameter1(&mut self, _parameter: f32) {}

    fn set_shape_parameter2(&mut self, _parameter: f32) {}

    fn set_phase(&mut self, _phase: f32) {}

    fn shape(&self) -> WaveShape {
        self.shape
    }

    fn reset(&mut self) {
        self.x_coordinate = DEFAULT_X_COORDINATE;
    }
}
