use super::constants::{DEFAULT_X_COORDINATE, DEFAULT_X_INCREMENT};
use super::{WaveShape, poly_blep};
use crate::modules::oscillator::generate_wave_trait::GenerateWave;
use accsyn_core::casting::f64_to_f32_clamped;
use std::f64::consts::PI;

const SHAPE: WaveShape = WaveShape::Saw;

/// Sawtooth wave oscillator using band-limited synthesis.
pub struct Saw {
    shape: WaveShape,
    x_coordinate: f64,
    sample_rate: u32,
    phase: Option<f64>,
}

impl Saw {
    pub(crate) fn new(sample_rate: u32) -> Self {
        log::debug!(target: "synth::oscillator", shape = "Saw"; "Constructing wave generator");

        let x_coordinate = DEFAULT_X_COORDINATE;

        Self {
            shape: SHAPE,
            x_coordinate,
            sample_rate,
            phase: None,
        }
    }
}

impl GenerateWave for Saw {
    fn next_sample(&mut self, tone_frequency: f32, modulation: Option<f32>) -> f32 {
        let sample_rate_f64 = f64::from(self.sample_rate);
        let tone_frequency_f64 = f64::from(tone_frequency);
        let period = sample_rate_f64 / tone_frequency_f64;
        let new_x_increment = DEFAULT_X_INCREMENT * f64::from(modulation.unwrap_or(1.0));

        if let Some(phase) = self.phase {
            self.x_coordinate = (phase / PI) * period;
            self.phase = None;
        }

        let normalized_x_increment = new_x_increment / period;
        let normalized_x_coordinate = (self.x_coordinate / period).rem_euclid(1.0);
        let y_coordinate = (2.0 * normalized_x_coordinate - 1.0)
            - poly_blep(normalized_x_coordinate, normalized_x_increment);

        self.x_coordinate += new_x_increment;

        if tone_frequency > 0.0 && self.x_coordinate >= period {
            self.x_coordinate -= period;
        }

        f64_to_f32_clamped(y_coordinate)
    }

    fn set_shape_parameter1(&mut self, _parameters: f32) {}

    fn set_shape_parameter2(&mut self, _parameters: f32) {}

    fn set_phase(&mut self, phase: f32) {
        self.phase = Some(f64::from(phase));
    }

    fn shape(&self) -> WaveShape {
        self.shape
    }

    fn reset(&mut self) {
        self.x_coordinate = DEFAULT_X_COORDINATE;
    }
}
