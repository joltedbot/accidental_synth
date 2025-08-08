use crate::modules::oscillator::GenerateSamples;

const PI: f32 = std::f32::consts::PI;
const DEFAULT_X_COORDINATE: f32 = 0.0;
const DEFAULT_X_INCREMENT: f32 = 1.0;

const DEFAULT_PULSE_WIDTH_ADJUSTMENT: f32 = 0.5;
const OSCILLATOR_MOD_TO_PWM_ADJUSTMENT_FACTOR: f32 = 0.5;

pub struct Pulse {
    x_coordinate: f32,
    sample_rate: u32,
    pulse_width: f32,
}

impl Pulse {
    pub fn new(sample_rate: u32) -> Self {
        log::info!("Constructing Pulse WaveShape Module");
        let x_coordinate = DEFAULT_X_COORDINATE;

        Self {
            x_coordinate,
            sample_rate,
            pulse_width: DEFAULT_PULSE_WIDTH_ADJUSTMENT,
        }
    }
}

impl GenerateSamples for Pulse {
    fn next_sample(&mut self, tone_frequency: f32, modulation: Option<f32>) -> f32 {
        let duty_cycle = if tone_frequency == 0.0 {
            0.0
        } else {
            match modulation {
                Some(modulation) => modulation - OSCILLATOR_MOD_TO_PWM_ADJUSTMENT_FACTOR,
                None => self.pulse_width,
            }
        };

        let mut y_coordinate: f32 =
            (tone_frequency * (2.0 * PI) * (self.x_coordinate / self.sample_rate as f32)).sin();

        if y_coordinate >= 0.0 - duty_cycle {
            y_coordinate = 1.0;
        } else {
            y_coordinate = -1.0;
        }

        self.x_coordinate += DEFAULT_X_INCREMENT;
        y_coordinate
    }

    fn set_shape_parameters(&mut self, parameters: Vec<f32>) {
        self.pulse_width = parameters[0];
    }

    fn reset(&mut self) {
        self.x_coordinate = DEFAULT_X_COORDINATE;
    }
}
