use crate::modules::oscillator::{Oscillator, WaveShape};
use accsyn_types::math::f32s_are_equal;
use serde::{Deserialize, Serialize};
use std::default::Default;
use std::sync::atomic::Ordering::Relaxed;
use std::sync::atomic::{AtomicBool, AtomicU8};
use accsyn_types::parameter_types::{Balance, Hertz, LfoRange, NormalizedValue};

/// Minimum LFO frequency in Hz.
pub const MIN_LFO_FREQUENCY: f32 = 0.01;
/// Maximum LFO frequency in Hz.
pub const MAX_LFO_FREQUENCY: f32 = 20000.0;
/// Maximum LFO output range (peak-to-peak).
pub const MAX_LFO_RANGE: f32 = 2.0;
/// Minimum LFO output range (peak-to-peak).
pub const MIN_LFO_RANGE: f32 = 0.01;
/// Maximum LFO center value (bipolar offset).
pub const MAX_LFO_CENTER_VALUE: f32 = 1.0;
/// Minimum LFO center value (bipolar offset).
pub const MIN_LFO_CENTER_VALUE: f32 = -1.0;
const DEFAULT_CENTER_VALUE: f32 = 0.0;
const DEFAULT_RANGE: f32 = 2.0;
/// Default LFO starting phase.
pub const DEFAULT_LFO_PHASE: f32 = 0.0;
/// Default LFO frequency in Hz.
pub const DEFAULT_LFO_FREQUENCY: f32 = 0.1;

/// Shared atomic parameters for controlling an LFO from the UI thread.
#[derive(Debug, Serialize, Deserialize)]
pub struct LfoParameters {
    /// LFO oscillation frequency in Hz.
    pub frequency: Hertz,
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
}

impl LfoParameters {
    /// Replace all the values in these `LfoParameters` with the values from the provided `LfoParameters`.
    pub fn assign_from(&self, parameters: &LfoParameters) {
        self.frequency.store(parameters.frequency.load());
        self.center_value.store(parameters.center_value.load());
        self.range.store(parameters.range.load());
        self.phase.store(parameters.phase.load());
        self.wave_shape.store(parameters.wave_shape.load(Relaxed), Relaxed);
        self.reset.store(parameters.reset.load(Relaxed), Relaxed)
    }
}

impl Default for LfoParameters {
    fn default() -> Self {
        Self {
            frequency: Hertz::new(DEFAULT_LFO_FREQUENCY),
            center_value: Balance::new(DEFAULT_CENTER_VALUE),
            range: LfoRange::new(DEFAULT_RANGE),
            phase: NormalizedValue::new(DEFAULT_LFO_PHASE),
            wave_shape: AtomicU8::new(WaveShape::default() as u8),
            reset: AtomicBool::new(false),
        }
    }
}

/// Low-frequency oscillator for modulating synthesis parameters.
pub struct Lfo {
    oscillator: Oscillator,
    frequency: f32,
    center_value: f32,
    wave_shape_index: u8,
    range: f32,
    phase: f32,
}

impl Lfo {
    /// Creates a new LFO with default frequency, range, and sine wave shape.
    pub fn new(sample_rate: u32) -> Self {
        log::debug!("Constructing LFO Module");
        let oscillator = Oscillator::new(sample_rate, WaveShape::Sine);
        Self {
            oscillator,
            frequency: DEFAULT_LFO_FREQUENCY,
            center_value: DEFAULT_CENTER_VALUE,
            range: DEFAULT_RANGE,
            phase: DEFAULT_LFO_PHASE,
            wave_shape_index: WaveShape::Sine as u8,
        }
    }

    /// Updates all LFO settings from the shared parameter block.
    pub fn set_parameters(&mut self, parameters: &LfoParameters) {
        self.set_frequency(parameters.frequency.load());
        self.set_center_value(parameters.center_value.load());
        self.set_range(parameters.range.load());
        self.set_phase(parameters.phase.load());
        self.set_wave_shape(parameters.wave_shape.load(Relaxed));
        if parameters.reset.load(Relaxed) {
            self.reset();
            parameters.reset.store(false, Relaxed);
        }
    }

    /// Generates the next LFO output sample, scaled by center value and range.
    pub fn generate(&mut self, modulation: Option<f32>) -> f32 {
        if self.range == 0.0 || self.frequency == 0.0 {
            return 0.0;
        }
        let wave_sample = self.oscillator.generate(modulation);
        self.center_value + (wave_sample * (self.range / 2.0))
    }

    /// Sets the LFO frequency in Hz, clamped to the valid range.
    pub fn set_frequency(&mut self, frequency: f32) {
        self.frequency = frequency.clamp(MIN_LFO_FREQUENCY, MAX_LFO_FREQUENCY);
        self.oscillator.set_frequency(self.frequency);
    }

    /// Sets the LFO center value (DC offset), clamped to [-1, 1].
    pub fn set_center_value(&mut self, center_value: f32) {
        self.center_value = center_value.clamp(MIN_LFO_CENTER_VALUE, MAX_LFO_CENTER_VALUE);
    }

    /// Sets the LFO output range (peak-to-peak), clamped to valid bounds or zero.
    pub fn set_range(&mut self, range: f32) {
        self.range = if range == 0.0 {
            0.0
        } else {
            range.clamp(MIN_LFO_RANGE, MAX_LFO_RANGE)
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

    fn reset(&mut self) {
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
        let below_min = MIN_LFO_FREQUENCY - 1.0;

        lfo.set_frequency(below_min);
        let actual = lfo.frequency;
        let expected = MIN_LFO_FREQUENCY;

        assert!(f32s_are_equal(actual, expected));
    }

    #[test]
    fn test_set_frequency_clamps_to_maximum() {
        let mut lfo = Lfo::new(SAMPLE_RATE);
        let above_max = MAX_LFO_FREQUENCY + 1.0;

        lfo.set_frequency(above_max);
        let actual = lfo.frequency;
        let expected = MAX_LFO_FREQUENCY;

        assert!(f32s_are_equal(actual, expected));
    }

    // Tests for set_center_value() - clamping logic
    #[test]
    fn test_set_center_value_clamps_to_minimum() {
        let mut lfo = Lfo::new(SAMPLE_RATE);
        let below_min = MIN_LFO_CENTER_VALUE - 1.0;

        lfo.set_center_value(below_min);
        let actual = lfo.center_value;
        let expected = MIN_LFO_CENTER_VALUE;

        assert!(f32s_are_equal(actual, expected));
    }

    #[test]
    fn test_set_center_value_clamps_to_maximum() {
        let mut lfo = Lfo::new(SAMPLE_RATE);
        let above_max = MAX_LFO_CENTER_VALUE + 1.0;

        lfo.set_center_value(above_max);
        let actual = lfo.center_value;
        let expected = MAX_LFO_CENTER_VALUE;

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
        let below_min = MIN_LFO_RANGE - 0.1;

        lfo.set_range(below_min);
        let actual = lfo.range;
        let expected = MIN_LFO_RANGE;

        assert!(f32s_are_equal(actual, expected));
    }

    #[test]
    fn test_set_range_clamps_to_maximum() {
        let mut lfo = Lfo::new(SAMPLE_RATE);
        let above_max = MAX_LFO_RANGE + 1.0;

        lfo.set_range(above_max);
        let actual = lfo.range;
        let expected = MAX_LFO_RANGE;

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
        let initial_phase = DEFAULT_LFO_PHASE;
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
