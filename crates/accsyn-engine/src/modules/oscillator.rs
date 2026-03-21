/// Amplitude modulation oscillator.
pub mod am;
/// Shared oscillator constants for tuning, phase, and shape parameters.
pub mod constants;
/// Frequency modulation oscillator.
pub mod fm;
mod generate_wave_trait;
/// White noise generator.
pub mod noise;
/// Pulse wave oscillator with variable pulse width.
pub mod pulse;
/// Ramp (reverse sawtooth) wave oscillator.
pub mod ramp;
/// Sawtooth wave oscillator.
pub mod saw;
/// Sine wave oscillator.
pub mod sine;
/// Square wave oscillator.
pub mod square;
/// Multi-voice detuned supersaw oscillator.
pub mod supersaw;
/// Triangle wave oscillator.
pub mod triangle;

use self::am::AM;
use self::constants::{
    DEFAULT_KEY_SYNC_ENABLED, DEFAULT_NOTE_FREQUENCY, DEFAULT_PORTAMENTO_TIME_IN_BUFFERS,
    MAX_MIDI_NOTE_NUMBER, MAX_NOTE_FREQUENCY, MIN_MIDI_NOTE_NUMBER, MIN_NOTE_FREQUENCY,
};
use self::fm::FM;
use self::noise::Noise;
use self::pulse::Pulse;
use self::ramp::Ramp;
use self::saw::Saw;
use self::sine::Sine;
use self::square::Square;
use self::supersaw::Supersaw;
use self::triangle::Triangle;
use crate::modules::oscillator::constants::{
    DEFAULT_HARD_SYNC_ENABLED, DEFAULT_PORTAMENTO_ENABLED,
};
use accsyn_types::defaults::Defaults;
use accsyn_types::math;
use accsyn_types::math::{dbfs_to_f32_sample, f32s_are_equal};
use generate_wave_trait::GenerateWave;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use std::sync::atomic::Ordering::{Acquire, Relaxed, Release};
use std::sync::atomic::{AtomicBool, AtomicU8};
use strum_macros::{EnumCount, EnumIter, FromRepr};
use accsyn_types::parameter_types::{Cents, NormalizedValue, PitchBend, PortamentoBuffers, Semitones};

/// Index of the first available wave shape variant.
pub const FIRST_WAVE_SHAPE_INDEX: u32 = 0;
/// Index of the last available wave shape variant.
pub const LAST_WAVE_SHAPE_INDEX: u32 = 9;

/// Available waveform shapes for oscillator generation.
#[derive(Default, Debug, Clone, Copy, PartialEq, Eq, EnumCount, EnumIter, FromRepr)]
#[repr(u8)]
pub enum WaveShape {
    /// Sine wave.
    #[default]
    Sine,
    /// Triangle wave.
    Triangle,
    /// Square wave.
    Square,
    /// Sawtooth wave.
    Saw,
    /// Pulse wave with variable width.
    Pulse,
    /// Ramp (reverse sawtooth) wave.
    Ramp,
    /// Multi-voice detuned supersaw wave.
    Supersaw,
    /// Amplitude modulation synthesis.
    AM,
    /// Frequency modulation synthesis.
    FM,
    /// White noise.
    Noise,
}

impl WaveShape {
    /// Converts a numeric index to the corresponding wave shape, defaulting on invalid values.
    pub fn from_index(index: u8) -> Self {
        Self::from_repr(index).unwrap_or_default()
    }
}

/// Role an oscillator plays in hard sync (source, synced, or none).
#[derive(Default, Debug)]
pub enum HardSyncRole {
    /// No hard sync role assigned.
    #[default]
    None,
    /// Source oscillator that triggers sync resets via the shared flag.
    Source(Arc<AtomicBool>),
    /// Synced oscillator that resets its phase when the shared flag is set.
    Synced(Arc<AtomicBool>),
}

/// Shared atomic parameters for controlling an oscillator from the UI thread.
#[derive(Debug, Serialize, Deserialize)]
pub struct OscillatorParameters {
    /// Fine tuning offset in cents.
    pub fine_tune: Cents,
    /// Coarse tuning offset in semitones.
    pub course_tune: Semitones,
    /// Pitch bend amount from MIDI controller.
    pub pitch_bend: PitchBend,
    /// First wave-shape-specific parameter (e.g., FM amount, pulse width).
    pub shape_parameter1: NormalizedValue,
    /// Second wave-shape-specific parameter (e.g., FM ratio, AM tone).
    pub shape_parameter2: NormalizedValue,
    /// Index selecting the active wave shape.
    pub wave_shape_index: AtomicU8,
    /// Flag indicating a new note gate event has occurred.
    pub gate_flag: AtomicBool,
    /// Whether key sync (phase reset on note-on) is enabled.
    pub key_sync_enabled: AtomicBool,
    /// Whether hard sync between oscillators is enabled.
    pub hard_sync_enabled: AtomicBool,
    /// Whether portamento (pitch glide) is enabled.
    pub portamento_enabled: AtomicBool,
    /// Portamento glide time in buffer increments.
    pub portamento_time: PortamentoBuffers,
    /// Clipper boost amount in dB for signal saturation.
    pub clipper_boost: AtomicU8,
}

impl Default for OscillatorParameters {
    fn default() -> Self {
        Self {
            fine_tune: Cents::default(),
            course_tune: Semitones::default(),
            pitch_bend: PitchBend::default(),
            shape_parameter1: NormalizedValue::default(),
            shape_parameter2: NormalizedValue::default(),
            wave_shape_index: AtomicU8::new(WaveShape::default() as u8),
            gate_flag: AtomicBool::new(false),
            key_sync_enabled: AtomicBool::new(DEFAULT_KEY_SYNC_ENABLED),
            hard_sync_enabled: AtomicBool::new(DEFAULT_HARD_SYNC_ENABLED),
            portamento_enabled: AtomicBool::new(DEFAULT_PORTAMENTO_ENABLED),
            portamento_time: PortamentoBuffers::new(DEFAULT_PORTAMENTO_TIME_IN_BUFFERS),
            clipper_boost: AtomicU8::new(0),
        }
    }
}

#[derive(Debug, Copy, Clone)]
struct Portamento {
    is_enabled: bool,
    time: u16,
    target_frequency: f32,
    increment: f32,
    recalculate_increment: bool,
}

impl Default for Portamento {
    fn default() -> Self {
        Self {
            is_enabled: false,
            time: DEFAULT_PORTAMENTO_TIME_IN_BUFFERS,
            target_frequency: DEFAULT_NOTE_FREQUENCY,
            increment: 0.0,
            recalculate_increment: false,
        }
    }
}

/// Tracks hard sync state between oscillators.
#[derive(Debug)]
pub struct HardSync {
    is_enabled: bool,
    last_sample: f32,
    sync_role: HardSyncRole,
}

impl Default for HardSync {
    fn default() -> Self {
        Self {
            sync_role: HardSyncRole::None,
            is_enabled: false,
            last_sample: 0.0,
        }
    }
}

/// Holds pitch tuning state including frequency, pitch bend, and transposition.
#[derive(Debug, Copy, Clone)]
pub struct Tuning {
    frequency: f32,
    pitch_bend: i16,
    course: i8,
    fine: i8,
    is_sub: bool,
}

impl Default for Tuning {
    fn default() -> Self {
        Self {
            frequency: DEFAULT_NOTE_FREQUENCY,
            pitch_bend: 0,
            course: 0,
            fine: 0,
            is_sub: false,
        }
    }
}

/// Core oscillator module that generates audio samples using a selectable wave shape.
pub struct Oscillator {
    sample_rate: u32,
    wave_generator: Box<dyn GenerateWave + Send + Sync>,
    wave_shape_index: u8,
    key_sync_enabled: bool,
    clipper_boost: u8,
    tuning: Tuning,
    hard_sync: HardSync,
    portamento: Portamento,
    aftertouch: f32,
}

impl Oscillator {
    /// Creates a new oscillator with the given sample rate and initial wave shape.
    pub fn new(sample_rate: u32, wave_shape: WaveShape) -> Self {
        log::debug!("Constructing Oscillator Module: {wave_shape:?}");
        let wave_generator = get_wave_generator_from_wave_shape(sample_rate, wave_shape);

        Self {
            sample_rate,
            wave_generator,
            wave_shape_index: WaveShape::default() as u8,
            key_sync_enabled: DEFAULT_KEY_SYNC_ENABLED,
            tuning: Tuning::default(),
            hard_sync: HardSync::default(),
            portamento: Portamento::default(),
            clipper_boost: 0,
            aftertouch: 0.0,
        }
    }

    /// Updates all oscillator settings from the shared parameter block.
    pub fn set_parameters(&mut self, parameters: &OscillatorParameters) {
        self.set_shape_parameter1(parameters.shape_parameter1.load());
        self.set_shape_parameter2(parameters.shape_parameter2.load());
        self.set_wave_shape_index(parameters.wave_shape_index.load(Relaxed));
        self.set_pitch_bend(parameters.pitch_bend.load());
        self.set_course_tune(parameters.course_tune.load());
        self.set_fine_tune(parameters.fine_tune.load());
        self.set_key_sync_enabled(parameters.key_sync_enabled.load(Relaxed));
        self.set_hard_sync_enabled(parameters.hard_sync_enabled.load(Relaxed));
        self.set_portamento(
            parameters.portamento_enabled.load(Relaxed),
            parameters.portamento_time.load(),
        );
        self.set_gate(&parameters.gate_flag);
        self.set_clipper_boost(parameters.clipper_boost.load(Relaxed));
    }

    /// Generates the next audio sample with optional modulation input.
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

    /// Changes the oscillator's waveform shape, replacing the wave generator if different.
    pub fn set_wave_shape(&mut self, wave_shape: WaveShape) {
        if wave_shape == self.wave_generator.shape() {
            return;
        }

        log::info!("Setting Oscillator Shape to {wave_shape:#?}");
        self.wave_generator = get_wave_generator_from_wave_shape(self.sample_rate, wave_shape);
    }

    /// Sets the wave shape by numeric index, updating the generator if changed.
    pub fn set_wave_shape_index(&mut self, wave_shape_index: u8) {
        if wave_shape_index == self.wave_shape_index {
            return;
        }

        let wave_shape = WaveShape::from_index(wave_shape_index);
        self.set_wave_shape(wave_shape);
        self.wave_shape_index = wave_shape_index;
    }

    /// Sets the oscillator's base tone frequency in Hz.
    pub fn set_frequency(&mut self, tone_frequency: f32) {
        self.tuning.frequency = tone_frequency;
    }

    /// Sets the oscillator's waveform phase position.
    pub fn set_phase(&mut self, phase: f32) {
        self.wave_generator.set_phase(phase);
    }

    /// Assigns a hard sync role (source, synced, or none) to this oscillator.
    pub fn set_hard_sync_role(&mut self, sync_role: HardSyncRole) {
        self.hard_sync.sync_role = sync_role;
    }

    /// Resets the oscillator's wave generator phase to the initial position.
    pub fn reset(&mut self) {
        self.wave_generator.reset();
    }

    /// Sets the MIDI aftertouch pressure value for clipper modulation.
    pub fn set_aftertouch(&mut self, aftertouch: f32) {
        self.aftertouch = aftertouch;
    }

    /// Applies clipper boost and aftertouch saturation to the signal, clamping to [-1, 1].
    pub fn clip_signal(&mut self, signal: f32) -> f32 {
        if self.clipper_boost == 0 && self.aftertouch == 0.0 {
            return signal;
        }

        let boost = dbfs_to_f32_sample(f32::from(self.clipper_boost));
        let boosted_signal = signal * (boost + (boost * self.aftertouch));
        boosted_signal.clamp(-1.0, 1.0)
    }

    /// Calculates and sets the oscillator frequency from a MIDI note number with tuning offsets.
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
        let increment = (new_frequency - self.tuning.frequency) / f32::from(self.portamento.time);
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

    fn set_portamento(&mut self, is_enabled: bool, time: u16) {
        self.portamento.is_enabled = is_enabled;
        self.portamento.time = time;
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

    /// Enables or disables sub-oscillator mode, which tunes down one octave.
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
        WaveShape::Supersaw => Box::new(Supersaw::new(sample_rate)),
        WaveShape::AM => Box::new(AM::new(sample_rate)),
        WaveShape::FM => Box::new(FM::new(sample_rate)),
        WaveShape::Noise => Box::new(Noise::new()),
    }
}

fn midi_note_to_frequency(note_number: u8) -> f32 {
    Defaults::MIDI_NOTE_FREQUENCIES[note_number as usize].0
}

#[cfg(test)]
mod tests {
    use super::*;
    use accsyn_types::math::f32s_are_equal;

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
            get_wave_generator_from_wave_shape(sample_rate, WaveShape::Supersaw).shape(),
            WaveShape::Supersaw
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
