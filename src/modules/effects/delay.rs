use std::collections::VecDeque;
use crate::modules::effects::{AudioEffect, EffectParameters};
use crate::modules::effects::constants::{DELAY_BUFFER_SIZE, MAX_DELAY_SAMPLES, MIN_DELAY_SAMPLES};
use crate::synthesizer::midi_value_converters::normal_value_to_unsigned_integer_range;

pub struct Delay {
    buffer: VecDeque<(f32, f32)>,
    is_enabled: bool,
}

impl Delay {
    pub fn new() -> Self {
        let buffer = VecDeque::new();

        Self {
            buffer,
            is_enabled: false
        }
    }

    fn reset(&mut self) {
        self.buffer = VecDeque::new();
    }
}

impl AudioEffect for Delay {
    fn process_samples(&mut self, samples: (f32, f32), effect: &EffectParameters) -> (f32, f32) {

        if self.is_enabled && !effect.is_enabled {
            self.reset();
        }

        self.is_enabled = effect.is_enabled;

        if !self.is_enabled || self.buffer.len() < (MAX_DELAY_SAMPLES - 1) as usize {
            self.buffer.push_front(samples);
            return samples;
        }


        let amount = effect.parameters[0];
        let delay =  normal_value_to_unsigned_integer_range(effect.parameters[1], MIN_DELAY_SAMPLES, MAX_DELAY_SAMPLES);
        let decay_factor = effect.parameters[2];


        let buffer_samples = self.buffer[delay as usize];

        if buffer_samples.0.is_nan() || buffer_samples.1.is_nan() {
            println!("NaN detected in delay buffer");
        }

        if buffer_samples.0.is_infinite() || buffer_samples.1.is_infinite() {
            println!("Infinity detected in delay buffer");
        }

        let delayed_samples = (buffer_samples.0 * decay_factor, buffer_samples.1 * decay_factor);

        if self.buffer.len() >= MAX_DELAY_SAMPLES as usize {
            let _throw_away_old_samples = self.buffer.pop_back();
        }
        self.buffer.push_front((samples.0 + delayed_samples.0 , samples.1 + delayed_samples.1));

        (samples.0 + delayed_samples.0 * amount, samples.1 + delayed_samples.1 * amount)

    }
}
