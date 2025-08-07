mod constants;
mod saw;
pub mod sine;

use self::saw::Saw;
use self::sine::Sine;

pub trait GenerateSamples {
    fn next_sample(&mut self, tone_frequency: f32, modulation: Option<f32>) -> f32;

    fn reset(&mut self);
}

#[derive(Default, Clone, Copy, PartialEq, Eq)]
pub enum WaveShape {
    #[default]
    Sine,
    Saw,
}

pub struct Oscillator {
    wave_shape: WaveShape,
    generator: Box<dyn GenerateSamples + Send + Sync>,
}

impl Oscillator {
    pub fn new(sample_rate: u32, wave_shape: WaveShape) -> Self {
        log::info!("Constructing Oscillator Module");
        let generator = get_wave_generator_from_wave_shape(sample_rate, wave_shape);

        Self {
            wave_shape,
            generator,
        }
    }

    pub fn next_sample(&mut self, tone_frequency: f32, modulation: Option<f32>) -> f32 {
        self.generator.next_sample(tone_frequency, modulation)
    }

    pub fn reset(&mut self) {
        self.generator.reset();
    }
}

fn get_wave_generator_from_wave_shape(
    sample_rate: u32,
    wave_shape: WaveShape,
) -> Box<dyn GenerateSamples + Send + Sync> {
    match wave_shape {
        WaveShape::Sine => Box::new(Sine::new(sample_rate)),
        WaveShape::Saw => Box::new(Saw::new(sample_rate)),
    }
}
