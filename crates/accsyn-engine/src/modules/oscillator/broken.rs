use super::WaveShape;
use super::constants::{DEFAULT_X_COORDINATE, DEFAULT_X_INCREMENT};
use crate::modules::oscillator::generate_wave_trait::GenerateWave;
use std::f64::consts::PI;

const SHAPE: WaveShape = WaveShape::Broken;
const MIN_JANK: f64 = 0.0;
const MAX_JANK: f64 = 0.99;

/// Some kind of broken wave generator using a saw and then some jank
pub struct Broken {
    shape: WaveShape,
    x_coordinate: f64,
    sample_rate: u32,
    phase: Option<f64>,
    jank_factor: f64,
    jank_amount: f64,
}

impl Broken {
    pub(crate) fn new(sample_rate: u32) -> Self {
        log::debug!(target: "synth::oscillator", shape = "Broken"; "Constructing wave generator");

        let x_coordinate = DEFAULT_X_COORDINATE;

        Self {
            shape: SHAPE,
            x_coordinate,
            sample_rate,
            phase: None,
            jank_factor: 1.0,
            jank_amount: 1.0,
        }
    }
}

impl GenerateWave for Broken {
    fn next_sample(&mut self, tone_frequency: f32, modulation: Option<f32>) -> f32 {
        let sample_rate_f64 = self.sample_rate as f64;
        let tone_frequency_f64 = tone_frequency as f64;

        if let Some(phase) = self.phase {
            self.x_coordinate = (phase / PI) * (sample_rate_f64 / tone_frequency_f64);
            self.phase = None;
        }

        let y_coordinate = (-2.0 / PI)
            * self.jank_factor
            * (1.0 / (tone_frequency_f64 * PI * (self.x_coordinate / sample_rate_f64)).tan())
                .atan();

        if y_coordinate > self.jank_amount || y_coordinate < -self.jank_amount {
            self.jank_factor = -self.jank_factor;
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

    fn set_shape_parameter1(&mut self, parameter: f32) {
        let internal_jank = f64::from(parameter);
        self.jank_amount = internal_jank.clamp(MIN_JANK, MAX_JANK);
    }

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
