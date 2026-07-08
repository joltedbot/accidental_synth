use crate::modules::effects;
use crate::modules::effects::constants::{
    CHORUS_DEFAULT_DELAY_MILLISECONDS, CHORUS_DEFAULT_LFO_RANGE, CHORUS_FEEDBACK_SCALE_FACTOR,
    CHORUS_LFO_CENTER_VALUE, CHORUS_LFO_FREQUENCY_COEFFICIENT, CHORUS_LFO_FREQUENCY_SCALE_FACTOR,
    CHORUS_LFO2_PHASE, DELAY_SMOOTHING_FACTOR, MAX_CHORUS_DELAY_SAMPLES,
};
use crate::modules::effects::wet_dry_blend;
use crate::modules::lfo::Lfo;
use accsyn_core::casting::f32_to_usize_clamped;
use accsyn_core::defaults::Defaults;
use accsyn_core::effects::{AudioEffect, EffectParameters};
use accsyn_core::math::{exponential_curve_from_normal_value_and_coefficient, f32s_are_equal};

// Ensure MAX_DELAY_SAMPLES is a power of 2 to guarantee the bitwise wrapping logic is safe
const _: () = assert!(MAX_CHORUS_DELAY_SAMPLES.is_power_of_two());

enum Side {
    Left,
    Right,
}

pub struct Chorus {
    buffer: Vec<(f32, f32)>,
    write_index: usize,
    is_enabled: bool,
    samples_count_left: f32,
    samples_count_right: f32,
    delay_center_value: f32,
    sample_rate: f32,
    lfo_left: Lfo,
    lfo_right: Lfo,
    rate: f32,
}

impl Chorus {
    pub fn new(sample_rate: u32) -> Self {
        log::debug!(target: "synthesizer::effects::chorus", "Constructing Delay Effect Module");

        let buffer = vec![(0.0, 0.0); MAX_CHORUS_DELAY_SAMPLES as usize];

        let mut lfo_left = Lfo::new(sample_rate);
        lfo_left.set_center_value(CHORUS_LFO_CENTER_VALUE);
        lfo_left.set_range(CHORUS_DEFAULT_LFO_RANGE);

        let mut lfo_right = Lfo::new(sample_rate);
        lfo_right.set_center_value(CHORUS_LFO_CENTER_VALUE);
        lfo_right.set_range(CHORUS_DEFAULT_LFO_RANGE);
        lfo_right.set_phase(CHORUS_LFO2_PHASE);

        // Sample rate is always ≤ 192_000, within f32 precision (2²³ = 8_388_608)
        #[allow(clippy::cast_precision_loss)]
        let sample_rate = sample_rate as f32;
        let delay_center_value = delay_center_value(sample_rate);

        Self {
            buffer,
            write_index: 0,
            is_enabled: false,
            delay_center_value,
            samples_count_left: delay_center_value,
            samples_count_right: delay_center_value,
            sample_rate,
            lfo_left,
            lfo_right,
            rate: 0.0,
        }
    }

    fn reset(&mut self) {
        self.buffer.fill((0.0, 0.0));
        self.write_index = 0;
        let current_chorus_samples = delay_center_value(self.sample_rate);
        self.samples_count_left = current_chorus_samples;
        self.samples_count_right = current_chorus_samples;
        self.lfo_left.set_phase(0.0);
        self.lfo_right.set_phase(CHORUS_LFO2_PHASE);
    }

    fn delayed_sample_from_buffer(
        &self,
        buffer_len: usize,
        current_samples_count: f32,
        new_samples_count: usize,
        side: Side,
    ) -> f32 {
        let read_index = (self.write_index + buffer_len - new_samples_count) & (buffer_len - 1);
        let read_index_next = (read_index + buffer_len - 1) & (buffer_len - 1);
        let buffer_samples = self.buffer[read_index];
        let buffer_samples_next = self.buffer[read_index_next];

        // new_samples_count is bounded by MAX_DELAY_SAMPLES (a small constant), within f32 precision (2²³ = 8_388_608)
        #[allow(clippy::cast_precision_loss)]
        let fractional_index = current_samples_count - new_samples_count as f32;
        match side {
            Side::Left => effects::interpolate_single_sample(
                buffer_samples.0,
                buffer_samples_next.0,
                fractional_index,
            ),
            Side::Right => effects::interpolate_single_sample(
                buffer_samples.1,
                buffer_samples_next.1,
                fractional_index,
            ),
        }
    }
}

impl AudioEffect for Chorus {
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
        let depth = effect.parameters[0] * CHORUS_DEFAULT_LFO_RANGE;
        let rate = effect.parameters[1];
        let feedback = effect.parameters[2] * CHORUS_FEEDBACK_SCALE_FACTOR;
        let blend = effect.parameters[3];

        self.lfo_left.set_range(depth);
        self.lfo_right.set_range(depth);

        if !f32s_are_equal(rate, self.rate) {
            self.rate = rate;
            let exponential_frequency = exponential_curve_from_normal_value_and_coefficient(
                rate,
                CHORUS_LFO_FREQUENCY_COEFFICIENT,
            ) / CHORUS_LFO_FREQUENCY_SCALE_FACTOR;

            self.lfo_left.set_frequency(exponential_frequency);
            self.lfo_right.set_frequency(exponential_frequency);
        }

        // Chorus Voice L
        let delay_offset_left = self.lfo_left.generate(None);
        let target_delay_left =
            self.delay_center_value + (self.delay_center_value * delay_offset_left);
        let target_delay_samples_left =
            (target_delay_left - self.samples_count_left) * DELAY_SMOOTHING_FACTOR;
        self.samples_count_left += target_delay_samples_left;
        let new_samples_count_left = f32_to_usize_clamped(self.samples_count_left.floor());
        let chorused_samples_left = self.delayed_sample_from_buffer(
            buffer_len,
            self.samples_count_left,
            new_samples_count_left,
            Side::Left,
        );

        // Chorus Voice R
        let delay_offset_right = self.lfo_right.generate(None);
        let target_delay_right =
            self.delay_center_value + (self.delay_center_value * delay_offset_right);
        let target_delay_samples_right =
            (target_delay_right - self.samples_count_right) * DELAY_SMOOTHING_FACTOR;
        self.samples_count_right += target_delay_samples_right;
        let new_samples_count_right = f32_to_usize_clamped(self.samples_count_right.floor());
        let chorused_samples_right = self.delayed_sample_from_buffer(
            buffer_len,
            self.samples_count_right,
            new_samples_count_right,
            Side::Right,
        );

        self.buffer[self.write_index] = (
            samples.0
                + (chorused_samples_left * feedback)
                    * Defaults::SAMPLE_MIXING_LEVEL_CORRECTION_FACTOR,
            samples.1
                + (chorused_samples_right * feedback)
                    * Defaults::SAMPLE_MIXING_LEVEL_CORRECTION_FACTOR,
        );

        self.write_index = (self.write_index + 1) & (buffer_len - 1);

        let chorused_samples = (chorused_samples_left, chorused_samples_right);
        wet_dry_blend(samples, chorused_samples, blend)
    }
}

fn delay_center_value(sample_rate: f32) -> f32 {
    ((CHORUS_DEFAULT_DELAY_MILLISECONDS / 1000.0) * sample_rate).round()
}

#[cfg(test)]
mod tests {
    use super::*;
    use accsyn_core::math::f32s_are_equal;

    #[test]
    fn chorus_process_samples_returns_original_when_disabled() {
        let sample_rate = 48_000;
        let mut chorus = Chorus::new(sample_rate);
        let effect = EffectParameters {
            name: String::new(),
            is_enabled: false,
            parameters: vec![0.5, 0.5, 0.5, 0.0],
        };
        let input = (0.7, -0.4);

        let result = chorus.process_samples(input, &effect);

        assert!(f32s_are_equal(result.0, 0.7));
        assert!(f32s_are_equal(result.1, -0.4));
    }

    #[test]
    fn chorus_process_samples_fills_buffer_when_buffer_not_full() {
        let sample_rate = 48_000;
        let mut chorus = Chorus::new(sample_rate);
        let effect = EffectParameters {
            name: String::new(),
            is_enabled: true,
            parameters: vec![0.5, 0.5, 0.5, 0.0],
        };
        let input = (0.5, 0.5);

        // Buffer starts pre-allocated
        assert_eq!(chorus.buffer.len(), MAX_CHORUS_DELAY_SAMPLES as usize);
        assert_eq!(chorus.write_index, 0);

        // Process first sample
        let _result = chorus.process_samples(input, &effect);

        // Write index should have advanced
        assert_eq!(chorus.write_index, 1);
    }

    #[test]
    fn chorus_process_samples_resets_buffer_when_disabled_after_enabled() {
        let sample_rate = 48_000;
        let mut chorus = Chorus::new(sample_rate);

        // Enable and process samples to fill buffer
        let enabled_effect = EffectParameters {
            name: String::new(),
            is_enabled: true,
            parameters: vec![0.5, 0.5, 0.5, 0.0],
        };
        chorus.process_samples((0.5, 0.5), &enabled_effect);
        assert_eq!(chorus.write_index, 1);
        assert!(chorus.is_enabled);

        // Disable effect - should trigger reset
        let disabled_effect = EffectParameters {
            name: String::new(),
            is_enabled: false,
            parameters: vec![0.5, 0.5, 0.5, 0.0],
        };
        chorus.process_samples((0.5, 0.5), &disabled_effect);

        // Buffer should be reset (cleared to zeros and write_index reset)
        assert_eq!(chorus.write_index, 0);
        assert!(!chorus.is_enabled);
        // Verify buffer was cleared
        assert!(
            chorus
                .buffer
                .iter()
                .all(|&s| f32s_are_equal(s.0, 0.0) && f32s_are_equal(s.1, 0.0))
        );
    }

    #[test]
    fn chorus_process_samples_does_not_buffer_when_disabled() {
        let sample_rate = 48_000;
        let mut chorus = Chorus::new(sample_rate);

        // Start disabled
        assert!(!chorus.is_enabled);
        let initial_write_index = chorus.write_index;

        // Process with disabled effect - should not modify write index
        let disabled_effect = EffectParameters {
            name: String::new(),
            is_enabled: false,
            parameters: vec![0.5, 0.5, 0.5, 0.0],
        };
        chorus.process_samples((0.5, 0.5), &disabled_effect);

        // Should still be disabled with unchanged write index
        assert!(!chorus.is_enabled);
        assert_eq!(chorus.write_index, initial_write_index);
    }

    #[test]
    fn chorus_process_samples_returns_original_when_buffer_filling() {
        let sample_rate = 48_000;
        let mut chorus = Chorus::new(sample_rate);
        let effect = EffectParameters {
            name: String::new(),
            is_enabled: true,
            parameters: vec![0.5, 0.5, 0.5, 0.0],
        };
        let input = (0.8, -0.6);

        // First sample should return original since buffer isn't full
        let result = chorus.process_samples(input, &effect);

        assert!(f32s_are_equal(result.0, 0.8));
        assert!(f32s_are_equal(result.1, -0.6));
    }

    #[test]
    fn delayed_sample_from_buffer_interpolates_seeded_buffer_values() {
        let mut chorus = Chorus::new(48_000);
        chorus.write_index = 5;
        let buffer_len = chorus.buffer.len();
        // read_index = (5 + buffer_len - 3) & (buffer_len - 1) == 2, read_index_next == 1
        chorus.buffer[2] = (0.4, -0.2);
        chorus.buffer[1] = (0.8, -0.6);

        let result_left = chorus.delayed_sample_from_buffer(buffer_len, 3.25, 3, Side::Left);
        let result_right = chorus.delayed_sample_from_buffer(buffer_len, 3.25, 3, Side::Right);

        let expected = (0.5, -0.3);
        assert!(
            f32s_are_equal(result_left, expected.0),
            "Expected {}, got {}",
            expected.0,
            result_left
        );
        assert!(
            f32s_are_equal(result_right, expected.1),
            "Expected {}, got {}",
            expected.1,
            result_right
        );
    }

    #[test]
    fn delay_center_value_computes_samples_for_sample_rate() {
        let expected_48k = 960.0;
        let result_48k = delay_center_value(48_000.0);
        assert!(
            f32s_are_equal(result_48k, expected_48k),
            "Expected {expected_48k}, got {result_48k}"
        );

        let expected_44k = 882.0;
        let result_44k = delay_center_value(44_100.0);
        assert!(
            f32s_are_equal(result_44k, expected_44k),
            "Expected {expected_44k}, got {result_44k}"
        );
    }

    #[test]
    fn chorus_process_samples_lfo_voices_diverge_due_to_phase_offset() {
        let mut chorus = Chorus::new(48_000);
        let effect = EffectParameters {
            name: String::new(),
            is_enabled: true,
            parameters: vec![1.0, 0.8, 0.0, 1.0],
        };

        for _ in 0..100 {
            chorus.process_samples((0.0, 0.0), &effect);
        }

        assert!(
            !f32s_are_equal(chorus.samples_count_left, chorus.samples_count_right),
            "Expected chorus voices to diverge due to lfo2's phase offset, but both were {}",
            chorus.samples_count_left
        );
    }

    #[test]
    fn chorus_reset_restores_delay_center_value_not_min_delay_samples() {
        let mut chorus = Chorus::new(48_000);
        let delay_center_value = chorus.delay_center_value;

        let enabled_effect = EffectParameters {
            name: String::new(),
            is_enabled: true,
            parameters: vec![1.0, 0.8, 0.0, 1.0],
        };
        for _ in 0..500 {
            chorus.process_samples((0.3, -0.3), &enabled_effect);
        }
        // Confirm the voices actually drifted away from delay_center_value before disabling
        assert!(!f32s_are_equal(
            chorus.samples_count_left,
            delay_center_value
        ));

        let disabled_effect = EffectParameters {
            name: String::new(),
            is_enabled: false,
            parameters: vec![1.0, 0.8, 0.0, 1.0],
        };
        chorus.process_samples((0.0, 0.0), &disabled_effect);

        assert!(
            f32s_are_equal(chorus.samples_count_left, delay_center_value),
            "Expected {}, got {}",
            delay_center_value,
            chorus.samples_count_left
        );
        assert!(
            f32s_are_equal(chorus.samples_count_right, delay_center_value),
            "Expected {}, got {}",
            delay_center_value,
            chorus.samples_count_right
        );
    }

    #[test]
    fn chorus_process_samples_tracks_rate_changes() {
        let mut chorus = Chorus::new(48_000);
        let mut effect = EffectParameters {
            name: String::new(),
            is_enabled: true,
            parameters: vec![0.5, 0.2, 0.0, 0.0],
        };

        chorus.process_samples((0.0, 0.0), &effect);
        assert!(f32s_are_equal(chorus.rate, 0.2));

        effect.parameters[1] = 0.9;
        chorus.process_samples((0.0, 0.0), &effect);
        assert!(f32s_are_equal(chorus.rate, 0.9));

        // Processing again with an unchanged rate should leave the tracked rate as-is
        let previous_rate = chorus.rate;
        chorus.process_samples((0.0, 0.0), &effect);
        assert!(f32s_are_equal(chorus.rate, previous_rate));
    }
}
