use crate::modules::oscillator::constants::DEFAULT_PHASE;
use crate::modules::oscillator::{Oscillator, WaveShape};
use accsyn_core::defaults::Defaults;
use accsyn_core::math::f32s_are_equal;
use accsyn_core::parameter_types::{Balance, Hertz, LfoRange, NormalizedValue};
use serde::{Deserialize, Serialize};
use std::default::Default;
use std::sync::atomic::Ordering::Relaxed;
use std::sync::atomic::{AtomicBool, AtomicU8, AtomicU16};

/// Shared atomic parameters for controlling an LFO from the UI thread.
#[derive(Debug, Serialize, Deserialize)]
pub struct LfoParameters {
    /// LFO oscillation frequency in Hz.
    pub frequency: Hertz,
    /// LFO oscillation frequency in Hz based on clock sync interval and thirty second note duration
    pub synced_frequency: Hertz,
    /// A trigger condition to alert the LFO that a thirty second note event has occured
    pub sync_triggered: AtomicBool,
    /// LFO oscillation frequency in thirty-second notes when synced to a clock
    pub thirty_second_notes: AtomicU16,
    /// Indicates whether the lfo is synced to clock
    pub clock_synced: AtomicBool,
    /// Indicates whether the lfo is synced to key press
    pub key_synced: AtomicBool,
    /// Center value (DC offset) of the LFO output.
    pub center_value: Balance,
    /// Peak-to-peak range of the LFO output.
    pub range: LfoRange,
    /// Starting phase offset of the LFO waveform.
    pub phase: NormalizedValue,
    /// Index selecting the LFO wave shape.
    pub wave_shape: AtomicU8,
    /// Flag to trigger a phase reset on the next processing cycle.
    pub reset: AtomicBool,
    /// Gate state flag: 0 = waiting, 1 = gate triggered
    pub gate_flag: AtomicBool,
}

impl LfoParameters {
    /// Replace all the values in these `LfoParameters` with the values from the provided `LfoParameters`.
    pub fn assign_from(&self, parameters: &LfoParameters) {
        self.frequency.store(parameters.frequency.load());
        self.synced_frequency
            .store(parameters.synced_frequency.load());
        self.clock_synced
            .store(parameters.clock_synced.load(Relaxed), Relaxed);
        self.key_synced
            .store(parameters.key_synced.load(Relaxed), Relaxed);
        self.center_value.store(parameters.center_value.load());
        self.range.store(parameters.range.load());
        self.phase.store(parameters.phase.load());
        self.wave_shape
            .store(parameters.wave_shape.load(Relaxed), Relaxed);
        self.reset.store(parameters.reset.load(Relaxed), Relaxed);
    }
}

impl Default for LfoParameters {
    fn default() -> Self {
        Self {
            frequency: Hertz::new(Defaults::LFO_FREQUENCY),
            synced_frequency: Hertz::new(Defaults::LFO_SYNCED_FREQUENCY),
            sync_triggered: AtomicBool::new(false),
            thirty_second_notes: AtomicU16::new(Defaults::LFO_THIRTY_SECOND_NOTES),
            clock_synced: AtomicBool::new(Defaults::CLOCK_SYNCED_STATE),
            key_synced: AtomicBool::new(Defaults::KEY_SYNCED_STATE),
            center_value: Balance::new(Defaults::LFO_CENTER_VALUE),
            range: LfoRange::new(Defaults::LFO_RANGE),
            phase: NormalizedValue::new(Defaults::LFO_PHASE),
            wave_shape: AtomicU8::new(WaveShape::default() as u8),
            reset: AtomicBool::new(Defaults::LFO_RESET_STATE),
            gate_flag: AtomicBool::new(Defaults::LFO_GATE_STATE),
        }
    }
}

/// Low-frequency oscillator for modulating synthesis parameters.
pub struct Lfo {
    oscillator: Oscillator,
    frequency: f32,
    sync_armed: bool,
    clock_synced: bool,
    key_synced: bool,
    center_value: f32,
    wave_shape_index: u8,
    range: f32,
    phase: f32,
}

impl Lfo {
    /// Creates a new LFO with default frequency, range, and sine wave shape.
    pub(crate) fn new(sample_rate: u32) -> Self {
        log::debug!(target: "synthesizer::modules::lfo", "Constructing LFO Module");
        let oscillator = Oscillator::new(sample_rate, WaveShape::Sine);
        Self {
            oscillator,
            frequency: Defaults::LFO_FREQUENCY,
            sync_armed: false,
            clock_synced: false,
            key_synced: false,
            center_value: Defaults::LFO_CENTER_VALUE,
            range: Defaults::LFO_RANGE,
            phase: Defaults::LFO_PHASE,
            wave_shape_index: WaveShape::Sine as u8,
        }
    }

    /// Updates all LFO settings from the shared parameter block.
    pub fn set_parameters(&mut self, parameters: &LfoParameters) {
        self.set_clock_synced(parameters);
        self.set_key_synced(parameters.key_synced.load(Relaxed));
        if parameters.gate_flag.swap(false, Relaxed) {
            self.handle_gate_flag();
        }
        self.set_center_value(parameters.center_value.load());
        self.set_range(parameters.range.load());
        self.set_phase(parameters.phase.load());
        self.set_wave_shape(parameters.wave_shape.load(Relaxed));
        if self.sync_armed && parameters.sync_triggered.load(Relaxed) {
            self.reset();
            parameters.reset.store(false, Relaxed);
            self.sync_armed = false;
        }
        if parameters.reset.load(Relaxed) {
            self.reset();
            parameters.reset.store(false, Relaxed);
        }
    }

    fn set_clock_synced(&mut self, parameters: &LfoParameters) {
        if parameters.clock_synced.load(Relaxed) {
            if !self.clock_synced {
                self.sync_armed = true;
                self.clock_synced = true;
            }
            self.set_frequency(parameters.synced_frequency.load());
        } else {
            self.clock_synced = false;
            self.sync_armed = false;
            self.set_frequency(parameters.frequency.load());
        }
    }

    fn handle_gate_flag(&mut self) {
        if !self.key_synced {
            return;
        }

        if self.clock_synced {
            self.sync_armed = true;
            return;
        }

        self.reset();
    }

    /// Generates the next LFO output sample, scaled by center value and range.
    pub fn generate(&mut self, modulation: Option<f32>) -> f32 {
        if self.range == 0.0 || self.frequency == 0.0 {
            return 0.0;
        }
        let wave_sample = self.oscillator.generate(modulation, None);
        self.center_value + (wave_sample * (self.range / 2.0))
    }

    /// Sets the LFO frequency in Hz, clamped to the valid range.
    pub fn set_frequency(&mut self, frequency: f32) {
        self.frequency = frequency.clamp(Defaults::MIN_LFO_FREQUENCY, Defaults::MAX_LFO_FREQUENCY);
        self.oscillator.set_frequency(self.frequency);
    }

    /// Sets the key synced state
    pub fn set_key_synced(&mut self, is_synced: bool) {
        self.key_synced = is_synced;
    }

    /// Sets the LFO center value (DC offset), clamped to [-1, 1].
    pub fn set_center_value(&mut self, center_value: f32) {
        self.center_value = center_value.clamp(
            Defaults::MIN_LFO_CENTER_VALUE,
            Defaults::MAX_LFO_CENTER_VALUE,
        );
    }

    /// Sets the LFO output range (peak-to-peak), clamped to valid bounds or zero.
    pub fn set_range(&mut self, range: f32) {
        self.range = if range == 0.0 {
            self.reset();
            0.0
        } else {
            range.clamp(Defaults::MIN_LFO_RANGE, Defaults::MAX_LFO_RANGE)
        }
    }

    /// Sets the LFO wave shape by numeric index, updating the oscillator if changed.
    pub fn set_wave_shape(&mut self, wave_shape_index: u8) {
        if wave_shape_index != self.wave_shape_index {
            let wave_shape = WaveShape::from_index(wave_shape_index);
            self.oscillator.set_wave_shape(wave_shape);
            self.wave_shape_index = wave_shape_index;
        }
    }

    /// Sets the LFO phase offset, updating the oscillator if the value changed.
    pub fn set_phase(&mut self, phase: f32) {
        if !f32s_are_equal(self.phase, phase) {
            self.oscillator.set_phase(phase);
            self.phase = phase;
        }
    }

    // The default phase is a hand set value of 0.0 so it will never exceed f32::MAX
    #[allow(clippy::cast_possible_truncation)]
    fn reset(&mut self) {
        self.phase = DEFAULT_PHASE as f32;
        self.oscillator.reset();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const SAMPLE_RATE: u32 = 48000;

    // Tests for generate() - early return branching
    #[test]
    fn test_generate_returns_zero_when_range_is_zero() {
        let mut lfo = Lfo::new(SAMPLE_RATE);
        lfo.set_frequency(1.0);
        lfo.set_range(0.0);
        lfo.set_center_value(0.5);

        let actual = lfo.generate(None);
        let expected = 0.0;

        assert!(f32s_are_equal(actual, expected));
    }

    // Tests for set_frequency() - clamping logic
    #[test]
    fn test_set_frequency_clamps_to_minimum() {
        let mut lfo = Lfo::new(SAMPLE_RATE);
        let below_min = Defaults::MIN_LFO_FREQUENCY - 1.0;

        lfo.set_frequency(below_min);
        let actual = lfo.frequency;
        let expected = Defaults::MIN_LFO_FREQUENCY;

        assert!(f32s_are_equal(actual, expected));
    }

    #[test]
    fn test_set_frequency_clamps_to_maximum() {
        let mut lfo = Lfo::new(SAMPLE_RATE);
        let above_max = Defaults::MAX_LFO_FREQUENCY + 1.0;

        lfo.set_frequency(above_max);
        let actual = lfo.frequency;
        let expected = Defaults::MAX_LFO_FREQUENCY;

        assert!(f32s_are_equal(actual, expected));
    }

    // Tests for set_center_value() - clamping logic
    #[test]
    fn test_set_center_value_clamps_to_minimum() {
        let mut lfo = Lfo::new(SAMPLE_RATE);
        let below_min = Defaults::MIN_LFO_CENTER_VALUE - 1.0;

        lfo.set_center_value(below_min);
        let actual = lfo.center_value;
        let expected = Defaults::MIN_LFO_CENTER_VALUE;

        assert!(f32s_are_equal(actual, expected));
    }

    #[test]
    fn test_set_center_value_clamps_to_maximum() {
        let mut lfo = Lfo::new(SAMPLE_RATE);
        let above_max = Defaults::MAX_LFO_CENTER_VALUE + 1.0;

        lfo.set_center_value(above_max);
        let actual = lfo.center_value;
        let expected = Defaults::MAX_LFO_CENTER_VALUE;

        assert!(f32s_are_equal(actual, expected));
    }

    // Tests for set_range() - special zero handling and clamping
    #[test]
    fn test_set_range_zero_returns_zero() {
        let mut lfo = Lfo::new(SAMPLE_RATE);

        lfo.set_range(0.0);
        let actual = lfo.range;
        let expected = 0.0;

        assert!(f32s_are_equal(actual, expected));
    }

    #[test]
    fn test_set_range_clamps_to_minimum() {
        let mut lfo = Lfo::new(SAMPLE_RATE);
        let below_min = Defaults::MIN_LFO_RANGE - 0.1;

        lfo.set_range(below_min);
        let actual = lfo.range;
        let expected = Defaults::MIN_LFO_RANGE;

        assert!(f32s_are_equal(actual, expected));
    }

    #[test]
    fn test_set_range_clamps_to_maximum() {
        let mut lfo = Lfo::new(SAMPLE_RATE);
        let above_max = Defaults::MAX_LFO_RANGE + 1.0;

        lfo.set_range(above_max);
        let actual = lfo.range;
        let expected = Defaults::MAX_LFO_RANGE;

        assert!(f32s_are_equal(actual, expected));
    }

    // Tests for set_wave_shape() - conditional update logic
    #[test]
    fn test_set_wave_shape_updates_when_changed() {
        let mut lfo = Lfo::new(SAMPLE_RATE);
        let initial_wave_shape = WaveShape::Sine as u8;
        let new_wave_shape = WaveShape::Square as u8;

        assert_eq!(lfo.wave_shape_index, initial_wave_shape);

        lfo.set_wave_shape(new_wave_shape);
        let actual = lfo.wave_shape_index;
        let expected = new_wave_shape;

        assert_eq!(actual, expected);
    }

    #[test]
    fn test_set_wave_shape_does_not_update_when_same() {
        let mut lfo = Lfo::new(SAMPLE_RATE);
        let wave_shape = WaveShape::Sine as u8;

        lfo.set_wave_shape(wave_shape);
        let actual = lfo.wave_shape_index;
        let expected = wave_shape;

        assert_eq!(actual, expected);
    }

    // Tests for set_phase() - conditional update logic
    #[test]
    fn test_set_phase_updates_when_changed() {
        let mut lfo = Lfo::new(SAMPLE_RATE);
        let initial_phase = Defaults::LFO_PHASE;
        let new_phase = 1.5;

        assert!(f32s_are_equal(lfo.phase, initial_phase));

        lfo.set_phase(new_phase);
        let actual = lfo.phase;
        let expected = new_phase;

        assert!(f32s_are_equal(actual, expected));
    }

    #[test]
    fn test_set_phase_does_not_update_when_same() {
        let mut lfo = Lfo::new(SAMPLE_RATE);
        let phase = 1.0;

        lfo.set_phase(phase);
        lfo.set_phase(phase); // Set again with same value

        let actual = lfo.phase;
        let expected = phase;

        assert!(f32s_are_equal(actual, expected));
    }
}
