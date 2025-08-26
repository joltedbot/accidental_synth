use super::{GenerateSamples, WaveShape};

const SHAPE: WaveShape = WaveShape::GigaSaw;
use std::f32::consts::PI;

const DEFAULT_X_COORDINATE: f32 = 0.0;
const DEFAULT_X_INCREMENT: f32 = 1.0;
const VOICE_FREQUENCY_SPREAD_CENTS: [i8; 7] = [-12, -7, -4, 0, 4, 7, 12];
const VOICE_COUNT_OUTPUT_LEVEL_OFFSET: f32 = 0.3;

pub struct GigaSaw {
    shape: WaveShape,
    x_coordinate: f32,
    x_increment: f32,
    sample_rate: u32,
}

impl GigaSaw {
    pub fn new(sample_rate: u32) -> Self {
        log::info!("Constructing GigaSaw WaveShape Module");
        let x_coordinate = DEFAULT_X_COORDINATE;
        let x_increment = DEFAULT_X_INCREMENT;

        Self {
            shape: SHAPE,
            x_coordinate,
            x_increment,
            sample_rate,
        }
    }
}

impl GenerateSamples for GigaSaw {
    fn next_sample(&mut self, tone_frequency: f32, modulation: Option<f32>) -> f32 {
        let mut voice_samples: Vec<f32> = vec![];

        for frequency_offset in VOICE_FREQUENCY_SPREAD_CENTS {
            voice_samples.push(self.single_saw_sample(
                frequency_from_cents(tone_frequency, frequency_offset),
                self.x_coordinate,
                modulation,
            ));
        }

        self.x_coordinate += self.x_increment;

        voice_samples.iter().sum::<f32>() * VOICE_COUNT_OUTPUT_LEVEL_OFFSET
    }

    fn set_shape_parameter1(&mut self, _parameter: f32) {}

    fn set_shape_parameter2(&mut self, _parameter: f32) {}

    fn set_phase(&mut self, _phase: f32) {}

    fn shape(&self) -> WaveShape {
        self.shape
    }

    fn reset(&mut self) {
        self.x_coordinate = DEFAULT_X_COORDINATE;
        self.x_increment = DEFAULT_X_INCREMENT;
    }
}

impl GigaSaw {
    fn single_saw_sample(
        &mut self,
        tone_frequency: f32,
        x_coordinate: f32,
        modulation: Option<f32>,
    ) -> f32 {
        let new_frequency = tone_frequency * modulation.unwrap_or(1.0);

        let y_coordinate: f32 = (-2.0 / PI)
            * (1.0f32 / (new_frequency * PI * (x_coordinate / self.sample_rate as f32)).tan())
                .atan();
        y_coordinate
    }
}

fn frequency_from_cents(frequency: f32, cents: i8) -> f32 {
    frequency * (2.0f32.powf(cents as f32 / 1200.0))
}
