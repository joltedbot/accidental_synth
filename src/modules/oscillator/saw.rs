use super::constants::*;
use super::{GenerateSamples, WaveShape};
use std::f32::consts::PI;

const SHAPE: WaveShape = WaveShape::Saw;
const DEFAULT_X_COORDINATE: f32 = 0.0;
const DEFAULT_X_INCREMENT: f32 = 1.0;

pub struct Saw {
    shape: WaveShape,
    x_coordinate: f32,
    sample_rate: u32,
    phase: f32,
}

impl Saw {
    pub fn new(sample_rate: u32) -> Self {
        log::info!("Constructing Saw WaveShape Module");

        let x_coordinate = DEFAULT_X_COORDINATE;

        Self {
            shape: SHAPE,
            x_coordinate,
            sample_rate,
            phase: DEFAULT_PHASE,
        }
    }
}

impl GenerateSamples for Saw {
    fn next_sample(&mut self, tone_frequency: f32, modulation: Option<f32>) -> f32 {
        let y_coordinate: f32 = (-2.0 / PI)
            * self.phase.clamp(MIN_PHASE, MAX_PHASE)
            * (1.0f32
                / (tone_frequency * PI * (self.x_coordinate / self.sample_rate as f32)).tan())
            .atan();

        self.x_coordinate += DEFAULT_X_INCREMENT * modulation.unwrap_or(1.0);
        y_coordinate
    }

    fn set_shape_parameter1(&mut self, _parameters: f32) {}

    fn set_shape_parameter2(&mut self, _parameters: f32) {}

    fn set_phase(&mut self, phase: f32) {
        self.phase = phase;
    }

    fn shape(&self) -> WaveShape {
        self.shape
    }

    fn reset(&mut self) {
        self.x_coordinate = DEFAULT_X_COORDINATE;
        self.phase = DEFAULT_PHASE;
    }
}
