use crate::modules::oscillator::GenerateSamples;

const PI: f32 = std::f32::consts::PI;
const DEFAULT_X_COORDINATE: f32 = 0.0;
const DEFAULT_X_INCREMENT: f32 = 1.0;

pub struct Saw {
    x_coordinate: f32,
    sample_rate: u32,
}

impl Saw {
    pub fn new(sample_rate: u32) -> Self {
        log::info!("Constructing Saw WaveShape Module");

        let x_coordinate = DEFAULT_X_COORDINATE;

        Self {
            x_coordinate,
            sample_rate,
        }
    }
}

impl GenerateSamples for Saw {
    fn next_sample(&mut self, tone_frequency: f32, modulation: Option<f32>) -> f32 {
        let new_frequency = tone_frequency * modulation.unwrap_or(1.0);
        let y_coordinate: f32 = (-2.0 / PI)
            * (1.0f32 / (new_frequency * PI * (self.x_coordinate / self.sample_rate as f32)).tan())
                .atan();

        self.x_coordinate += DEFAULT_X_INCREMENT;
        y_coordinate
    }

    fn reset(&mut self) {
        self.x_coordinate = DEFAULT_X_COORDINATE;
    }
}
