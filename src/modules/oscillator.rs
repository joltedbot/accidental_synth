mod constants;

pub mod am;
pub mod fm;
pub mod noise;
pub mod pulse;
pub mod ramp;
pub mod saw;
pub mod sine;
pub mod square;
pub mod super_saw;
pub mod triangle;

use self::saw::Saw;
use self::sine::Sine;

pub trait GenerateSamples {
    fn next_sample(&mut self, tone_frequency: f32, modulation: Option<f32>) -> f32;

    fn set_shape_parameters(&mut self, parameters: Vec<f32>);

    fn reset(&mut self);
}

#[derive(Default, Debug, Clone, Copy, PartialEq, Eq)]
pub enum WaveShape {
    AM,
    FM,
    Noise,
    Pulse,
    Ramp,
    Saw,
    #[default]
    Sine,
    Square,
    SuperSaw,
    Triangle,
}

pub struct Oscillator {
    sample_rate: u32,
    wave_generator: Box<dyn GenerateSamples + Send + Sync>,
}

impl Oscillator {
    pub fn new(sample_rate: u32, wave_shape: WaveShape) -> Self {
        log::info!("Constructing Oscillator Module");
        let wave_generator = get_wave_generator_from_wave_shape(sample_rate, wave_shape);

        Self {
            sample_rate,
            wave_generator,
        }
    }

    pub fn next_sample(&mut self, tone_frequency: f32, modulation: Option<f32>) -> f32 {
        self.wave_generator.next_sample(tone_frequency, modulation)
    }

    pub fn set_wave_shape(&mut self, wave_shape: WaveShape) {
        log::info!("Setting Oscillator Shape to {wave_shape:?}");
        self.wave_generator = get_wave_generator_from_wave_shape(self.sample_rate, wave_shape)
    }

    pub fn set_shape_parameters(&mut self, parameters: Vec<f32>) {
        self.wave_generator.set_shape_parameters(parameters);
    }

    pub fn reset(&mut self) {
        self.wave_generator.reset();
    }
}

fn get_wave_generator_from_wave_shape(
    sample_rate: u32,
    wave_shape: WaveShape,
) -> Box<dyn GenerateSamples + Send + Sync> {
    match wave_shape {
        WaveShape::AM => Box::new(am::AM::new(sample_rate)),
        WaveShape::FM => Box::new(fm::FM::new(sample_rate)),
        WaveShape::Noise => Box::new(noise::Noise::new()),
        WaveShape::Pulse => Box::new(pulse::Pulse::new(sample_rate)),
        WaveShape::Ramp => Box::new(ramp::Ramp::new(sample_rate)),
        WaveShape::Saw => Box::new(Saw::new(sample_rate)),
        WaveShape::Sine => Box::new(Sine::new(sample_rate)),
        WaveShape::Square => Box::new(square::Square::new(sample_rate)),
        WaveShape::SuperSaw => Box::new(super_saw::SuperSaw::new(sample_rate)),
        WaveShape::Triangle => Box::new(triangle::Triangle::new(sample_rate)),
    }
}
