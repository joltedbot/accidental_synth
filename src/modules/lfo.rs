use crate::math::{f32s_are_equal, load_f32_from_atomic_u32};
use crate::modules::oscillator::{Oscillator, WaveShape};
use std::default::Default;
use std::sync::atomic::Ordering::Relaxed;
use std::sync::atomic::{AtomicBool, AtomicU8, AtomicU32};

pub const MIN_LFO_FREQUENCY: f32 = 0.01;
pub const MAX_LFO_FREQUENCY: f32 = 20000.0;
pub const MAX_LFO_RANGE: f32 = 2.0;
pub const MIN_LFO_RANGE: f32 = 0.01;
pub const MAX_LFO_CENTER_VALUE: f32 = 1.0;
pub const MIN_LFO_CENTER_VALUE: f32 = -1.0;
const DEFAULT_CENTER_VALUE: f32 = 0.0;
const DEFAULT_RANGE: f32 = 2.0;
pub const DEFAULT_LFO_PHASE: f32 = 0.0;
pub const DEFAULT_LFO_FREQUENCY: f32 = 0.1;

#[derive(Debug)]
pub struct LfoParameters {
    pub frequency: AtomicU32,
    pub center_value: AtomicU32,
    pub range: AtomicU32,
    pub phase: AtomicU32,
    pub wave_shape: AtomicU8,
    pub reset: AtomicBool,
}

impl Default for LfoParameters {
    fn default() -> Self {
        Self {
            frequency: AtomicU32::new(DEFAULT_LFO_FREQUENCY.to_bits()),
            center_value: AtomicU32::new(DEFAULT_CENTER_VALUE.to_bits()),
            range: AtomicU32::new(DEFAULT_RANGE.to_bits()),
            phase: AtomicU32::new(DEFAULT_LFO_PHASE.to_bits()),
            wave_shape: AtomicU8::new(WaveShape::default() as u8),
            reset: AtomicBool::new(false),
        }
    }
}

pub struct Lfo {
    oscillator: Oscillator,
    frequency: f32,
    center_value: f32,
    wave_shape_index: u8,
    range: f32,
    phase: f32,
}

impl Lfo {
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

    pub fn set_parameters(&mut self, parameters: &LfoParameters) {
        self.set_frequency(load_f32_from_atomic_u32(&parameters.frequency));
        self.set_center_value(load_f32_from_atomic_u32(&parameters.center_value));
        self.set_range(load_f32_from_atomic_u32(&parameters.range));
        self.set_phase(load_f32_from_atomic_u32(&parameters.phase));
        self.set_wave_shape(parameters.wave_shape.load(Relaxed));
        if parameters.reset.load(Relaxed) {
            self.reset();
            parameters.reset.store(false, Relaxed);
        }
    }

    pub fn generate(&mut self, modulation: Option<f32>) -> f32 {
        if self.range == 0.0 || self.frequency == 0.0 {
            return 0.0;
        }
        let wave_sample = self.oscillator.generate(modulation);
        self.center_value + (wave_sample * (self.range / 2.0))
    }

    pub fn set_frequency(&mut self, frequency: f32) {
        self.frequency = frequency.clamp(MIN_LFO_FREQUENCY, MAX_LFO_FREQUENCY);
        self.oscillator.set_frequency(self.frequency);
    }

    pub fn set_center_value(&mut self, center_value: f32) {
        self.center_value = center_value.clamp(MIN_LFO_CENTER_VALUE, MAX_LFO_CENTER_VALUE);
    }

    pub fn set_range(&mut self, range: f32) {
        self.range = if range == 0.0 {
            0.0
        } else {
            range.clamp(MIN_LFO_RANGE, MAX_LFO_RANGE)
        }
    }

    pub fn set_wave_shape(&mut self, wave_shape_index: u8) {
        if wave_shape_index != self.wave_shape_index {
            let wave_shape = WaveShape::from_index(wave_shape_index);
            self.oscillator.set_wave_shape(wave_shape);
            self.wave_shape_index = wave_shape_index;
        }
    }

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
