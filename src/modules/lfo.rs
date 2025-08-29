use crate::modules::oscillator::{Oscillator, WaveShape};
use std::default::Default;
use std::sync::atomic::Ordering::Relaxed;
use std::sync::atomic::{AtomicBool, AtomicU8, AtomicU32};

pub const MIN_LFO_FREQUENCY: f32 = 0.01;
pub const MAX_LFO_FREQUENCY: f32 = 20000.0;
pub const MAX_LFO_RANGE: f32 = 2.0;
pub const MIN_LFO_RANGE: f32 = 0.01;
const DEFAULT_CENTER_VALUE: f32 = 0.0;
const MAX_CENTER_VALUE: f32 = 1.0;
const MIN_CENTER_VALUE: f32 = -1.0;
const DEFAULT_RANGE: f32 = 2.0;
const DEFAULT_PHASE: f32 = 0.0;
const DEFAULT_FREQUENCY: f32 = 0.1;

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
            frequency: AtomicU32::new(DEFAULT_FREQUENCY.to_bits()),
            center_value: AtomicU32::new(DEFAULT_CENTER_VALUE.to_bits()),
            range: AtomicU32::new(DEFAULT_RANGE.to_bits()),
            phase: AtomicU32::new(DEFAULT_PHASE.to_bits()),
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
        let oscillator = Oscillator::new(sample_rate, WaveShape::Sine);
        Self {
            oscillator,
            frequency: DEFAULT_FREQUENCY,
            center_value: DEFAULT_CENTER_VALUE,
            range: DEFAULT_RANGE,
            phase: DEFAULT_PHASE,
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
        self.center_value = center_value.clamp(MIN_CENTER_VALUE, MAX_CENTER_VALUE);
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
        if self.phase != phase {
            self.oscillator.set_phase(phase);
            self.phase = phase;
        }
    }

    fn reset(&mut self) {
        self.oscillator.reset();
    }
}

pub fn load_f32_from_atomic_u32(atomic: &AtomicU32) -> f32 {
    f32::from_bits(atomic.load(Relaxed))
}
