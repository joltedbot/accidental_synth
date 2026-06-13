use super::WaveShape;
use super::constants::{DEFAULT_X_COORDINATE, DEFAULT_X_INCREMENT};
use crate::modules::oscillator::generate_wave_trait::GenerateWave;
use std::f64::consts::PI;

const SHAPE: WaveShape = WaveShape::Supersaw;
const DEFAULT_DETUNE: f64 = 0.3;
const VOICE_FREQUENCY_OFFSETS: [(f64, f64); 7] = [
    (0.893, 0.85),
    (0.939, 0.90),
    (0.98, 0.95),
    (1.0, 1.0),
    (1.02, 0.95),
    (1.064, 0.9),
    (1.11, 0.85),
];

/// Multi-voice detuned supersaw oscillator blending seven saw waves.
pub struct Supersaw {
    shape: WaveShape,
    x_coordinate: [f64; 7],
    x_increment: f64,
    detune: f64,
    sample_rate: u32,
    phase: Option<f64>,
}

impl Supersaw {
    pub(crate) fn new(sample_rate: u32) -> Self {
        log::debug!(target: "synth::oscillator", shape = "Supersaw"; "Constructing wave generator");
        let x_coordinate = [1.0, 3.0, 11.0, 15.0, 13.0, 21.0, 35.0];
        let x_increment = DEFAULT_X_INCREMENT;

        Self {
            shape: SHAPE,
            x_coordinate,
            x_increment,
            detune: DEFAULT_DETUNE,
            sample_rate,
            phase: None,
        }
    }

    fn single_saw_sample(&mut self, tone_frequency: f64, x_coordinate: f64) -> f64 {
        let sample_rate_f64 = self.sample_rate as f64;
        let y_coordinate = (-2.0 / PI)
            * (1.0 / (tone_frequency * PI * (x_coordinate / sample_rate_f64)).tan()).atan();
        y_coordinate
    }
}

impl GenerateWave for Supersaw {
    fn next_sample(&mut self, tone_frequency: f32, modulation: Option<f32>) -> f32 {
        let sample_rate_f64 = self.sample_rate as f64;
        let tone_frequency_f64 = tone_frequency as f64;

        let mut voice_mix = 0.0;

        let sample_modulation = modulation.unwrap_or(1.0) as f64;

        for (voice, (frequency_offset, level_offset)) in VOICE_FREQUENCY_OFFSETS.iter().enumerate()
        {
            let voice_frequency =
                tone_frequency_f64 * (1.0 + self.detune * (frequency_offset - 1.0));

            let sample = self.single_saw_sample(voice_frequency, self.x_coordinate[voice]);
            self.x_coordinate[voice] += self.x_increment * sample_modulation;

            voice_mix += sample * level_offset;

            if tone_frequency_f64 > 0.0 {
                let period = sample_rate_f64 / voice_frequency;
                if self.x_coordinate[voice] >= period {
                    self.x_coordinate[voice] -= period;
                }
            }
        }

        voice_mix as f32 / 2.0
    }

    fn set_shape_parameter1(&mut self, parameter: f32) {
        self.detune = f64::from(parameter);
    }

    fn set_shape_parameter2(&mut self, _parameter: f32) {}

    fn set_phase(&mut self, phase: f32) {
        self.phase = Some(f64::from(phase));
    }

    fn shape(&self) -> WaveShape {
        self.shape
    }

    fn reset(&mut self) {
        self.x_coordinate = [DEFAULT_X_COORDINATE; 7];
        self.x_increment = DEFAULT_X_INCREMENT;
    }
}
