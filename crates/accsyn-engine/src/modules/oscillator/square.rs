use super::WaveShape;
use super::constants::{DEFAULT_X_COORDINATE, DEFAULT_X_INCREMENT, RADS_PER_CYCLE};
const SHAPE: WaveShape = WaveShape::Square;
use crate::modules::oscillator::generate_wave_trait::GenerateWave;

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
        let sample_rate_f64 = self.sample_rate as f64;
        let tone_frequency_f64 = tone_frequency as f64;

        if let Some(phase) = self.phase {
            self.x_coordinate = (phase / RADS_PER_CYCLE) * (sample_rate_f64 / tone_frequency_f64);
            self.phase = None;
        }

        let mut y_coordinate =
            (tone_frequency_f64 * RADS_PER_CYCLE * (self.x_coordinate / sample_rate_f64)).sin();

        if y_coordinate >= 0.0 {
            y_coordinate = 1.0;
        } else {
            y_coordinate = -1.0;
        }

        self.x_coordinate += DEFAULT_X_INCREMENT * modulation.unwrap_or(1.0) as f64;

        if tone_frequency_f64 > 0.0 {
            let period = sample_rate_f64 / tone_frequency_f64;
            if self.x_coordinate >= period {
                self.x_coordinate -= period;
            }
        }

        y_coordinate as f32
    }

    fn set_shape_parameter1(&mut self, _parameter: f32) {}

    fn set_shape_parameter2(&mut self, _parameter: f32) {}

    fn set_phase(&mut self, phase: f32) {
        self.phase = Some(phase as f64);
    }

    fn shape(&self) -> WaveShape {
        self.shape
    }

    fn reset(&mut self) {
        self.x_coordinate = DEFAULT_X_COORDINATE;
    }
}
