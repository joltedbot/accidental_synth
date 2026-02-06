use crate::modules::effects::constants::{
    DELAY_SMOOTHING_FACTOR, MAX_DELAY_SAMPLES, MIN_DELAY_SAMPLES,
};
use crate::modules::effects::{AudioEffect, EffectParameters};
use crate::synthesizer::midi_value_converters::normal_value_to_unsigned_integer_range;

// Ensure MAX_DELAY_SAMPLES is a power of 2 to guarantee the bitwise wrapping logic is safe
const _: () = assert!(MAX_DELAY_SAMPLES.is_power_of_two());

pub struct Delay {
    buffer: Vec<(f32, f32)>,
    write_index: usize,
    is_enabled: bool,
    current_delay_samples: f32,
}

impl Delay {
    pub fn new() -> Self {
        log::debug!("Constructing Delay Effect Module");

        let buffer = vec![(0.0, 0.0); MAX_DELAY_SAMPLES as usize];

        Self {
            buffer,
            write_index: 0,
            is_enabled: false,
            current_delay_samples: MIN_DELAY_SAMPLES as f32,
        }
    }

    fn reset(&mut self) {
        self.buffer.fill((0.0, 0.0));
        self.write_index = 0;
        self.current_delay_samples = MIN_DELAY_SAMPLES as f32;
    }
}

impl AudioEffect for Delay {
    fn process_samples(&mut self, samples: (f32, f32), effect: &EffectParameters) -> (f32, f32) {
        if self.is_enabled && !effect.is_enabled {
            self.reset();
        }

        self.is_enabled = effect.is_enabled;

        if !self.is_enabled {
            return samples;
        }

        let amount = effect.parameters[0];
        let feedback = effect.parameters[2];
        let target_delay = normal_value_to_unsigned_integer_range(
            effect.parameters[1],
            MIN_DELAY_SAMPLES,
            MAX_DELAY_SAMPLES,
        ) as f32;

        self.current_delay_samples +=
            (target_delay - self.current_delay_samples) * DELAY_SMOOTHING_FACTOR;
        let delay_samples = self.current_delay_samples.floor() as usize;
        let fractional_index = self.current_delay_samples - delay_samples as f32;

        let buffer_len = self.buffer.len();

        let read_index = (self.write_index + buffer_len - delay_samples) & (buffer_len - 1);
        let read_index_next = (read_index + buffer_len - 1) & (buffer_len - 1);

        let buffer_samples_a = self.buffer[read_index];
        let buffer_samples_b = self.buffer[read_index_next];

        let mut interpolated_left =
            buffer_samples_a.0 * (1.0 - fractional_index) + buffer_samples_b.0 * fractional_index;
        let mut interpolated_right =
            buffer_samples_a.1 * (1.0 - fractional_index) + buffer_samples_b.1 * fractional_index;

        if interpolated_left.is_nan() || interpolated_right.is_nan() {
            interpolated_left = buffer_samples_a.0;
            interpolated_right = buffer_samples_a.1;
        }

        if interpolated_left.is_infinite() || interpolated_right.is_infinite() {
            interpolated_left = buffer_samples_a.0;
            interpolated_right = buffer_samples_a.1;
        }

        let delayed_samples = (interpolated_left * feedback, interpolated_right * feedback);

        self.buffer[self.write_index] =
            (samples.0 + delayed_samples.0, samples.1 + delayed_samples.1);
        self.write_index = (self.write_index + 1) & (buffer_len - 1);

        (
            samples.0 + delayed_samples.0 * amount,
            samples.1 + delayed_samples.1 * amount,
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use accsyn_types::math::f32s_are_equal;

    #[test]
    fn delay_process_samples_returns_original_when_disabled() {
        let mut delay = Delay::new();
        let effect = EffectParameters {
            is_enabled: false,
            parameters: vec![0.5, 0.5, 0.5, 0.0],
        };
        let input = (0.7, -0.4);

        let result = delay.process_samples(input, &effect);

        assert!(f32s_are_equal(result.0, 0.7));
        assert!(f32s_are_equal(result.1, -0.4));
    }

    #[test]
    fn delay_process_samples_fills_buffer_when_buffer_not_full() {
        let mut delay = Delay::new();
        let effect = EffectParameters {
            is_enabled: true,
            parameters: vec![0.5, 0.5, 0.5, 0.0],
        };
        let input = (0.5, 0.5);

        // Buffer starts pre-allocated
        assert_eq!(delay.buffer.len(), MAX_DELAY_SAMPLES as usize);
        assert_eq!(delay.write_index, 0);

        // Process first sample
        let _result = delay.process_samples(input, &effect);

        // Write index should have advanced
        assert_eq!(delay.write_index, 1);
    }

    #[test]
    fn delay_process_samples_resets_buffer_when_disabled_after_enabled() {
        let mut delay = Delay::new();

        // Enable and process samples to fill buffer
        let enabled_effect = EffectParameters {
            is_enabled: true,
            parameters: vec![0.5, 0.5, 0.5, 0.0],
        };
        delay.process_samples((0.5, 0.5), &enabled_effect);
        assert_eq!(delay.write_index, 1);
        assert!(delay.is_enabled);

        // Disable effect - should trigger reset
        let disabled_effect = EffectParameters {
            is_enabled: false,
            parameters: vec![0.5, 0.5, 0.5, 0.0],
        };
        delay.process_samples((0.5, 0.5), &disabled_effect);

        // Buffer should be reset (cleared to zeros and write_index reset)
        assert_eq!(delay.write_index, 0);
        assert!(!delay.is_enabled);
        // Verify buffer was cleared
        assert!(
            delay
                .buffer
                .iter()
                .all(|&s| f32s_are_equal(s.0, 0.0) && f32s_are_equal(s.1, 0.0))
        );
    }

    #[test]
    fn delay_process_samples_does_not_buffer_when_disabled() {
        let mut delay = Delay::new();

        // Start disabled
        assert!(!delay.is_enabled);
        let initial_write_index = delay.write_index;

        // Process with disabled effect - should not modify write index
        let disabled_effect = EffectParameters {
            is_enabled: false,
            parameters: vec![0.5, 0.5, 0.5, 0.0],
        };
        delay.process_samples((0.5, 0.5), &disabled_effect);

        // Should still be disabled with unchanged write index
        assert!(!delay.is_enabled);
        assert_eq!(delay.write_index, initial_write_index);
    }

    #[test]
    fn delay_process_samples_returns_original_when_buffer_filling() {
        let mut delay = Delay::new();
        let effect = EffectParameters {
            is_enabled: true,
            parameters: vec![0.5, 0.5, 0.5, 0.0],
        };
        let input = (0.8, -0.6);

        // First sample should return original since buffer isn't full
        let result = delay.process_samples(input, &effect);

        assert!(f32s_are_equal(result.0, 0.8));
        assert!(f32s_are_equal(result.1, -0.6));
    }
}
