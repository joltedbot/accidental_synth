use crate::modules::effects::constants::{
    CHORUS_DEFAULT_DELAY_MILLISECONDS, CHORUS_DEFAULT_LFO_RANGE, CHORUS_FEEDBACK_SCALE_FACTOR,
    CHORUS_LFO_CENTER_VALUE, CHORUS_LFO_FREQUENCY_COEFFICIENT, CHORUS_LFO_FREQUENCY_SCALE_FACTOR,
    CHORUS_LFO2_PHASE, DELAY_SMOOTHING_FACTOR, MAX_CHORUS_DELAY_SAMPLES, MAX_DELAY_SAMPLES,
    MIN_DELAY_SAMPLES,
};
use crate::modules::effects::dry_wet_blend;
use crate::modules::lfo::Lfo;
use accsyn_core::casting::f32_to_usize_clamped;
use accsyn_core::effects::{AudioEffect, EffectParameters};
use accsyn_core::math::{exponential_curve_from_normal_value_and_coefficient, f32s_are_equal};

// Ensure MAX_DELAY_SAMPLES is a power of 2 to guarantee the bitwise wrapping logic is safe
const _: () = assert!(MAX_DELAY_SAMPLES.is_power_of_two());

pub struct Chorus {
    buffer: Vec<(f32, f32)>,
    write_index: usize,
    is_enabled: bool,
    samples_count_1: f32,
    samples_count_2: f32,
    delay_center_value: f32,
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
        lfo2.set_range(CHORUS_LFO2_PHASE);

        // CHORUS_DEFAULT_DELAY_MILLISECONDS is a small positive integer < 100 so it fits easily within the f32 mantissa
        #[allow(clippy::cast_precision_loss)]
        let delay_center_value =
            ((CHORUS_DEFAULT_DELAY_MILLISECONDS as f32 / 1000.0) * sample_rate as f32).round();

        Self {
            buffer,
            write_index: 0,
            is_enabled: false,
            delay_center_value,
            samples_count_1: delay_center_value,
            samples_count_2: delay_center_value,
            lfo1,
            lfo2,
            rate: 0.0,
        }
    }

    fn reset(&mut self) {
        self.buffer.fill((0.0, 0.0));
        self.write_index = 0;
        // MIN_DELAY_SAMPLES is a small constant (chorus samples count), within f32 precision (2²³ = 8_388_608)
        #[allow(clippy::cast_precision_loss)]
        let current_chorus_samples = MIN_DELAY_SAMPLES as f32;
        self.samples_count_1 = current_chorus_samples;
        self.samples_count_2 = current_chorus_samples;
        self.lfo1.set_phase(0.0);
        self.lfo2.set_phase(CHORUS_LFO2_PHASE);
    }

    fn interpolate_samples(
        buffer_samples_a: (f32, f32),
        buffer_samples_b: (f32, f32),
        fractional_index: f32,
    ) -> (f32, f32) {

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

        let delay_offset1 = self.lfo1.generate(None);

        // Delay sample count is bounded by MAX_DELAY_SAMPLES, within f32 precision (2²³ = 8_388_608)
        #[allow(clippy::cast_precision_loss)]
        let target_delay1 = self.delay_center_value + (self.delay_center_value * delay_offset1);
        let target_samples = (target_delay1 - self.samples_count_1) * DELAY_SMOOTHING_FACTOR;
        self.samples_count_1 += target_samples;

        let whole_sample_count1 = f32_to_usize_clamped(self.samples_count_1.floor());
        let read_index1 = (self.write_index + buffer_len - whole_sample_count1) & (buffer_len - 1);
        let read_index_next1 = (read_index1 + buffer_len - 1) & (buffer_len - 1);

        let buffer_samples_1 = self.buffer[read_index1];
        let buffer_samples_next_1 = self.buffer[read_index_next1];

        // chorus_samples is bounded by MAX_DELAY_SAMPLES (a small constant), within f32 precision (2²³ = 8_388_608)
        #[allow(clippy::cast_precision_loss)]
        let fractional_index1 = self.samples_count_1 - whole_sample_count1 as f32;
        let (interpolated_left1, interpolated_right1) =
            Self::interpolate_samples(buffer_samples_1, buffer_samples_next_1, fractional_index1);
        let chorused_samples1 = (interpolated_left1, interpolated_right1);

        // *

        let delay_offset2 = self.lfo2.generate(None);
        // Delay sample count is bounded by MAX_DELAY_SAMPLES, within f32 precision (2²³ = 8_388_608)
        #[allow(clippy::cast_precision_loss)]
        let target_delay2 = self.delay_center_value + (self.delay_center_value * delay_offset2);

        self.samples_count_2 += (target_delay2 - self.samples_count_2) * DELAY_SMOOTHING_FACTOR;
        let new_sample_count2 = f32_to_usize_clamped(self.samples_count_2.floor());

        let read_index2 = (self.write_index + buffer_len - new_sample_count2) & (buffer_len - 1);
        let read_index_next2 = (read_index2 + buffer_len - 1) & (buffer_len - 1);

        let buffer_samples_2 = self.buffer[read_index2];
        let buffer_samples_next_2 = self.buffer[read_index_next2];

        // chorus_samples is bounded by MAX_DELAY_SAMPLES (a small constant), within f32 precision (2²³ = 8_388_608)
        #[allow(clippy::cast_precision_loss)]
        let fractional_index2 = self.samples_count_2 - new_sample_count2 as f32;
        let (interpolated_left2, interpolated_right2) =
            Self::interpolate_samples(buffer_samples_2, buffer_samples_next_2, fractional_index2);
        let chorused_samples2 = (interpolated_left2, interpolated_right2);

        self.buffer[self.write_index] = (
            samples.0
                + f32::midpoint(
                    (chorused_samples1.0 * feedback),
                    (chorused_samples2.0 * feedback),
                ),
            samples.1
                + f32::midpoint(
                    (chorused_samples1.1 * feedback),
                    (chorused_samples2.1 * feedback),
                ),
        );
        self.write_index = (self.write_index + 1) & (buffer_len - 1);

        let chorused_samples = (
            f32::midpoint(chorused_samples1.0, chorused_samples2.0),
            f32::midpoint(chorused_samples1.1, chorused_samples2.1),
        );

        dry_wet_blend(samples, chorused_samples, blend)
    }
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
        assert_eq!(chorus.buffer.len(), MAX_DELAY_SAMPLES as usize);
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
}
