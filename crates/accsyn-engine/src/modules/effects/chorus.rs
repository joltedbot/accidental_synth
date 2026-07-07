use crate::modules::effects;
use crate::modules::effects::constants::{
    CHORUS_DEFAULT_DELAY_MILLISECONDS, CHORUS_DEFAULT_LFO_RANGE, CHORUS_FEEDBACK_SCALE_FACTOR,
    CHORUS_LFO_CENTER_VALUE, CHORUS_LFO_FREQUENCY_COEFFICIENT, CHORUS_LFO_FREQUENCY_SCALE_FACTOR,
    CHORUS_LFO2_PHASE, DELAY_SMOOTHING_FACTOR, MAX_CHORUS_DELAY_SAMPLES,
};
use crate::modules::effects::dry_wet_blend;
use crate::modules::lfo::Lfo;
use accsyn_core::casting::f32_to_usize_clamped;
use accsyn_core::defaults::Defaults;
use accsyn_core::effects::{AudioEffect, EffectParameters};
use accsyn_core::math::{exponential_curve_from_normal_value_and_coefficient, f32s_are_equal};

// Ensure MAX_DELAY_SAMPLES is a power of 2 to guarantee the bitwise wrapping logic is safe
const _: () = assert!(MAX_CHORUS_DELAY_SAMPLES.is_power_of_two());

pub struct Chorus {
    buffer: Vec<(f32, f32)>,
    write_index: usize,
    is_enabled: bool,
    samples_count_1: f32,
    samples_count_2: f32,
    delay_center_value: f32,
    sample_rate: f32,
    lfo1: Lfo,
    lfo2: Lfo,
    rate: f32,
}

impl Chorus {
    pub fn new(sample_rate: u32) -> Self {
        log::debug!(target: "synthesizer::effects::chorus", "Constructing Delay Effect Module");

        let buffer = vec![(0.0, 0.0); MAX_CHORUS_DELAY_SAMPLES as usize];

        let mut lfo1 = Lfo::new(sample_rate);
        lfo1.set_center_value(CHORUS_LFO_CENTER_VALUE);
        lfo1.set_range(CHORUS_DEFAULT_LFO_RANGE);

        let mut lfo2 = Lfo::new(sample_rate);
        lfo2.set_center_value(CHORUS_LFO_CENTER_VALUE);
        lfo2.set_range(CHORUS_DEFAULT_LFO_RANGE);
        lfo2.set_phase(CHORUS_LFO2_PHASE);

        // Sample rate is always ≤ 192_000, within f32 precision (2²³ = 8_388_608)
        #[allow(clippy::cast_precision_loss)]
        let sample_rate = sample_rate as f32;
        let delay_center_value = delay_center_value(sample_rate);

        Self {
            buffer,
            write_index: 0,
            is_enabled: false,
            delay_center_value,
            samples_count_1: delay_center_value,
            samples_count_2: delay_center_value,
            sample_rate,
            lfo1,
            lfo2,
            rate: 0.0,
        }
    }

    fn reset(&mut self) {
        self.buffer.fill((0.0, 0.0));
        self.write_index = 0;
        let current_chorus_samples = delay_center_value(self.sample_rate);
        self.samples_count_1 = current_chorus_samples;
        self.samples_count_2 = current_chorus_samples;
        self.lfo1.set_phase(0.0);
        self.lfo2.set_phase(CHORUS_LFO2_PHASE);
    }

    fn delayed_sample_from_buffer(
        &mut self,
        buffer_len: usize,
        samples_count: f32,
        new_samples_count: usize,
    ) -> (f32, f32) {
        let read_index = (self.write_index + buffer_len - new_samples_count) & (buffer_len - 1);
        let read_index_next = (read_index + buffer_len - 1) & (buffer_len - 1);
        let buffer_samples = self.buffer[read_index];
        let buffer_samples_next = self.buffer[read_index_next];

        // new_samples_count is bounded by MAX_DELAY_SAMPLES (a small constant), within f32 precision (2²³ = 8_388_608)
        #[allow(clippy::cast_precision_loss)]
        let fractional_index1 = samples_count - new_samples_count as f32;
        let (interpolated_left, interpolated_right) =
            effects::interpolate_samples(buffer_samples, buffer_samples_next, fractional_index1);
        (interpolated_left, interpolated_right)
    }
}

impl AudioEffect for Chorus {
    fn process_samples(&mut self, samples: (f32, f32), effect: &EffectParameters) -> (f32, f32) {
        if self.is_enabled && !effect.is_enabled {
            self.reset();
        }

        self.is_enabled = effect.is_enabled;

        if !self.is_enabled {
            return samples;
        }

        let buffer_len = self.buffer.len();
        let depth = effect.parameters[0] * CHORUS_DEFAULT_LFO_RANGE;
        let rate = effect.parameters[1];
        let feedback = effect.parameters[2] * CHORUS_FEEDBACK_SCALE_FACTOR;
        let blend = effect.parameters[3];

        self.lfo1.set_range(depth);
        self.lfo2.set_range(depth);

        if !f32s_are_equal(rate, self.rate) {
            self.rate = rate;
            self.lfo1.set_frequency(
                exponential_curve_from_normal_value_and_coefficient(
                    rate,
                    CHORUS_LFO_FREQUENCY_COEFFICIENT,
                ) / CHORUS_LFO_FREQUENCY_SCALE_FACTOR,
            );

            self.lfo2.set_frequency(
                exponential_curve_from_normal_value_and_coefficient(
                    rate,
                    CHORUS_LFO_FREQUENCY_COEFFICIENT,
                ) / CHORUS_LFO_FREQUENCY_SCALE_FACTOR,
            );
        }

        // Chorus Voice 1
        let delay_offset1 = self.lfo1.generate(None);
        let target_delay1 = self.delay_center_value + (self.delay_center_value * delay_offset1);
        let target_samples1 = (target_delay1 - self.samples_count_1) * DELAY_SMOOTHING_FACTOR;
        self.samples_count_1 += target_samples1;
        let new_samples_count1 = f32_to_usize_clamped(self.samples_count_1.floor());
        let chorused_samples1 =
            self.delayed_sample_from_buffer(buffer_len, self.samples_count_1, new_samples_count1);

        // Chorus Voice 2
        let delay_offset2 = self.lfo2.generate(None);
        let target_delay2 = self.delay_center_value + (self.delay_center_value * delay_offset2);
        let target_samples2 = (target_delay2 - self.samples_count_2) * DELAY_SMOOTHING_FACTOR;
        self.samples_count_2 += target_samples2;
        let new_samples_count2 = f32_to_usize_clamped(self.samples_count_2.floor());
        let chorused_samples2 =
            self.delayed_sample_from_buffer(buffer_len, self.samples_count_2, new_samples_count2);

        self.buffer[self.write_index] = (
            samples.0
                + f32::midpoint(
                    chorused_samples1.0 * feedback,
                    chorused_samples2.0 * feedback,
                ),
            samples.1
                + f32::midpoint(
                    chorused_samples1.1 * feedback,
                    chorused_samples2.1 * feedback,
                ),
        );
        self.write_index = (self.write_index + 1) & (buffer_len - 1);

        let chorused_samples = (
            (chorused_samples1.0 + chorused_samples2.0)
                * Defaults::SAMPLE_MIXING_LEVEL_CORRECTION_FACTOR,
            (chorused_samples1.1 + chorused_samples2.1)
                * Defaults::SAMPLE_MIXING_LEVEL_CORRECTION_FACTOR,
        );

        dry_wet_blend(samples, chorused_samples, blend)
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

        let result = chorus.delayed_sample_from_buffer(buffer_len, 3.25, 3);

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
            !f32s_are_equal(chorus.samples_count_1, chorus.samples_count_2),
            "Expected chorus voices to diverge due to lfo2's phase offset, but both were {}",
            chorus.samples_count_1
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
        assert!(!f32s_are_equal(chorus.samples_count_1, delay_center_value));

        let disabled_effect = EffectParameters {
            name: String::new(),
            is_enabled: false,
            parameters: vec![1.0, 0.8, 0.0, 1.0],
        };
        chorus.process_samples((0.0, 0.0), &disabled_effect);

        assert!(
            f32s_are_equal(chorus.samples_count_1, delay_center_value),
            "Expected {}, got {}",
            delay_center_value,
            chorus.samples_count_1
        );
        assert!(
            f32s_are_equal(chorus.samples_count_2, delay_center_value),
            "Expected {}, got {}",
            delay_center_value,
            chorus.samples_count_2
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
