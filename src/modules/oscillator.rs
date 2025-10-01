pub mod am;
mod constants;
pub mod fm;
mod generate_wave_trait;
pub mod gigasaw;
pub mod noise;
pub mod pulse;
pub mod ramp;
pub mod saw;
pub mod sine;
pub mod square;
pub mod triangle;

use self::am::AM;
use self::constants::{
    DEFAULT_KEY_SYNC_ENABLED, DEFAULT_NOTE_FREQUENCY, DEFAULT_PORTAMENTO_SPEED_IN_BUFFERS,
    MAX_MIDI_NOTE_NUMBER, MAX_NOTE_FREQUENCY, MIDI_NOTE_FREQUENCIES, MIN_MIDI_NOTE_NUMBER,
    MIN_NOTE_FREQUENCY,
};
use self::fm::FM;
use self::gigasaw::GigaSaw;
use self::noise::Noise;
use self::pulse::Pulse;
use self::ramp::Ramp;
use self::saw::Saw;
use self::sine::Sine;
use self::square::Square;
use self::triangle::Triangle;
use crate::math;
use crate::math::{dbfs_to_f32_sample, f32s_are_equal, load_f32_from_atomic_u32};
use generate_wave_trait::GenerateWave;
use std::sync::Arc;
use std::sync::atomic::Ordering::{Acquire, Relaxed, Release};
use std::sync::atomic::{AtomicBool, AtomicI8, AtomicI16, AtomicU8, AtomicU16, AtomicU32};

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

#[derive(Default, Debug)]
pub enum HardSyncRole {
    #[default]
    None,
    Source(Arc<AtomicBool>),
    Synced(Arc<AtomicBool>),
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
            _ => WaveShape::default(),
        }
    }
}

#[derive(Debug)]
pub struct OscillatorParameters {
    pub fine_tune: AtomicI8,
    pub course_tune: AtomicI8,
    pub pitch_bend: AtomicI16,
    pub shape_parameter1: AtomicU32,
    pub shape_parameter2: AtomicU32,
    pub wave_shape_index: AtomicU8,
    pub gate_flag: AtomicBool,
    pub key_sync_enabled: AtomicBool,
    pub hard_sync_enabled: AtomicBool,
    pub portamento_is_enabled: AtomicBool,
    pub portamento_speed: AtomicU16,
    pub clipper_boost: AtomicU8,
}

impl Default for OscillatorParameters {
    fn default() -> Self {
        Self {
            fine_tune: AtomicI8::new(0),
            course_tune: AtomicI8::new(0),
            pitch_bend: AtomicI16::new(0),
            shape_parameter1: AtomicU32::new(0),
            shape_parameter2: AtomicU32::new(0),
            wave_shape_index: AtomicU8::new(WaveShape::default() as u8),
            gate_flag: AtomicBool::new(false),
            key_sync_enabled: AtomicBool::new(DEFAULT_KEY_SYNC_ENABLED),
            hard_sync_enabled: AtomicBool::new(false),
            portamento_is_enabled: AtomicBool::new(false),
            portamento_speed: AtomicU16::new(DEFAULT_PORTAMENTO_SPEED_IN_BUFFERS),
            clipper_boost: AtomicU8::new(0),
        }
    }
}

#[derive(Default, Debug, Copy, Clone)]
struct Portamento {
    is_enabled: bool,
    speed: u16,
    target_frequency: f32,
    increment: f32,
    recalculate_increment: bool,
}

#[derive(Default, Debug)]
pub struct HardSync {
    is_enabled: bool,
    last_sample: f32,
    sync_role: HardSyncRole,
}

#[derive(Default, Debug, Copy, Clone)]
pub struct Tuning {
    frequency: f32,
    pitch_bend: i16,
    course: i8,
    fine: i8,
    is_sub: bool,
}

pub struct Oscillator {
    sample_rate: u32,
    wave_generator: Box<dyn GenerateWave + Send + Sync>,
    wave_shape_index: u8,
    key_sync_enabled: bool,
    clipper_boost: u8,
    tuning: Tuning,
    hard_sync: HardSync,
    portamento: Portamento,
}

impl Oscillator {
    pub fn new(sample_rate: u32, wave_shape: WaveShape) -> Self {
        log::debug!("Constructing Oscillator Module: {wave_shape:?}");
        let wave_generator = get_wave_generator_from_wave_shape(sample_rate, wave_shape);

        let portamento = Portamento {
            is_enabled: false,
            target_frequency: DEFAULT_NOTE_FREQUENCY,
            speed: DEFAULT_PORTAMENTO_SPEED_IN_BUFFERS,
            recalculate_increment: false,
            ..Portamento::default()
        };

        let tuning = Tuning {
            frequency: DEFAULT_NOTE_FREQUENCY,
            pitch_bend: 0,
            course: 0,
            fine: 0,
            is_sub: false,
        };

        let hard_sync = HardSync {
            sync_role: HardSyncRole::None,
            is_enabled: false,
            last_sample: 0.0,
        };

        Self {
            sample_rate,
            wave_generator,
            wave_shape_index: WaveShape::default() as u8,
            key_sync_enabled: DEFAULT_KEY_SYNC_ENABLED,
            tuning,
            hard_sync,
            portamento,
            clipper_boost: 0,
        }
    }

    pub fn set_parameters(&mut self, parameters: &OscillatorParameters) {
        self.set_shape_parameter1(load_f32_from_atomic_u32(&parameters.shape_parameter1));
        self.set_shape_parameter2(load_f32_from_atomic_u32(&parameters.shape_parameter2));
        self.set_wave_shape_index(parameters.wave_shape_index.load(Relaxed));
        self.set_pitch_bend(parameters.pitch_bend.load(Relaxed));
        self.set_course_tune(parameters.course_tune.load(Relaxed));
        self.set_fine_tune(parameters.fine_tune.load(Relaxed));
        self.set_key_sync_enabled(parameters.key_sync_enabled.load(Relaxed));
        self.set_hard_sync_enabled(parameters.hard_sync_enabled.load(Relaxed));
        self.set_portamento(
            parameters.portamento_is_enabled.load(Relaxed),
            parameters.portamento_speed.load(Relaxed),
        );
        self.set_gate(&parameters.gate_flag);
        self.set_clipper_boost(parameters.clipper_boost.load(Relaxed));
    }

    pub fn generate(&mut self, mut modulation: Option<f32>) -> f32 {
        if modulation == Some(0.0) {
            modulation = None;
        }

        let mut next_sample = self
            .wave_generator
            .next_sample(self.tuning.frequency, modulation);

        next_sample = self.clip_signal(next_sample);

        if !self.hard_sync.is_enabled {
            return next_sample;
        }

        self.perform_sync_role(next_sample);
        self.hard_sync.last_sample = next_sample;

        next_sample
    }

    fn perform_sync_role(&mut self, next_sample: f32) {
        match &self.hard_sync.sync_role {
            HardSyncRole::None => {}
            HardSyncRole::Source(buffer) => {
                if self.hard_sync.last_sample.is_sign_negative() && next_sample.is_sign_positive() {
                    buffer.store(true, Relaxed);
                }
            }
            HardSyncRole::Synced(buffer) => {
                let sync_state = buffer.load(Acquire);
                if sync_state {
                    self.wave_generator.reset();
                    buffer.store(false, Release);
                }
            }
        }
    }

    pub fn set_wave_shape(&mut self, wave_shape: WaveShape) {
        if wave_shape == self.wave_generator.shape() {
            return;
        }

        log::info!("Setting Oscillator Shape to {wave_shape:#?}");
        self.wave_generator = get_wave_generator_from_wave_shape(self.sample_rate, wave_shape);
    }

    pub fn set_wave_shape_index(&mut self, wave_shape_index: u8) {
        if wave_shape_index == self.wave_shape_index {
            return;
        }

        let wave_shape = WaveShape::from_index(wave_shape_index);
        self.set_wave_shape(wave_shape);
        self.wave_shape_index = wave_shape_index;
    }

    pub fn set_frequency(&mut self, tone_frequency: f32) {
        self.tuning.frequency = tone_frequency;
    }

    pub fn set_phase(&mut self, phase: f32) {
        self.wave_generator.set_phase(phase);
    }

    pub fn set_hard_sync_role(&mut self, sync_role: HardSyncRole) {
        self.hard_sync.sync_role = sync_role;
    }

    pub fn reset(&mut self) {
        self.wave_generator.reset();
    }

    pub fn clip_signal(&mut self, signal: f32) -> f32 {
        if self.clipper_boost == 0 {
            return signal;
        }

        let boost = dbfs_to_f32_sample(f32::from(self.clipper_boost));
        let boosted_signal = signal * boost;
        boosted_signal.clamp(-1.0, 1.0)
    }

    pub fn tune(&mut self, mut note_number: u8) {
        if self.tuning.course != 0 {
            note_number = i16::from(note_number)
                .saturating_add(i16::from(self.tuning.course))
                .clamp(MIN_MIDI_NOTE_NUMBER, MAX_MIDI_NOTE_NUMBER) as u8;
        }

        if self.tuning.is_sub {
            note_number = i16::from(note_number)
                .saturating_sub(12)
                .clamp(MIN_MIDI_NOTE_NUMBER, MAX_MIDI_NOTE_NUMBER) as u8;
        }

        let mut note_frequency = midi_note_to_frequency(note_number);

        if self.tuning.fine != 0 {
            note_frequency =
                math::frequency_from_cents(note_frequency, i16::from(self.tuning.fine));
        }

        if self.portamento.is_enabled {
            note_frequency = self.run_portamento(note_frequency);
        }

        if self.tuning.pitch_bend != 0 {
            note_frequency = math::frequency_from_cents(note_frequency, self.tuning.pitch_bend)
                .clamp(MIN_NOTE_FREQUENCY, MAX_NOTE_FREQUENCY);
        }

        self.tuning.frequency = note_frequency;
    }

    fn run_portamento(&mut self, frequency: f32) -> f32 {
        if self.portamento.recalculate_increment {
            self.recalculate_portamento_increment(frequency);
        }

        if f32s_are_equal(frequency, self.tuning.frequency) {
            return frequency;
        }

        let frequency_delta = (self.tuning.frequency - self.portamento.target_frequency).abs();

        if frequency_delta < self.portamento.increment.abs() {
            self.portamento.target_frequency
        } else {
            (self.tuning.frequency + self.portamento.increment)
                .clamp(MIN_NOTE_FREQUENCY, MAX_NOTE_FREQUENCY)
        }
    }

    fn recalculate_portamento_increment(&mut self, new_frequency: f32) {
        let increment = (new_frequency - self.tuning.frequency) / f32::from(self.portamento.speed);
        self.portamento.increment = increment;
        self.portamento.target_frequency = new_frequency;
        self.portamento.recalculate_increment = false;
    }

    fn set_gate(&mut self, gate_flag: &AtomicBool) {
        let gate_on = gate_flag.swap(false, Acquire);
        if !gate_on {
            return;
        }

        self.portamento.recalculate_increment = true;

        if self.key_sync_enabled {
            self.reset();
        }
    }

    fn set_portamento(&mut self, is_enabled: bool, speed: u16) {
        self.portamento.is_enabled = is_enabled;
        self.portamento.speed = speed;
    }

    fn set_key_sync_enabled(&mut self, key_sync_enabled: bool) {
        self.key_sync_enabled = key_sync_enabled;
    }

    fn set_hard_sync_enabled(&mut self, hard_sync_enabled: bool) {
        self.hard_sync.is_enabled = hard_sync_enabled;
    }

    fn set_pitch_bend(&mut self, pitch_bend: i16) {
        self.tuning.pitch_bend = pitch_bend;
    }

    fn set_course_tune(&mut self, course_tune: i8) {
        self.tuning.course = course_tune;
    }

    fn set_fine_tune(&mut self, fine_tune: i8) {
        self.tuning.fine = fine_tune;
    }

    pub fn set_is_sub_oscillator(&mut self, is_sub_oscillator: bool) {
        self.tuning.is_sub = is_sub_oscillator;
    }

    fn set_shape_parameter1(&mut self, parameter: f32) {
        self.wave_generator.set_shape_parameter1(parameter);
    }

    fn set_shape_parameter2(&mut self, parameter: f32) {
        self.wave_generator.set_shape_parameter2(parameter);
    }

    fn set_clipper_boost(&mut self, clipper_boost: u8) {
        self.clipper_boost = clipper_boost;
    }
}

fn get_wave_generator_from_wave_shape(
    sample_rate: u32,
    wave_shape: WaveShape,
) -> Box<dyn GenerateWave + Send + Sync> {
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::math::f32s_are_equal;

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
            assert!(!f32s_are_equal(oscillator.generate(None), first_value));
        }
        let first_sample = oscillator.generate(None);

        oscillator.reset();
        oscillator.set_shape_parameter1(1.0);
        oscillator.set_shape_parameter2(2.0);

        let first_value = oscillator.generate(None);
        for _ in 0..5 {
            assert!(!f32s_are_equal(oscillator.generate(None), first_value));
        }
        let second_sample = oscillator.generate(None);

        assert!(!f32s_are_equal(first_sample, second_sample));
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
            assert!(!f32s_are_equal(oscillator.generate(None), first_value));
        }
        oscillator.reset();
        assert!(f32s_are_equal(oscillator.generate(None), first_value));
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
    fn set_synced_buffer_correctly_sets_sync_role() {
        let mut oscillator = Oscillator::new(44100, WaveShape::Sine);
        assert!(matches!(oscillator.hard_sync.sync_role, HardSyncRole::None));
        oscillator.set_hard_sync_role(HardSyncRole::Synced(Arc::new(AtomicBool::new(false))));
        assert!(matches!(
            oscillator.hard_sync.sync_role,
            HardSyncRole::Synced(_)
        ));
    }

    #[test]
    fn generate_sets_hard_sync_buffer_to_true_when_enabled_and_sync_role_is_source_and_wave_phase_roles_over()
     {
        let sync_buffer = Arc::new(AtomicBool::new(false));
        let mut oscillator = Oscillator::new(44100, WaveShape::Sine);
        oscillator.set_hard_sync_enabled(true);
        oscillator.set_hard_sync_role(HardSyncRole::Source(sync_buffer.clone()));
        assert!(!sync_buffer.load(Relaxed));

        let negative_sample = -0.01;
        oscillator.hard_sync.last_sample = negative_sample;
        let _ = oscillator.generate(None);

        assert!(sync_buffer.load(Relaxed));
    }

    #[test]
    fn generate_syncs_wave_generator_when_sync_enabled_and_sync_role_is_synced_and_sync_buffer_is_true()
     {
        let sync_buffer = Arc::new(AtomicBool::new(false));
        let mut oscillator = Oscillator::new(44100, WaveShape::Sine);
        oscillator.set_hard_sync_role(HardSyncRole::Synced(sync_buffer.clone()));
        oscillator.set_hard_sync_enabled(true);

        let expected_first_sample = 0.037_266_6;

        let sample = oscillator.generate(None);
        assert!(
            f32s_are_equal(sample, expected_first_sample),
            "Expected {expected_first_sample:?}, but got {sample:?}"
        );

        for _i in 0..3 {
            let _ = oscillator.generate(None);
        }

        sync_buffer.store(true, Release);

        let not_yet_synced = oscillator.generate(None);
        assert!(
            !f32s_are_equal(not_yet_synced, expected_first_sample),
            "Expected {expected_first_sample:?}, but got {not_yet_synced:?}"
        );

        let first_synced_sample = oscillator.generate(None);
        assert!(
            f32s_are_equal(first_synced_sample, expected_first_sample),
            "Expected {expected_first_sample:?}, but got {first_synced_sample:?}"
        );
    }
}
