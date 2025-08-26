pub mod am;
mod constants;
pub mod fm;
pub mod gigasaw;
pub mod noise;
pub mod pulse;
pub mod ramp;
pub mod saw;
pub mod sine;
pub mod square;
pub mod triangle;

use self::am::AM;
use self::constants::*;
use self::fm::FM;
use self::gigasaw::GigaSaw;
use self::noise::Noise;
use self::pulse::Pulse;
use self::ramp::Ramp;
use self::saw::Saw;
use self::sine::Sine;
use self::square::Square;
use self::triangle::Triangle;
use crate::modules::lfo::load_f32_from_atomic_u32;
use std::sync::atomic::Ordering::Relaxed;
use std::sync::atomic::{AtomicBool, AtomicI8, AtomicI16, AtomicU8, AtomicU32};

pub trait GenerateSamples {
    fn next_sample(&mut self, tone_frequency: f32, modulation: Option<f32>) -> f32;

    fn set_shape_parameter1(&mut self, parameter: f32);
    fn set_shape_parameter2(&mut self, parameter: f32);

    fn set_phase(&mut self, phase: f32);

    fn shape(&self) -> WaveShape;

    fn reset(&mut self);
}

pub const NUMBER_OF_WAVE_SHAPES: u8 = 10;

#[derive(Default, Debug, Clone, Copy, PartialEq, Eq)]
pub enum WaveShape {
    #[default]
    Sine = 1,
    Triangle,
    Square,
    Saw,
    Pulse,
    Ramp,
    GigaSaw,
    AM,
    FM,
    Noise,
}

impl WaveShape {
    pub fn from_index(index: u8) -> Self {
        match index {
            1 => WaveShape::Sine,
            2 => WaveShape::Triangle,
            3 => WaveShape::Square,
            4 => WaveShape::Saw,
            5 => WaveShape::Pulse,
            6 => WaveShape::Ramp,
            7 => WaveShape::GigaSaw,
            8 => WaveShape::AM,
            9 => WaveShape::FM,
            10 => WaveShape::Noise,
            _ => Default::default(),
        }
    }
}

#[derive(Debug)]
pub struct OscillatorParameters {
    pub fine_tune: AtomicI16,
    pub course_tune: AtomicI8,
    pub pitch_bend: AtomicI16,
    pub shape_parameter1: AtomicU32,
    pub shape_parameter2: AtomicU32,
    pub wave_shape_index: AtomicU8,
    pub gate_flag: AtomicBool,
    pub key_sync_enabled: AtomicBool,
}

impl Default for OscillatorParameters {
    fn default() -> Self {
        Self {
            fine_tune: AtomicI16::new(0),
            course_tune: AtomicI8::new(0),
            pitch_bend: AtomicI16::new(0),
            shape_parameter1: AtomicU32::new(0),
            shape_parameter2: AtomicU32::new(0),
            wave_shape_index: AtomicU8::new(WaveShape::default() as u8),
            gate_flag: AtomicBool::new(false),
            key_sync_enabled: AtomicBool::new(DEFAULT_KEY_SYNC_ENABLED),
        }
    }
}

pub struct Oscillator {
    sample_rate: u32,
    wave_generator: Box<dyn GenerateSamples + Send + Sync>,
    tone_frequency: f32,
    wave_shape_index: u8,
    pitch_bend: i16,
    course_tune: i8,
    fine_tune: i16,
    is_sub_oscillator: bool,
    key_sync_enabled: bool,
}

impl Oscillator {
    pub fn new(sample_rate: u32, wave_shape: WaveShape) -> Self {
        log::info!("Constructing Oscillator Module");
        let wave_generator = get_wave_generator_from_wave_shape(sample_rate, wave_shape);

        Self {
            sample_rate,
            wave_generator,
            tone_frequency: 0.0,
            wave_shape_index: WaveShape::default() as u8,
            pitch_bend: 0,
            course_tune: 0,
            fine_tune: 0,
            is_sub_oscillator: false,
            key_sync_enabled: DEFAULT_KEY_SYNC_ENABLED,
        }
    }

    pub fn set_parameters(&mut self, parameters: &OscillatorParameters) {
        self.set_shape_parameter1(load_f32_from_atomic_u32(&parameters.shape_parameter1));
        self.set_shape_parameter2(load_f32_from_atomic_u32(&parameters.shape_parameter2));
        self.set_wave_shape_index(parameters.wave_shape_index.load(Relaxed));
        self.set_pitch_bend(parameters.pitch_bend.load(Relaxed));
        self.set_course_tune(parameters.course_tune.load(Relaxed));
        self.set_fine_tune(parameters.fine_tune.load(Relaxed));
        self.set_gate(&parameters.gate_flag);
        self.set_key_sync_enabled(parameters.key_sync_enabled.load(Relaxed));
    }

    pub fn generate(&mut self, modulation: Option<f32>) -> f32 {
        self.wave_generator
            .next_sample(self.tone_frequency, modulation)
    }

    pub fn set_wave_shape(&mut self, wave_shape: WaveShape) {
        if wave_shape == self.wave_generator.shape() {
            return;
        }

        log::info!("Setting Oscillator Shape to {:#?}", wave_shape);
        self.wave_generator = get_wave_generator_from_wave_shape(self.sample_rate, wave_shape)
    }

    pub fn set_wave_shape_index(&mut self, wave_shape_index: u8) {
        if wave_shape_index != self.wave_shape_index {
            let wave_shape = WaveShape::from_index(wave_shape_index);
            self.set_wave_shape(wave_shape);
            self.wave_shape_index = wave_shape_index;
        }
    }

    pub fn set_frequency(&mut self, tone_frequency: f32) {
        self.tone_frequency = tone_frequency;
    }

    pub fn set_phase(&mut self, phase: f32) {
        self.wave_generator.set_phase(phase);
    }

    pub fn reset(&mut self) {
        self.wave_generator.reset();
    }

    pub fn tune(&mut self, mut note_number: u8) {
        if self.course_tune != 0 {
            note_number = (note_number as i8)
                .saturating_add(self.course_tune)
                .clamp(MIN_MIDI_NOTE_NUMBER, MAX_MIDI_NOTE_NUMBER) as u8;
        }

        if self.is_sub_oscillator {
            note_number = (note_number as i8)
                .saturating_sub(12)
                .clamp(MIN_MIDI_NOTE_NUMBER, MAX_MIDI_NOTE_NUMBER) as u8;
        }

        let mut note_frequency = midi_note_to_frequency(note_number);

        if self.pitch_bend != 0 {
            note_frequency = frequency_from_cents(note_frequency, self.pitch_bend)
                .clamp(MIN_NOTE_FREQUENCY, MAX_NOTE_FREQUENCY);
        }

        if self.fine_tune != 0 {
            note_frequency = frequency_from_cents(note_frequency, self.fine_tune);
        }
        self.tone_frequency = note_frequency
    }

    fn set_gate(&mut self, gate_flag: &AtomicBool) {
        let gate_on = gate_flag.load(Relaxed);
        if gate_on && self.key_sync_enabled {
            self.reset();
            gate_flag.store(false, Relaxed); // Reset the gate flag
        }
    }

    fn set_key_sync_enabled(&mut self, key_sync_enabled: bool) {
        if key_sync_enabled != self.key_sync_enabled {
            println!("Key Sync Enabled: {}", self.key_sync_enabled);
        }
        self.key_sync_enabled = key_sync_enabled;
    }

    fn set_pitch_bend(&mut self, pitch_bend: i16) {
        self.pitch_bend = pitch_bend;
    }

    fn set_course_tune(&mut self, course_tune: i8) {
        self.course_tune = course_tune;
    }

    fn set_fine_tune(&mut self, fine_tune: i16) {
        self.fine_tune = fine_tune;
    }

    pub fn set_is_sub_oscillator(&mut self, is_sub_oscillator: bool) {
        self.is_sub_oscillator = is_sub_oscillator;
    }

    fn set_shape_parameter1(&mut self, parameter: f32) {
        self.wave_generator.set_shape_parameter1(parameter);
    }

    fn set_shape_parameter2(&mut self, parameter: f32) {
        self.wave_generator.set_shape_parameter2(parameter);
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
        WaveShape::GigaSaw => Box::new(GigaSaw::new(sample_rate)),
        WaveShape::AM => Box::new(AM::new(sample_rate)),
        WaveShape::FM => Box::new(FM::new(sample_rate)),
        WaveShape::Noise => Box::new(Noise::new()),
    }
}

fn midi_note_to_frequency(note_number: u8) -> f32 {
    MIDI_NOTE_FREQUENCIES[note_number as usize].0
}

fn frequency_from_cents(frequency: f32, cents: i16) -> f32 {
    frequency * (2.0f32.powf(cents as f32 / CENTS_PER_OCTAVE))
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
        oscillator.set_frequency(100.0);

        let first_value = oscillator.generate(None);
        for _ in 0..5 {
            assert!(!f32_value_equality(oscillator.generate(None), first_value));
        }
        let first_sample = oscillator.generate(None);

        oscillator.reset();
        oscillator.set_shape_parameter1(1.0);
        oscillator.set_shape_parameter2(2.0);

        let first_value = oscillator.generate(None);
        for _ in 0..5 {
            assert!(!f32_value_equality(oscillator.generate(None), first_value));
        }
        let second_sample = oscillator.generate(None);

        assert_ne!(first_sample, second_sample);
    }

    #[test]
    fn reset_correctly_resets_oscillator_phase() {
        let sample_rate = 44100;
        let wave_shape = WaveShape::Sine;
        let mut oscillator = Oscillator::new(sample_rate, wave_shape);
        let frequency = 100.0;
        oscillator.set_frequency(frequency);

        let first_value = oscillator.generate(None);
        for _ in 0..5 {
            assert!(!f32_value_equality(oscillator.generate(None), first_value));
        }
        oscillator.reset();
        assert!(f32_value_equality(oscillator.generate(None), first_value));
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
            get_wave_generator_from_wave_shape(sample_rate, WaveShape::GigaSaw).shape(),
            WaveShape::GigaSaw
        );
        assert_eq!(
            get_wave_generator_from_wave_shape(sample_rate, WaveShape::Triangle).shape(),
            WaveShape::Triangle
        );
    }

    #[test]
    fn midi_note_to_frequency_returns_correct_values_for_note_numbers() {
        let notes: [u8; 4] = [0, 21, 72, 127];
        let expected_frequencies: [f32; 4] = [8.175, 27.5, 523.251, 12543.854];

        for i in 0..notes.len() {
            assert!(f32_value_equality(
                midi_note_to_frequency(notes[i]),
                expected_frequencies[i]
            ));
        }
    }

    #[test]
    fn frequency_from_cents_returns_correct_values_for_value_frequencies_and_cents() {
        let frequencies: [f32; 4] = [8.175, 27.5, 523.251, 12543.854];
        let cents = 50;
        let expected_frequencies: [f32; 4] = [8.414546, 28.30581, 538.5834, 12911.417];

        for i in 0..frequencies.len() {
            assert!(f32_value_equality(
                frequency_from_cents(frequencies[i], cents),
                expected_frequencies[i]
            ));
        }
    }

    #[test]
    fn frequency_from_cents_returns_correct_values_for_value_frequencies_and_negative_cents() {
        let frequencies: [f32; 4] = [8.175, 27.5, 523.251, 12543.854];
        let cents = -50;
        let expected_frequencies: [f32; 4] = [7.9422736, 26.717129, 508.35504, 12186.754];

        for i in 0..frequencies.len() {
            assert!(f32_value_equality(
                frequency_from_cents(frequencies[i], cents),
                expected_frequencies[i]
            ));
        }
    }
}
