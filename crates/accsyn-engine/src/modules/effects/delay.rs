use crate::modules::effects;
use crate::modules::effects::constants::{
    DELAY_SMOOTHING_FACTOR, MAX_DELAY_SAMPLES, MIN_DELAY_SAMPLES,
};
use crate::synthesizer::midi_value_converters::normal_value_to_unsigned_integer_range;
use accsyn_core::casting::f32_to_usize_clamped;
use accsyn_core::effects::{AudioEffect, EffectParameters};

// Ensure MAX_DELAY_SAMPLES is a power of 2 to guarantee the bitwise wrapping logic is safe
const _: () = assert!(MAX_DELAY_SAMPLES.is_power_of_two());

pub struct Delay {
    buffer: Vec<(f32, f32)>,
    write_index: usize,
    is_enabled: bool,
    number_of_delay_samples: f32,
}

impl Delay {
    pub fn new() -> Self {
        log::debug!(target: "synthesizer::effects::delay", "Constructing Delay Effect Module");

        let buffer = vec![(0.0, 0.0); MAX_DELAY_SAMPLES as usize];

        // MIN_DELAY_SAMPLES is a small constant (delay samples count), within f32 precision (2²³ = 8_388_608)
        #[allow(clippy::cast_precision_loss)]
        let current_delay_samples = MIN_DELAY_SAMPLES as f32;

        Self {
            buffer,
            write_index: 0,
            is_enabled: false,
            number_of_delay_samples: current_delay_samples,
        }
    }

    fn reset(&mut self) {
        self.buffer.fill((0.0, 0.0));
        self.write_index = 0;
        // MIN_DELAY_SAMPLES is a small constant (delay samples count), within f32 precision (2²³ = 8_388_608)
        #[allow(clippy::cast_precision_loss)]
        let current_delay_samples = MIN_DELAY_SAMPLES as f32;
        self.number_of_delay_samples = current_delay_samples;
    }

    fn delayed_samples_from_buffer(
        &mut self,
        buffer_len: usize,
        samples_count: f32,
        new_samples_count: usize,
    ) -> (f32, f32) {
        // new_samples_count is bounded by MAX_DELAY_SAMPLES (a small constant), within f32 precision (2²³ = 8_388_608)
        #[allow(clippy::cast_precision_loss)]
        let fractional_index1 = samples_count - new_samples_count as f32;
        let read_index = (self.write_index + buffer_len - new_samples_count) & (buffer_len - 1);
        let read_index_next = (read_index + buffer_len - 1) & (buffer_len - 1);
        let buffer_samples = self.buffer[read_index];
        let buffer_samples_next = self.buffer[read_index_next];
        let (interpolated_left, interpolated_right) =
            effects::interpolate_samples(buffer_samples, buffer_samples_next, fractional_index1);
        (interpolated_left, interpolated_right)
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
        // Delay sample count is bounded by MAX_DELAY_SAMPLES, within f32 precision (2²³ = 8_388_608)
        #[allow(clippy::cast_precision_loss)]
        let target_delay = normal_value_to_unsigned_integer_range(
            effect.parameters[1],
            MIN_DELAY_SAMPLES,
            MAX_DELAY_SAMPLES,
        ) as f32;

        let buffer_len = self.buffer.len();
        self.number_of_delay_samples +=
            (target_delay - self.number_of_delay_samples) * DELAY_SMOOTHING_FACTOR;
        let delay_samples = f32_to_usize_clamped(self.number_of_delay_samples.floor());
        let (interpolated_left, interpolated_right) = self.delayed_samples_from_buffer(
            buffer_len,
            self.number_of_delay_samples,
            delay_samples,
        );
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
    use accsyn_core::math::f32s_are_equal;

    #[test]
    fn delay_process_samples_returns_original_when_disabled() {
        let mut delay = Delay::new();
        let effect = EffectParameters {
            name: String::new(),
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
            name: String::new(),
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
            name: String::new(),
            is_enabled: true,
            parameters: vec![0.5, 0.5, 0.5, 0.0],
        };
        delay.process_samples((0.5, 0.5), &enabled_effect);
        assert_eq!(delay.write_index, 1);
        assert!(delay.is_enabled);

        // Disable effect - should trigger reset
        let disabled_effect = EffectParameters {
            name: String::new(),
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
            name: String::new(),
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
            name: String::new(),
            is_enabled: true,
            parameters: vec![0.5, 0.5, 0.5, 0.0],
        };
        let input = (0.8, -0.6);

        // First sample should return original since buffer isn't full
        let result = delay.process_samples(input, &effect);

        assert!(f32s_are_equal(result.0, 0.8));
        assert!(f32s_are_equal(result.1, -0.6));
    }

    #[test]
    fn delayed_samples_from_buffer_interpolates_seeded_buffer_values() {
        let mut delay = Delay::new();
        delay.write_index = 5;
        let buffer_len = delay.buffer.len();
        // read_index = (5 + buffer_len - 3) & (buffer_len - 1) == 2, read_index_next == 1
        delay.buffer[2] = (0.4, -0.2);
        delay.buffer[1] = (0.8, -0.6);

        let result = delay.delayed_samples_from_buffer(buffer_len, 3.25, 3);

        let expected = (0.5, -0.3);
        assert!(
            f32s_are_equal(result.0, expected.0),
            "Expected {}, got {}",
            expected.0,
            result.0
        );
        assert!(
            f32s_are_equal(result.1, expected.1),
            "Expected {}, got {}",
            expected.1,
            result.1
        );
    }

    #[test]
    fn delayed_samples_from_buffer_wraps_read_index_to_end_of_buffer() {
        let mut delay = Delay::new();
        delay.write_index = 0;
        let buffer_len = delay.buffer.len();
        // read_index = (0 + buffer_len - 1) & (buffer_len - 1) == buffer_len - 1, read_index_next == buffer_len - 2
        delay.buffer[buffer_len - 1] = (0.9, -0.1);
        delay.buffer[buffer_len - 2] = (0.1, 0.9);

        let result = delay.delayed_samples_from_buffer(buffer_len, 1.5, 1);

        let expected = (0.5, 0.4);
        assert!(
            f32s_are_equal(result.0, expected.0),
            "Expected {}, got {}",
            expected.0,
            result.0
        );
        assert!(
            f32s_are_equal(result.1, expected.1),
            "Expected {}, got {}",
            expected.1,
            result.1
        );
    }

    #[test]
    fn delay_process_samples_outputs_impulse_after_delay_time_elapses() {
        let mut delay = Delay::new();
        // amount=1.0, position=0.0 (-> MIN_DELAY_SAMPLES), feedback=1.0
        let effect = EffectParameters {
            name: String::new(),
            is_enabled: true,
            parameters: vec![1.0, 0.0, 1.0, 0.0],
        };
        let impulse = (0.9, -0.9);
        let silence = (0.0, 0.0);

        let first_output = delay.process_samples(impulse, &effect);
        assert!(f32s_are_equal(first_output.0, impulse.0));
        assert!(f32s_are_equal(first_output.1, impulse.1));

        let mut last_output = (0.0, 0.0);
        for _ in 0..MIN_DELAY_SAMPLES {
            last_output = delay.process_samples(silence, &effect);
        }

        assert!(
            f32s_are_equal(last_output.0, impulse.0),
            "Expected delayed impulse {}, got {}",
            impulse.0,
            last_output.0
        );
        assert!(
            f32s_are_equal(last_output.1, impulse.1),
            "Expected delayed impulse {}, got {}",
            impulse.1,
            last_output.1
        );
    }
}
