use super::WaveShape;
use crate::modules::oscillator::generate_wave_trait::GenerateWave;
use std::f32::consts::PI;

const SHAPE: WaveShape = WaveShape::Supersaw;
const DEFAULT_X_COORDINATE: f32 = 0.0;
const DEFAULT_X_INCREMENT: f32 = 1.0;
const VOICE_FREQUENCY_OFFSETS: [(f32, f32); 7] = [
    (0.893, 0.85),
    (0.939, 0.90),
    (0.98, 0.95),
    (1.0, 0.0),
    (1.02, 0.95),
    (1.064, 0.9),
    (1.11, 0.85),
];

/// Multi-voice detuned supersaw oscillator blending seven saw waves.
pub struct Supersaw {
    shape: WaveShape,
    x_coordinate: [f32; 7],
    x_increment: f32,
    sample_rate: u32,
    phase: Option<f32>,
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
            sample_rate,
            phase: None,
        }
    }

    fn single_saw_sample(&mut self, tone_frequency: f32, x_coordinate: f32) -> f32 {
        // Sample rate is always ≤ 192_000, within f32 precision (2²³ = 8_388_608)
        #[allow(clippy::cast_precision_loss)]
        let sample_rate_f32 = self.sample_rate as f32;
        let y_coordinate: f32 = (-2.0 / PI)
            * (1.0f32 / (tone_frequency * PI * (x_coordinate / sample_rate_f32)).tan()).atan();
        y_coordinate
    }
}

impl GenerateWave for Supersaw {
    fn next_sample(&mut self, tone_frequency: f32, modulation: Option<f32>) -> f32 {
        // Sample rate is always ≤ 192_000, within f32 precision (2²³ = 8_388_608)
        #[allow(clippy::cast_precision_loss)]
        let sample_rate_f32 = self.sample_rate as f32;

        let mut voice_samples: Vec<f32> = vec![];

        for (voice, (frequency_offset, level_offset)) in VOICE_FREQUENCY_OFFSETS.iter().enumerate()
        {
            let sample =
                self.single_saw_sample(tone_frequency * frequency_offset, self.x_coordinate[voice]);
            voice_samples.push(sample * level_offset);
            self.x_coordinate[voice] += self.x_increment * modulation.unwrap_or(1.0);
        }

        if tone_frequency > 0.0 {
            let period = sample_rate_f32 / tone_frequency;
            for voice in 0..7 {
                if self.x_coordinate[voice as usize] >= period {
                    self.x_coordinate[voice as usize] -= period;
                }
            }
        }

        voice_samples.iter().sum::<f32>() / 2.0
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
        self.x_coordinate = [DEFAULT_X_COORDINATE; 7];
        self.x_increment = DEFAULT_X_INCREMENT;
    }
}
