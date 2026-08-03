use super::constants::{DEFAULT_X_COORDINATE, DEFAULT_X_INCREMENT, RADS_PER_CYCLE};
use super::{WaveShape, poly_blep};
use accsyn_core::casting::f64_to_f32_clamped;

const SHAPE: WaveShape = WaveShape::Square;
use crate::modules::oscillator::generate_wave_trait::GenerateWave;

const NORMALIZED_X_COORD_ZERO_POINT: f64 = 0.5;

/// Square wave oscillator producing +1/-1 output.
pub struct Square {
    shape: WaveShape,
    x_coordinate: f64,
    sample_rate: u32,
    phase: Option<f64>,
}

impl Square {
    pub(crate) fn new(sample_rate: u32) -> Self {
        log::debug!(target: "synth::oscillator", shape = "Square"; "Constructing wave generator");
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
        let sample_rate_f64 = f64::from(self.sample_rate);
        let tone_frequency_f64 = f64::from(tone_frequency);
        let period = sample_rate_f64 / tone_frequency_f64;
        let new_x_increment = DEFAULT_X_INCREMENT * f64::from(modulation.unwrap_or(1.0));
        let normalized_x_increment = new_x_increment / period;

        if let Some(phase) = self.phase {
            self.x_coordinate = (phase / RADS_PER_CYCLE) * period;
            self.phase = None;
        }

        let normalized_x_coordinate = (self.x_coordinate / period).rem_euclid(1.0);
        let mut y_coordinate = if normalized_x_coordinate < NORMALIZED_X_COORD_ZERO_POINT {
            1.0
        } else {
            -1.0
        };

        y_coordinate += poly_blep(normalized_x_coordinate, normalized_x_increment);
        y_coordinate -= poly_blep(
            (normalized_x_coordinate - NORMALIZED_X_COORD_ZERO_POINT).rem_euclid(1.0),
            normalized_x_increment,
        );

        self.x_coordinate += new_x_increment;

        if tone_frequency_f64 > 0.0 && self.x_coordinate >= period {
            self.x_coordinate -= period;
        }

        f64_to_f32_clamped(y_coordinate)
    }

    fn set_shape_parameter1(&mut self, _parameter: f32) {}

    fn set_shape_parameter2(&mut self, _parameter: f32) {}

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
