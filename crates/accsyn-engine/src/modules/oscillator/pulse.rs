use super::WaveShape;
use super::constants::{
    DEFAULT_PULSE_WIDTH_ADJUSTMENT, DEFAULT_X_COORDINATE, DEFAULT_X_INCREMENT,
    OSCILLATOR_MOD_TO_PWM_ADJUSTMENT_FACTOR, RADS_PER_CYCLE,
};
use crate::modules::oscillator::generate_wave_trait::GenerateWave;

/// Pulse wave oscillator with variable duty cycle (pulse width).
pub struct Pulse {
    shape: WaveShape,
    x_coordinate: f32,
    sample_rate: u32,
    width: f32,
    phase: Option<f32>,
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
        // Sample rate is always ≤ 192_000, within f32 precision (2²³ = 8_388_608)
        #[allow(clippy::cast_precision_loss)]
        let sample_rate_f32 = self.sample_rate as f32;

        let duty_cycle = match modulation {
            Some(modulation) => modulation - OSCILLATOR_MOD_TO_PWM_ADJUSTMENT_FACTOR,
            None => self.width,
        };

        if let Some(phase) = self.phase {
            self.x_coordinate =
                (phase / RADS_PER_CYCLE) * (sample_rate_f32 / tone_frequency);
            self.phase = None;
        }

        let mut y_coordinate: f32 =
            (tone_frequency * RADS_PER_CYCLE * (self.x_coordinate / sample_rate_f32)).sin();

        if y_coordinate >= 0.0 + duty_cycle {
            y_coordinate = 1.0;
        } else {
            y_coordinate = -1.0;
        }

        self.x_coordinate += DEFAULT_X_INCREMENT;

        if tone_frequency > 0.0 {
            let period = sample_rate_f32 / tone_frequency;
            if self.x_coordinate >= period {
                self.x_coordinate -= period;
            }
        }

        y_coordinate
    }

    fn set_shape_parameter1(&mut self, parameter: f32) {
        self.width = parameter;
    }

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
