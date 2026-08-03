use super::constants::{
    DEFAULT_PULSE_WIDTH_ADJUSTMENT, DEFAULT_X_COORDINATE, DEFAULT_X_INCREMENT,
    OSCILLATOR_MOD_TO_PWM_ADJUSTMENT_FACTOR, RADS_PER_CYCLE,
};
use super::{WaveShape, poly_blep};
use crate::modules::oscillator::generate_wave_trait::GenerateWave;
use accsyn_core::casting::f64_to_f32_clamped;

const NORMALIZED_X_COORD_ZERO_POINT: f64 = 0.5;

/// Pulse wave oscillator with variable duty cycle (pulse width).
pub struct Pulse {
    shape: WaveShape,
    x_coordinate: f64,
    sample_rate: u32,
    width: f64,
    phase: Option<f64>,
}

impl Pulse {
    pub(crate) fn new(sample_rate: u32) -> Self {
        log::debug!(target: "synth::oscillator", shape = "Pulse"; "Constructing wave generator");
        let x_coordinate = DEFAULT_X_COORDINATE;

        Self {
            shape: WaveShape::Pulse,
            x_coordinate,
            sample_rate,
            width: DEFAULT_PULSE_WIDTH_ADJUSTMENT,
            phase: None,
        }
    }
}

impl GenerateWave for Pulse {
    fn next_sample(&mut self, tone_frequency: f32, modulation: Option<f32>) -> f32 {
        let sample_rate_f64 = f64::from(self.sample_rate);
        let tone_frequency_f64 = f64::from(tone_frequency);
        let period = sample_rate_f64 / tone_frequency_f64;
        let normalized_x_increment = DEFAULT_X_INCREMENT / period;

        let duty_cycle = match modulation {
            Some(modulation) => f64::from(modulation) - OSCILLATOR_MOD_TO_PWM_ADJUSTMENT_FACTOR,
            None => self.width,
        };

        if let Some(phase) = self.phase {
            self.x_coordinate = (phase / RADS_PER_CYCLE) * period;
            self.phase = None;
        }

        let normalized_x_coordinate = (self.x_coordinate / period).rem_euclid(1.0);

        let mut y_coordinate = if normalized_x_coordinate
            < (NORMALIZED_X_COORD_ZERO_POINT + NORMALIZED_X_COORD_ZERO_POINT * duty_cycle)
        {
            1.0
        } else {
            -1.0
        };

        y_coordinate += poly_blep(normalized_x_coordinate, normalized_x_increment);
        y_coordinate -= poly_blep(
            (normalized_x_coordinate
                - (NORMALIZED_X_COORD_ZERO_POINT + NORMALIZED_X_COORD_ZERO_POINT * duty_cycle))
                .rem_euclid(1.0),
            normalized_x_increment,
        );

        self.x_coordinate += DEFAULT_X_INCREMENT;

        if tone_frequency_f64 > 0.0 && self.x_coordinate >= period {
            self.x_coordinate -= period;
        }

        f64_to_f32_clamped(y_coordinate)
    }

    fn set_shape_parameter1(&mut self, parameter: f32) {
        self.width = f64::from(parameter);
    }

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
