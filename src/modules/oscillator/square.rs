use super::WaveShape;

const SHAPE: WaveShape = WaveShape::Square;
use crate::modules::oscillator::generate_wave_trait::GenerateWave;
use crate::modules::oscillator::constants::RADS_PER_CYCLE;

const DEFAULT_X_COORDINATE: f32 = 0.0;
const DEFAULT_X_INCREMENT: f32 = 1.0;

pub struct Square {
    shape: WaveShape,
    x_coordinate: f32,
    sample_rate: u32,
    phase: Option<f32>,   
}

impl Square {
    pub fn new(sample_rate: u32) -> Self {
        log::info!("Constructing Square WaveShape Module");
        let x_coordinate = DEFAULT_X_COORDINATE;

        Self {
            shape: SHAPE,
            x_coordinate,
            sample_rate,
            phase: None,       
        }
    }
}

impl GenerateWave for Square {
    fn next_sample(&mut self, tone_frequency: f32, modulation: Option<f32>) -> f32 {
        let new_frequency = tone_frequency;

        if let Some(phase) = self.phase {
            self.x_coordinate = (phase / RADS_PER_CYCLE) * (self.sample_rate as f32 / new_frequency);
            self.phase = None;
        }
        
        let mut y_coordinate: f32 =
            (new_frequency * RADS_PER_CYCLE * (self.x_coordinate / self.sample_rate as f32)).sin();

        if y_coordinate >= 0.0 {
            y_coordinate = 1.0;
        } else {
            y_coordinate = -1.0;
        }

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
