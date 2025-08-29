use super::constants::*;
use super::{GenerateSamples, WaveShape};
use std::f32::consts::PI;

pub struct Pulse {
    shape: WaveShape,
    x_coordinate: f32,
    sample_rate: u32,
    pulse_width: f32,
}

impl Pulse {
    pub fn new(sample_rate: u32) -> Self {
        log::info!("Constructing Pulse WaveShape Module");
        let x_coordinate = DEFAULT_X_COORDINATE;

        Self {
            shape: WaveShape::Pulse,
            x_coordinate,
            sample_rate,
            pulse_width: DEFAULT_PULSE_WIDTH_ADJUSTMENT,
        }
    }
}

impl GenerateSamples for Pulse {
    fn next_sample(&mut self, tone_frequency: f32, modulation: Option<f32>) -> f32 {
        if tone_frequency == 0.0 {
            return 0.0;
        }

        let duty_cycle = match modulation {
            Some(modulation) => modulation - OSCILLATOR_MOD_TO_PWM_ADJUSTMENT_FACTOR,
            None => self.pulse_width,
        };

        let mut y_coordinate: f32 =
            (tone_frequency * (2.0 * PI) * (self.x_coordinate / self.sample_rate as f32)).sin();

        if y_coordinate >= 0.0 + duty_cycle {
            y_coordinate = 1.0;
        } else {
            y_coordinate = -1.0;
        }

        self.x_coordinate += DEFAULT_X_INCREMENT;
        y_coordinate
    }

    fn set_shape_parameter1(&mut self, parameter: f32) {
        self.pulse_width = parameter;
    }

    fn set_shape_parameter2(&mut self, _parameter: f32) {}

    fn set_phase(&mut self, _phase: f32) {}

    fn shape(&self) -> WaveShape {
        self.shape
    }

    fn reset(&mut self) {
        self.x_coordinate = DEFAULT_X_COORDINATE;
    }
}
