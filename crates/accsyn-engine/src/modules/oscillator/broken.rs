use super::WaveShape;
use crate::modules::oscillator::generate_wave_trait::GenerateWave;
use std::f32::consts::PI;

const SHAPE: WaveShape = WaveShape::Broken;
const DEFAULT_X_COORDINATE: f32 = 0.0;
const DEFAULT_X_INCREMENT: f32 = 1.0;
const MIN_JANK: f32 = 0.0;
const MAX_JANK: f32 = 0.99;

/// Some kind of broken wave generator using a saw and then some jank
pub struct Broken {
    shape: WaveShape,
    x_coordinate: f32,
    sample_rate: u32,
    phase: Option<f32>,
    jank_factor: f32,
    jank_amount: f32,
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
        // Sample rate is always ≤ 192_000, within f32 precision (2²³ = 8_388_608)
        #[allow(clippy::cast_precision_loss)]
        let sample_rate_f32 = self.sample_rate as f32;

        if let Some(phase) = self.phase {
            self.x_coordinate = (phase / PI) * (sample_rate_f32 / tone_frequency);
            self.phase = None;
        }

        let y_coordinate: f32 = (-2.0 / PI)
            * self.jank_factor
            * (1.0f32 / (tone_frequency * PI * (self.x_coordinate / sample_rate_f32)).tan()).atan();

        if y_coordinate > self.jank_amount || y_coordinate < -self.jank_amount {
            self.jank_factor = -self.jank_factor;
        }

        self.x_coordinate += DEFAULT_X_INCREMENT * modulation.unwrap_or(1.0);

        if tone_frequency > 0.0 {
            let period = sample_rate_f32 / tone_frequency;
            if self.x_coordinate >= period {
                self.x_coordinate -= period;
            }
        }

        y_coordinate
    }

    fn set_shape_parameter1(&mut self, parameter: f32) {
        self.jank_amount = parameter.clamp(MIN_JANK, MAX_JANK);
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
