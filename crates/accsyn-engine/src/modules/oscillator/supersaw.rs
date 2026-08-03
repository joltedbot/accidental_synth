use super::constants::{DEFAULT_X_COORDINATE, DEFAULT_X_INCREMENT};
use super::{WaveShape, poly_blep};
use crate::modules::oscillator::generate_wave_trait::GenerateWave;
use accsyn_core::casting::f64_to_f32_clamped;

const SHAPE: WaveShape = WaveShape::Supersaw;
const DEFAULT_DETUNE: f64 = 0.1;
const DEFAULT_BLEND: f64 = 1.0;

const VOICE_FREQUENCY_OFFSETS: [(f64, f64); 6] = [
    (0.893, 0.85),
    (0.939, 0.90),
    (0.98, 0.95),
    (1.02, 0.95),
    (1.064, 0.9),
    (1.11, 0.85),
];

/// Multi-voice detuned supersaw oscillator blending seven saw waves.
pub struct Supersaw {
    shape: WaveShape,
    center_x_coordinate: f64,
    x_coordinates: [f64; 6],
    detune: f64,
    blend: f64,
    sample_rate: u32,
}

impl Supersaw {
    pub(crate) fn new(sample_rate: u32) -> Self {
        log::debug!(target: "synth::oscillator", shape = "Supersaw"; "Constructing wave generator");
        let center_x_coordinate = DEFAULT_X_COORDINATE;
        let x_coordinates = [3.0, 11.0, 15.0, 13.0, 21.0, 35.0];

        Self {
            shape: SHAPE,
            center_x_coordinate,
            x_coordinates,
            detune: DEFAULT_DETUNE,
            blend: DEFAULT_BLEND,
            sample_rate,
        }
    }
}

impl GenerateWave for Supersaw {
    fn next_sample(&mut self, tone_frequency: f32, modulation: Option<f32>) -> f32 {
        let sample_rate_f64 = f64::from(self.sample_rate);
        let tone_frequency_f64 = f64::from(tone_frequency);
        let new_x_increment = DEFAULT_X_INCREMENT * f64::from(modulation.unwrap_or(1.0));

        let center_period = sample_rate_f64 / tone_frequency_f64;
        let normalized_center_x_increment = new_x_increment / center_period;
        let normalized_center_x_coordinate =
            (self.center_x_coordinate / center_period).rem_euclid(1.0);
        let mut voice_mix = single_saw_sample(normalized_center_x_coordinate)
            - poly_blep(
                normalized_center_x_coordinate,
                normalized_center_x_increment,
            );
        self.center_x_coordinate += new_x_increment;

        if tone_frequency_f64 > 0.0 && self.center_x_coordinate >= center_period {
            self.center_x_coordinate -= center_period;
        }

        for (voice, (frequency_offset, level_offset)) in VOICE_FREQUENCY_OFFSETS.iter().enumerate()
        {
            let voice_frequency =
                tone_frequency_f64 * (1.0 + self.detune * (frequency_offset - 1.0));

            let voice_period = sample_rate_f64 / voice_frequency;
            let normalized_voice_x_increment = new_x_increment / voice_period;
            let normalized_voice_x_coordinate =
                (self.x_coordinates[voice] / voice_period).rem_euclid(1.0);
            let sample = single_saw_sample(normalized_voice_x_coordinate)
                - poly_blep(normalized_voice_x_coordinate, normalized_voice_x_increment);
            self.x_coordinates[voice] += new_x_increment;

            voice_mix += sample * level_offset * self.blend;

            if tone_frequency_f64 > 0.0 && self.x_coordinates[voice] >= voice_period {
                self.x_coordinates[voice] -= voice_period;
            }
        }

        f64_to_f32_clamped(voice_mix)
    }

    fn set_shape_parameter1(&mut self, parameter: f32) {
        self.detune = f64::from(parameter);
    }

    fn set_shape_parameter2(&mut self, parameter: f32) {
        self.blend = f64::from(parameter);
    }

    fn set_phase(&mut self, _phase: f32) {}

    fn shape(&self) -> WaveShape {
        self.shape
    }

    fn reset(&mut self) {
        self.x_coordinates = [DEFAULT_X_COORDINATE; 6];
        self.center_x_coordinate = DEFAULT_X_COORDINATE;
    }
}

fn single_saw_sample(normalized_x_coordinate: f64) -> f64 {
    2.0 * normalized_x_coordinate - 1.0
}
