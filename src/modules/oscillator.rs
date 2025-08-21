pub mod am;
mod constants;
pub mod fm;
pub mod noise;
pub mod pulse;
pub mod ramp;
pub mod saw;
pub mod sine;
pub mod square;
pub mod super_saw;
pub mod triangle;

use self::am::AM;
use self::fm::FM;
use self::noise::Noise;
use self::pulse::Pulse;
use self::ramp::Ramp;
use self::saw::Saw;
use self::sine::Sine;
use self::square::Square;
use self::super_saw::SuperSaw;
use self::triangle::Triangle;

pub trait GenerateSamples {
    fn next_sample(&mut self, tone_frequency: f32, modulation: Option<f32>) -> f32;

    fn set_shape_parameter1(&mut self, parameter: f32);
    fn set_shape_parameter2(&mut self, parameter: f32);

    fn set_phase(&mut self, phase: f32);

    fn shape(&self) -> WaveShape;

    fn reset(&mut self);
}

#[derive(Default, Debug, Clone, Copy, PartialEq, Eq)]
pub enum WaveShape {
    #[default]
    Sine,
    Triangle,
    Square,
    Saw,
    Pulse,
    Ramp,
    SuperSaw,
    AM,
    FM,
    Noise,
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

    pub fn generate(&mut self, tone_frequency: f32, modulation: Option<f32>) -> f32 {
        self.wave_generator.next_sample(tone_frequency, modulation)
    }

    pub fn set_wave_shape(&mut self, wave_shape: WaveShape) {
        if wave_shape == self.wave_generator.shape() {
            return;
        }

        log::info!("Setting Oscillator Shape to {wave_shape:?}");
        self.wave_generator = get_wave_generator_from_wave_shape(self.sample_rate, wave_shape)
    }

    pub fn set_shape_parameter1(&mut self, parameter: f32) {
        self.wave_generator.set_shape_parameter1(parameter);
    }

    pub fn set_shape_parameter2(&mut self, parameter: f32) {
        self.wave_generator.set_shape_parameter2(parameter);
    }

    pub fn set_phase(&mut self, phase: f32) {
        self.wave_generator.set_phase(phase);
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
        WaveShape::Sine => Box::new(Sine::new(sample_rate)),
        WaveShape::Triangle => Box::new(Triangle::new(sample_rate)),
        WaveShape::Square => Box::new(Square::new(sample_rate)),
        WaveShape::Saw => Box::new(Saw::new(sample_rate)),
        WaveShape::Pulse => Box::new(Pulse::new(sample_rate)),
        WaveShape::Ramp => Box::new(Ramp::new(sample_rate)),
        WaveShape::SuperSaw => Box::new(SuperSaw::new(sample_rate)),
        WaveShape::AM => Box::new(AM::new(sample_rate)),
        WaveShape::FM => Box::new(FM::new(sample_rate)),
        WaveShape::Noise => Box::new(Noise::new()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn f32_value_equality(value_1: f32, value_2: f32) -> bool {
        (value_1 - value_2).abs() <= f32::EPSILON
    }

    #[test]
    fn new_returns_oscillator_with_correct_default_values() {
        let sample_rate = 44100;
        let wave_shape = WaveShape::Sine;
        let oscillator = Oscillator::new(sample_rate, wave_shape);
        assert_eq!(oscillator.sample_rate, sample_rate);
        assert_eq!(oscillator.wave_generator.shape(), wave_shape);
    }

    #[test]
    fn set_shape_parameters_correctly_sets_the_oscillators_shape_specific_parameters() {
        let sample_rate = 44100;
        let wave_shape = WaveShape::FM;
        let mut oscillator = Oscillator::new(sample_rate, wave_shape);

        let first_value = oscillator.generate(100.0, None);
        for _ in 0..5 {
            assert!(!f32_value_equality(
                oscillator.generate(100.0, None),
                first_value
            ));
        }
        let first_sample = oscillator.generate(100.0, None);

        oscillator.reset();
        oscillator.set_shape_parameter1(1.0);
        oscillator.set_shape_parameter2(2.0);

        let first_value = oscillator.generate(100.0, None);
        for _ in 0..5 {
            assert!(!f32_value_equality(
                oscillator.generate(100.0, None),
                first_value
            ));
        }
        let second_sample = oscillator.generate(100.0, None);

        assert_ne!(first_sample, second_sample);
    }

    #[test]
    fn reset_correctly_resets_oscillator_phase() {
        let sample_rate = 44100;
        let wave_shape = WaveShape::Sine;
        let mut oscillator = Oscillator::new(sample_rate, wave_shape);
        let first_value = oscillator.generate(100.0, None);
        for _ in 0..5 {
            assert!(!f32_value_equality(
                oscillator.generate(100.0, None),
                first_value
            ));
        }
        oscillator.reset();
        assert!(f32_value_equality(
            oscillator.generate(100.0, None),
            first_value
        ));
    }

    #[test]
    fn set_wave_shape_returns_oscillator_with_correct_wave_shape() {
        let sample_rate = 44100;
        let wave_shape = WaveShape::Sine;
        let mut oscillator = Oscillator::new(sample_rate, wave_shape);
        oscillator.set_wave_shape(WaveShape::Square);
        assert_eq!(oscillator.wave_generator.shape(), WaveShape::Square);
    }

    #[test]
    fn get_wave_generator_from_wave_shape_returns_correct_wave_generator() {
        let sample_rate = 44100;

        assert_eq!(
            get_wave_generator_from_wave_shape(sample_rate, WaveShape::AM).shape(),
            WaveShape::AM
        );
        assert_eq!(
            get_wave_generator_from_wave_shape(sample_rate, WaveShape::FM).shape(),
            WaveShape::FM
        );
        assert_eq!(
            get_wave_generator_from_wave_shape(sample_rate, WaveShape::Noise).shape(),
            WaveShape::Noise
        );
        assert_eq!(
            get_wave_generator_from_wave_shape(sample_rate, WaveShape::Pulse).shape(),
            WaveShape::Pulse
        );
        assert_eq!(
            get_wave_generator_from_wave_shape(sample_rate, WaveShape::Ramp).shape(),
            WaveShape::Ramp
        );
        assert_eq!(
            get_wave_generator_from_wave_shape(sample_rate, WaveShape::Saw).shape(),
            WaveShape::Saw
        );
        assert_eq!(
            get_wave_generator_from_wave_shape(sample_rate, WaveShape::Sine).shape(),
            WaveShape::Sine
        );
        assert_eq!(
            get_wave_generator_from_wave_shape(sample_rate, WaveShape::Square).shape(),
            WaveShape::Square
        );
        assert_eq!(
            get_wave_generator_from_wave_shape(sample_rate, WaveShape::SuperSaw).shape(),
            WaveShape::SuperSaw
        );
        assert_eq!(
            get_wave_generator_from_wave_shape(sample_rate, WaveShape::Triangle).shape(),
            WaveShape::Triangle
        );
    }
}
