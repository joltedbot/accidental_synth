use super::WaveShape;
use crate::modules::oscillator::generate_wave_trait::GenerateWave;
use accsyn_types::math::frequency_from_cents;
use std::f32::consts::PI;

const SHAPE: WaveShape = WaveShape::Supersaw;
const DEFAULT_X_COORDINATE: f32 = 0.0;
const DEFAULT_X_INCREMENT: f32 = 1.0;
const VOICE_FREQUENCY_SPREAD_CENTS: [i8; 7] = [-12, -7, -4, 0, 4, 7, 12];
const VOICE_COUNT_OUTPUT_LEVEL_OFFSET: f32 = 0.3;

pub struct Supersaw {
    shape: WaveShape,
    x_coordinate: f32,
    x_increment: f32,
    sample_rate: u32,
    phase: Option<f32>,
}

impl Supersaw {
    pub fn new(sample_rate: u32) -> Self {
        log::debug!(target: "synth::oscillator", shape = "Supersaw"; "Constructing wave generator");
        let x_coordinate = DEFAULT_X_COORDINATE;
        let x_increment = DEFAULT_X_INCREMENT;

        Self {
            shape: SHAPE,
            x_coordinate,
            x_increment,
            sample_rate,
            phase: None,
        }
    }

    fn single_saw_sample(&mut self, tone_frequency: f32, x_coordinate: f32) -> f32 {
        let y_coordinate: f32 = (-2.0 / PI)
            * (1.0f32 / (tone_frequency * PI * (x_coordinate / self.sample_rate as f32)).tan())
                .atan();
        y_coordinate
    }
}

impl GenerateWave for Supersaw {
    fn next_sample(&mut self, tone_frequency: f32, modulation: Option<f32>) -> f32 {
        let mut voice_samples: Vec<f32> = vec![];

        if let Some(phase) = self.phase {
            self.x_coordinate = (phase / PI) * (self.sample_rate as f32 / tone_frequency);
            self.phase = None;
        }

        for frequency_offset in VOICE_FREQUENCY_SPREAD_CENTS {
            voice_samples.push(self.single_saw_sample(
                frequency_from_cents(tone_frequency, i16::from(frequency_offset)),
                self.x_coordinate,
            ));
        }

        self.x_coordinate += self.x_increment * modulation.unwrap_or(1.0);

        voice_samples.iter().sum::<f32>() * VOICE_COUNT_OUTPUT_LEVEL_OFFSET
    }

    fn set_shape_parameter1(&mut self, _parameter: f32) {}

    fn set_shape_parameter2(&mut self, _parameter: f32) {}

    fn set_phase(&mut self, phase: f32) {
        self.phase = Some(phase);
    }

    fn shape(&self) -> WaveShape {
        self.shape
    }

    fn reset(&mut self) {
        self.x_coordinate = DEFAULT_X_COORDINATE;
        self.x_increment = DEFAULT_X_INCREMENT;
    }
}
