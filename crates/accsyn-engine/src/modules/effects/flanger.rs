use crate::modules::effects;
use crate::modules::effects::constants::{
    DELAY_SMOOTHING_FACTOR, FLANGER_DEFAULT_DELAY_MILLISECONDS, FLANGER_DEFAULT_LFO_RANGE,
    FLANGER_FEEDBACK_SCALE_FACTOR, FLANGER_LFO_CENTER_VALUE, FLANGER_LFO_FREQUENCY_COEFFICIENT,
    FLANGER_LFO_FREQUENCY_SCALE_FACTOR, FLANGER_LFO_PHASE, MAX_FLANGER_DELAY_SAMPLES,
};
use crate::modules::effects::wet_dry_blend;
use crate::modules::lfo::Lfo;
use accsyn_core::casting::f32_to_usize_clamped;
use accsyn_core::effects::{AudioEffect, EffectParameters};
use accsyn_core::math::{exponential_curve_from_normal_value_and_coefficient, f32s_are_equal};

// Ensure MAX_DELAY_SAMPLES is a power of 2 to guarantee the bitwise wrapping logic is safe
const _: () = assert!(MAX_FLANGER_DELAY_SAMPLES.is_power_of_two());

pub struct Flanger {
    buffer: Vec<(f32, f32)>,
    write_index: usize,
    is_enabled: bool,
    samples_count: f32,
    delay_center: f32,
    sample_rate: f32,
    lfo: Lfo,
    rate: f32,
}

impl Flanger {
    pub fn new(sample_rate: u32) -> Self {
        log::debug!(target: "synthesizer::effects::flanger", "Constructing Delay Effect Module");

        let buffer = vec![(0.0, 0.0); MAX_FLANGER_DELAY_SAMPLES as usize];

        let mut lfo = Lfo::new(sample_rate);
        lfo.set_center_value(FLANGER_LFO_CENTER_VALUE);
        lfo.set_range(FLANGER_DEFAULT_LFO_RANGE);
        lfo.set_phase(FLANGER_LFO_PHASE);

        // Sample rate is always ≤ 192_000, within f32 precision (2²³ = 8_388_608)
        #[allow(clippy::cast_precision_loss)]
        let sample_rate = sample_rate as f32;
        let delay_center_value = delay_center_value(sample_rate);

        Self {
            buffer,
            write_index: 0,
            is_enabled: false,
            delay_center: delay_center_value,
            samples_count: delay_center_value,
            sample_rate,
            lfo,
            rate: 0.0,
        }
    }

    fn reset(&mut self) {
        self.buffer.fill((0.0, 0.0));
        self.write_index = 0;
        let current_flanger_samples = delay_center_value(self.sample_rate);
        self.samples_count = current_flanger_samples;
        self.lfo.set_phase(FLANGER_LFO_PHASE);
    }

    fn delayed_samples_from_buffer(
        &mut self,
        buffer_len: usize,
        samples_count: f32,
        new_samples_count: usize,
    ) -> (f32, f32) {
        // samples_count is bounded by MAX_FLANGER_DELAY_SAMPLES (a small constant), within f32 precision (2²³ = 8_388_608)
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

impl AudioEffect for Flanger {
    fn process_samples(&mut self, samples: (f32, f32), effect: &EffectParameters) -> (f32, f32) {
        if self.is_enabled && !effect.is_enabled {
            self.reset();
            self.is_enabled = false;
            return samples;
        }

        if !effect.is_enabled {
            return samples;
        }

        self.is_enabled = effect.is_enabled;

        let buffer_len = self.buffer.len();
        let depth = effect.parameters[0] * FLANGER_DEFAULT_LFO_RANGE;
        let rate = effect.parameters[1];
        let feedback = effect.parameters[2] * FLANGER_FEEDBACK_SCALE_FACTOR;
        let blend = effect.parameters[3];

        self.lfo.set_range(depth);

        if !f32s_are_equal(rate, self.rate) {
            self.rate = rate;
            let exponential_frequency = exponential_curve_from_normal_value_and_coefficient(
                rate,
                FLANGER_LFO_FREQUENCY_COEFFICIENT,
            ) / FLANGER_LFO_FREQUENCY_SCALE_FACTOR;

            self.lfo.set_frequency(exponential_frequency);
        }

        // Flanger delay line
        let delay_offset_right = self.lfo.generate(None);
        let target_delay_right = self.delay_center + (self.delay_center * delay_offset_right);
        let target_delay_samples_right =
            (target_delay_right - self.samples_count) * DELAY_SMOOTHING_FACTOR;
        self.samples_count += target_delay_samples_right;
        // Clamped locally (not just relying on upstream parameter/LFO-range invariants) so a
        // future change to those invariants can't reintroduce an out-of-bounds buffer offset.
        let new_samples_count_right =
            f32_to_usize_clamped(self.samples_count.floor()).min(buffer_len - 1);
        let flanged_samples = self.delayed_samples_from_buffer(
            buffer_len,
            self.samples_count,
            new_samples_count_right,
        );

        self.buffer[self.write_index] = (
            samples.0 + flanged_samples.0 * feedback,
            samples.1 + flanged_samples.1 * feedback,
        );

        self.write_index = (self.write_index + 1) & (buffer_len - 1);

        wet_dry_blend(samples, flanged_samples, blend)
    }
}

fn delay_center_value(sample_rate: f32) -> f32 {
    ((FLANGER_DEFAULT_DELAY_MILLISECONDS / 1000.0) * sample_rate).round()
}

#[cfg(test)]
mod tests {
    use super::*;
    use accsyn_core::math::f32s_are_equal;

    #[test]
    fn flanger_process_samples_returns_original_when_disabled() {
        let sample_rate = 48_000;
        let mut flanger = Flanger::new(sample_rate);
        let effect = EffectParameters {
            name: String::new(),
            is_enabled: false,
            parameters: vec![0.5, 0.5, 0.5, 0.0],
        };
        let input = (0.7, -0.4);

        let result = flanger.process_samples(input, &effect);

        assert!(f32s_are_equal(result.0, 0.7));
        assert!(f32s_are_equal(result.1, -0.4));
    }

    #[test]
    fn flanger_process_samples_fills_buffer_when_buffer_not_full() {
        let sample_rate = 48_000;
        let mut flanger = Flanger::new(sample_rate);
        let effect = EffectParameters {
            name: String::new(),
            is_enabled: true,
            parameters: vec![0.5, 0.5, 0.5, 0.0],
        };
        let input = (0.5, 0.5);

        // Buffer starts pre-allocated
        assert_eq!(flanger.buffer.len(), MAX_FLANGER_DELAY_SAMPLES as usize);
        assert_eq!(flanger.write_index, 0);

        // Process first sample
        let _result = flanger.process_samples(input, &effect);

        // Write index should have advanced
        assert_eq!(flanger.write_index, 1);
    }

    #[test]
    fn flanger_process_samples_resets_buffer_when_disabled_after_enabled() {
        let sample_rate = 48_000;
        let mut flanger = Flanger::new(sample_rate);

        // Enable and process samples to fill buffer
        let enabled_effect = EffectParameters {
            name: String::new(),
            is_enabled: true,
            parameters: vec![0.5, 0.5, 0.5, 0.0],
        };
        flanger.process_samples((0.5, 0.5), &enabled_effect);
        assert_eq!(flanger.write_index, 1);
        assert!(flanger.is_enabled);

        // Disable effect - should trigger reset
        let disabled_effect = EffectParameters {
            name: String::new(),
            is_enabled: false,
            parameters: vec![0.5, 0.5, 0.5, 0.0],
        };
        flanger.process_samples((0.5, 0.5), &disabled_effect);

        // Buffer should be reset (cleared to zeros and write_index reset)
        assert_eq!(flanger.write_index, 0);
        assert!(!flanger.is_enabled);
        // Verify buffer was cleared
        assert!(
            flanger
                .buffer
                .iter()
                .all(|&s| f32s_are_equal(s.0, 0.0) && f32s_are_equal(s.1, 0.0))
        );
    }

    #[test]
    fn flanger_process_samples_does_not_buffer_when_disabled() {
        let sample_rate = 48_000;
        let mut flanger = Flanger::new(sample_rate);

        // Start disabled
        assert!(!flanger.is_enabled);
        let initial_write_index = flanger.write_index;

        // Process with disabled effect - should not modify write index
        let disabled_effect = EffectParameters {
            name: String::new(),
            is_enabled: false,
            parameters: vec![0.5, 0.5, 0.5, 0.0],
        };
        flanger.process_samples((0.5, 0.5), &disabled_effect);

        // Should still be disabled with unchanged write index
        assert!(!flanger.is_enabled);
        assert_eq!(flanger.write_index, initial_write_index);
    }

    #[test]
    fn flanger_process_samples_returns_original_when_buffer_filling() {
        let sample_rate = 48_000;
        let mut flanger = Flanger::new(sample_rate);
        let effect = EffectParameters {
            name: String::new(),
            is_enabled: true,
            parameters: vec![0.5, 0.5, 0.5, 0.0],
        };
        let input = (0.8, -0.6);

        // First sample should return original since buffer isn't full
        let result = flanger.process_samples(input, &effect);

        assert!(f32s_are_equal(result.0, 0.8));
        assert!(f32s_are_equal(result.1, -0.6));
    }

    #[test]
    fn delayed_sample_from_buffer_interpolates_seeded_buffer_values() {
        let mut flanger = Flanger::new(48_000);
        flanger.write_index = 5;
        let buffer_len = flanger.buffer.len();
        // read_index = (5 + buffer_len - 3) & (buffer_len - 1) == 2, read_index_next == 1
        flanger.buffer[2] = (0.4, -0.2);
        flanger.buffer[1] = (0.8, -0.6);

        let result = flanger.delayed_samples_from_buffer(buffer_len, 3.25, 3);

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
    fn delay_center_value_computes_samples_for_sample_rate() {
        let expected_48k = 24.0;
        let result_48k = delay_center_value(48_000.0);
        assert!(
            f32s_are_equal(result_48k, expected_48k),
            "Expected {expected_48k}, got {result_48k}"
        );

        let expected_44k = 22.0;
        let result_44k = delay_center_value(44_100.0);
        assert!(
            f32s_are_equal(result_44k, expected_44k),
            "Expected {expected_44k}, got {result_44k}"
        );
    }

    #[test]
    fn flanger_reset_restores_delay_center_value_not_min_delay_samples() {
        let mut flanger = Flanger::new(48_000);
        let delay_center_value = flanger.delay_center;

        let enabled_effect = EffectParameters {
            name: String::new(),
            is_enabled: true,
            parameters: vec![1.0, 0.8, 0.0, 1.0],
        };
        for _ in 0..500 {
            flanger.process_samples((0.3, -0.3), &enabled_effect);
        }
        // Confirm the voices actually drifted away from delay_center_value before disabling
        assert!(!f32s_are_equal(flanger.samples_count, delay_center_value));

        let disabled_effect = EffectParameters {
            name: String::new(),
            is_enabled: false,
            parameters: vec![1.0, 0.8, 0.0, 1.0],
        };
        flanger.process_samples((0.0, 0.0), &disabled_effect);

        assert!(
            f32s_are_equal(flanger.samples_count, delay_center_value),
            "Expected {}, got {}",
            delay_center_value,
            flanger.samples_count
        );
    }

    #[test]
    fn flanger_process_samples_tracks_rate_changes() {
        let mut flanger = Flanger::new(48_000);
        let mut effect = EffectParameters {
            name: String::new(),
            is_enabled: true,
            parameters: vec![0.5, 0.2, 0.0, 0.0],
        };

        flanger.process_samples((0.0, 0.0), &effect);
        assert!(f32s_are_equal(flanger.rate, 0.2));

        effect.parameters[1] = 0.9;
        flanger.process_samples((0.0, 0.0), &effect);
        assert!(f32s_are_equal(flanger.rate, 0.9));

        // Processing again with an unchanged rate should leave the tracked rate as-is
        let previous_rate = flanger.rate;
        flanger.process_samples((0.0, 0.0), &effect);
        assert!(f32s_are_equal(flanger.rate, previous_rate));
    }
}
