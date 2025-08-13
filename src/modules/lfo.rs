#![allow(dead_code)]

use crate::modules::oscillator::{Oscillator, WaveShape};

const DEFAULT_CENTER_VALUE: f32 = 0.0;
const MAX_CENTER_VALUE: f32 = 1.0;
const MIN_CENTER_VALUE: f32 = -1.0;
const DEFAULT_RANGE: f32 = 2.0;
const MAX_RANGE: f32 = 2.0;
const MIN_RANGE: f32 = 0.001;
const DEFAULT_PHASE: f32 = 0.0;
const DEFAULT_FREQUENCY: f32 = 0.1;

pub struct LFO {
    sample_rate: u32,
    oscillator: Oscillator,
    frequency: f32,
    center_value: f32,
    range: f32,
    phase: f32,
}

impl LFO {
    pub fn new(sample_rate: u32) -> Self {
        let oscillator = Oscillator::new(sample_rate, WaveShape::Sine);
        Self {
            sample_rate,
            oscillator,
            frequency: DEFAULT_FREQUENCY,
            center_value: DEFAULT_CENTER_VALUE,
            range: DEFAULT_RANGE,
            phase: DEFAULT_PHASE,
        }
    }

    pub fn generate(&mut self) -> f32 {
        let wave_position = self.oscillator.generate(self.frequency, None);
        self.center_value + (wave_position * (self.range / 2.0))
    }

    pub fn set_frequency(&mut self, frequency: f32) {
        self.frequency = frequency;
    }

    pub fn set_center_value(&mut self, center_value: f32) {
        self.center_value = center_value.clamp(MIN_CENTER_VALUE, MAX_CENTER_VALUE);
    }

    pub fn set_range(&mut self, range: f32) {
        self.range = range.clamp(MIN_RANGE, MAX_RANGE);
    }

    pub fn set_wave_shape(&mut self, wave_shape: WaveShape) {
        self.oscillator.set_wave_shape(wave_shape);
    }

    pub fn set_phase(&mut self, phase: f32) {
        self.oscillator.set_phase(phase);
    }

    pub fn reset(&mut self) {
        self.oscillator.reset();
    }
}
