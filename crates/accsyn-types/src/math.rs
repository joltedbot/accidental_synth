#![allow(dead_code)]
use std::sync::atomic::AtomicU32;
use std::sync::atomic::Ordering::Relaxed;

const ALTERNATE_EPSILON: f32 = 1e-6;
const DBFS_SILENCE_LEVEL: f32 = -70.0;
const CENTS_PER_OCTAVE: f32 = 1200.0;
const MAX_MIDI_VALUE: f32 = 127.0;
const CENTER_MIDI_VALUE: u8 = 64;

/// Exponential coefficient for filter cutoff frequency mapping (ln(20000)).
pub const EXPONENTIAL_FILTER_COEFFICIENT: f32 = 9.903_487;
// filter range 20,000hz so ln(20000) = 9.903487
/// Exponential coefficient for level mapping (ln(1000)).
pub const EXPONENTIAL_LEVEL_COEFFICIENT: f32 = 6.908;
// Level linear range is 1000x so ln(1000) = 6.908
/// Level curve linear range: -60 to 0 dBFS = 60dB, so 10^(60/20) = 1000x.
pub const LEVEL_CURVE_LINEAR_RANGE: f32 = 1000.0;
/// Exponential coefficient for LFO rate mapping (ln(100000)).
pub const EXPONENTIAL_LFO_COEFFICIENT: f32 = 13.81551;
// ln(100_000) = 13.81551
/// Exponential coefficient for portamento time mapping (ln(500)).
pub const EXPONENTIAL_PORTAMENTO_COEFFICIENT: f32 = 6.214_608;
// ln(500) = 6.214608
/// Exponential curve parameters for envelope attack (min, max) in milliseconds.
pub const EXPONENTIAL_ENVELOPE_CURVE_ATTACK_VALUES: (f32, f32) = (0.5, 700.0);
/// Exponential curve parameters for envelope decay (min, max) in milliseconds.
pub const EXPONENTIAL_ENVELOPE_CURVE_DECAY_VALUES: (f32, f32) = (0.5, 1000.0);
/// Exponential curve parameters for envelope release (min, max) in milliseconds.
pub const EXPONENTIAL_ENVELOPE_CURVE_RELEASE_VALUES: (f32, f32) = (0.6, 2000.0);

/// Converts a dBFS value to a linear f32 sample amplitude.
#[inline]
#[must_use]
pub fn dbfs_to_f32_sample(dbfs: f32) -> f32 {
    if !dbfs.is_finite() || dbfs <= DBFS_SILENCE_LEVEL {
        return 0.0;
    }

    10.0_f32.powf(dbfs / 20.0)
}

/// Converts a linear f32 sample amplitude to dBFS.
#[inline]
#[must_use]
pub fn f32_sample_to_dbfs(sample: f32) -> f32 {
    if sample.is_nan() || sample == f32::NEG_INFINITY {
        return f32::NEG_INFINITY;
    }

    let sample_absolute_value = sample.abs();
    if sample_absolute_value <= ALTERNATE_EPSILON {
        return f32::NEG_INFINITY;
    }
    20.0 * sample_absolute_value.log10()
}

/// Compares two f32 values for approximate equality within epsilon tolerance.
#[inline]
#[must_use]
pub fn f32s_are_equal(value_1: f32, value_2: f32) -> bool {
    if value_1.is_nan() && value_2.is_nan() {
        return true;
    }
    if value_1.is_infinite() && value_2.is_infinite() {
        return true;
    }

    (value_1 - value_2).abs() <= ALTERNATE_EPSILON
}

/// Stores an f32 value in an `AtomicU32` using bit representation.
#[inline]
pub fn store_f32_as_atomic_u32(atomic: &AtomicU32, value: f32) {
    atomic.store(value.to_bits(), Relaxed);
}

/// Loads an f32 value from an `AtomicU32` using bit representation.
#[inline]
pub fn load_f32_from_atomic_u32(atomic: &AtomicU32) -> f32 {
    f32::from_bits(atomic.load(Relaxed))
}

/// Calculates a new frequency offset by the given number of cents.
#[inline]
#[must_use]
pub fn frequency_from_cents(frequency: f32, cents: i16) -> f32 {
    if !frequency.is_finite() || frequency.is_nan() {
        return 0.0;
    }

    frequency.abs() * (2.0f32.powf(f32::from(cents) / CENTS_PER_OCTAVE))
}

/// Normalizes a MIDI value (0–127) to a 0.0–1.0 range.
#[inline]
#[must_use]
pub fn normalize_midi_value(midi_value: u8) -> f32 {
    if midi_value == CENTER_MIDI_VALUE {
        return 0.5;
    }
    (f32::from(midi_value) / MAX_MIDI_VALUE).clamp(0.0, 1.0)
}

/// Normalizes an unsigned integer to 0.0–1.0 within the given range.
#[inline]
#[must_use]
pub fn normalize_unsigned_integer_range(input_value: u32, range_min: u32, range_max: u32) -> f32 {
    let range = range_max - range_min;
    // Audio values (sample rates, MIDI ranges) are always ≤ 192_000, within f32 precision (2²³ = 8_388_608)
    #[allow(clippy::cast_precision_loss)]
    let result = (input_value.saturating_sub(range_min) as f32 / range as f32).clamp(0.0, 1.0);
    result
}

/// Normalizes a signed integer to 0.0–1.0 within the given range.
#[inline]
#[must_use]
pub fn normalize_signed_integer_range(input_value: i32, range_min: i32, range_max: i32) -> f32 {
    let range = range_max - range_min;
    // Values represent audio ranges bounded by domain constraints; precision loss is acceptable in f32 DSP
    #[allow(clippy::cast_precision_loss)]
    let result = (input_value - range_min) as f32 / range as f32;
    result
}

/// Normalizes a float to 0.0–1.0 within the given range.
#[inline]
#[must_use]
pub fn normalize_float_range(input_value: f32, range_min: f32, range_max: f32) -> f32 {
    let range = range_max - range_min;
    (input_value - range_min) / range
}

/// Maps a linear input value to an exponential output scale, returning a 0.0–1.0 normalized result.
#[inline]
#[must_use]
pub fn map_value_from_linear_to_exponential_scale(
    mut input_value: f32,
    mut input_range: (f32, f32),
    mut output_range: (f32, f32),
) -> f32 {
    if f32s_are_equal(input_value, input_range.0) {
        return output_range.0;
    }

    if f32s_are_equal(input_value, input_range.1) {
        return output_range.1;
    }

    if f32s_are_equal(input_range.0, input_range.1) {
        return input_range.0;
    }

    if f32s_are_equal(output_range.0, output_range.1) {
        return output_range.0;
    }

    if input_range.1 < input_range.0 {
        swap_tuple_order(&mut input_range);
    }

    if output_range.1 < output_range.0 {
        swap_tuple_order(&mut output_range);
    }

    if output_range.0 <= 0.0 {
        output_range.0 = ALTERNATE_EPSILON;
    }

    input_value = input_value.clamp(input_range.0, input_range.1);

    let exponential_rate =
        (output_range.1.ln() - output_range.0.ln()) / (input_range.1 - input_range.0);

    let scaling_coefficient = output_range.0 / (exponential_rate * input_range.0).exp();

    let output_value = scaling_coefficient * (exponential_rate * input_value).exp();
    let scaled_output_value = (output_value - output_range.0) / (output_range.1 - output_range.0);

    scaled_output_value.clamp(0.0, 1.0)
}

#[inline]
fn swap_tuple_order(tuple: &mut (f32, f32)) {
    std::mem::swap(&mut tuple.0, &mut tuple.1);
}

/// Converts a normalized value (0.0–1.0) to an exponential curve using the given coefficient.
#[inline]
#[must_use]
pub fn exponential_curve_from_normal_value_and_coefficient(
    normal_value: f32,
    exponential_coefficient: f32,
) -> f32 {
    // exponential_coefficient is the log of the effective range for the linear scale you want to map to exponential range
    // If the range max is 1000x then min, then the exponential_coefficient is log(1000) = 6.908
    (exponential_coefficient * normal_value).exp()
}

/// Converts an exponential curve value back to a normalized value (0.0–1.0) using the given coefficient.
#[inline]
#[must_use]
pub fn normal_value_from_exponential_curve_and_coefficient(
    curve_value: f32,
    exponential_coefficient: f32,
) -> f32 {
    // exponential_coefficient is the log of the effective range for the linear scale you want to map to exponential range
    // If the range max is 1000x then min, then the exponential_coefficient is log(1000) = 6.908
    curve_value.ln() / exponential_coefficient
}

/// Inverse of `exponential_curve_level_adjustment_from_normal_value`.
/// Converts an engine-side level value back to a normalized UI value (0.0–1.0).
#[inline]
#[must_use]
pub fn normal_value_from_exponential_level_curve(level_value: f32) -> f32 {
    if level_value == 0.0 {
        return 0.0;
    }
    (level_value * LEVEL_CURVE_LINEAR_RANGE).ln() / EXPONENTIAL_LEVEL_COEFFICIENT
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dbfs_to_sample_returns_correct_values_for_valid_input() {
        let dbfs_values: [f32; 9] = [
            6.0,
            0.0,
            -6.0,
            -18.19484,
            -144.0,
            f32::NEG_INFINITY,
            f32::NAN,
            f32::EPSILON,
            f32::INFINITY,
        ];
        let expected_samples: [f32; 9] = [
            1.995_262_4,
            1.0,
            0.501_187_2,
            0.123_099_98,
            0.0,
            0.0,
            0.0,
            1.0,
            0.0,
        ];

        for (dbfs, expected_sample) in dbfs_values.iter().zip(expected_samples.iter()) {
            let sample = dbfs_to_f32_sample(*dbfs);
            assert!(
                f32s_are_equal(sample, *expected_sample),
                "For: {dbfs:?}, Expected: {expected_sample:?}, got: {sample:?}"
            );
        }
    }

    #[test]
    fn f32_sample_to_dbfs_returns_correct_values_for_valid_input() {
        let sample_values: [f32; 6] = [-1.0, -0.5, -0.000_123_1, 0.0, 0.5, 1.0];
        let expected_dbfs: [f32; 6] = [
            0.0,
            -6.020_600_3,
            -78.19484,
            f32::NEG_INFINITY,
            -6.020_600_3,
            0.0,
        ];
        for (sample, expected_dbfs) in sample_values.iter().zip(expected_dbfs.iter()) {
            let dbfs = f32_sample_to_dbfs(*sample);
            assert!(
                f32s_are_equal(dbfs, *expected_dbfs),
                "For: {sample:?}, Expected: {expected_dbfs:?}, got: {dbfs:?}"
            );
        }
    }

    #[test]
    fn f32_sample_to_dbfs_returns_correct_values_for_out_of_range_input() {
        let sample_values: [f32; 4] = [f32::NAN, f32::NEG_INFINITY, f32::EPSILON, f32::INFINITY];
        let expected_dbfs: [f32; 4] = [
            f32::NEG_INFINITY,
            f32::NEG_INFINITY,
            f32::NEG_INFINITY,
            f32::INFINITY,
        ];
        for (sample, expected_dbfs) in sample_values.iter().zip(expected_dbfs.iter()) {
            let dbfs = f32_sample_to_dbfs(*sample);
            assert!(
                f32s_are_equal(dbfs, *expected_dbfs),
                "For: {sample:?}, Expected: {expected_dbfs:?}, got: {dbfs:?}"
            );
        }
    }

    #[test]
    fn f32s_are_equal_returns_true_for_valid_equal_values() {
        let values = [
            1.0,
            -1.0,
            0.0,
            f32::MAX,
            f32::MIN,
            f32::EPSILON,
            f32::NEG_INFINITY,
            f32::INFINITY,
            f32::NAN,
        ];
        for value in values {
            assert!(
                f32s_are_equal(value, value),
                "For: {value:?} & {value:?}, Expected: true, got: false"
            );
        }
    }

    #[test]
    fn f32s_are_equal_returns_false_for_valid_unequal_values() {
        let values1 = [
            1.0,
            -1.0,
            0.0,
            f32::MAX,
            f32::MIN,
            f32::EPSILON,
            f32::NEG_INFINITY,
            f32::INFINITY,
            f32::NAN,
        ];
        let values2 = [
            0.0,
            1.0,
            -1.0,
            f32::MIN,
            f32::MAX,
            f32::INFINITY,
            f32::NAN,
            f32::EPSILON,
            f32::NEG_INFINITY,
        ];
        for (value1, value2) in values1.iter().zip(values2.iter()) {
            assert!(
                !f32s_are_equal(*value1, *value2),
                "For: {value1:?} & {value1:?}, Expected: false, got: true"
            );
        }
    }

    #[test]
    fn f32s_are_equal_returns_true_for_sub_epsilon_differences() {
        let base_number = 1.0;
        let difference_below_epsilon = base_number + (ALTERNATE_EPSILON / 2.0);
        assert!(f32s_are_equal(base_number, difference_below_epsilon));
    }

    #[test]
    fn f32s_are_equal_returns_true_for_just_above_epsilon_differences() {
        let base_number = 1.0;
        let difference_above_epsilon = base_number + (ALTERNATE_EPSILON * 2.0);
        assert!(!f32s_are_equal(base_number, difference_above_epsilon));
    }

    #[test]
    fn f32s_are_equal_returns_true_for_zeros() {
        assert!(f32s_are_equal(0.0, 0.0));
        assert!(f32s_are_equal(-0.0, 0.0));
    }

    #[test]
    fn atomic_load_and_store_of_atomic_u32_converts_to_and_from_f32_values() {
        let atomic = AtomicU32::new(0);
        let test_values = [0.0, 1.0, -1.0, f32::MAX, f32::MIN, f32::EPSILON];

        for &value in &test_values {
            store_f32_as_atomic_u32(&atomic, value);
            let result = load_f32_from_atomic_u32(&atomic);
            assert!(
                f32s_are_equal(result, value),
                "For: {value:?}, Expected: {value:?}, got: {result:?}"
            );
        }
    }

    #[test]
    fn frequency_from_cents_returns_correct_frequency_from_valid_cents() {
        let frequency = 440.0;
        let test_cents = [0, 50, 700, 1200];
        let expected_results = [440.0, 452.892_97, 659.255_1, 880.0];

        for (cents, expected_result) in test_cents.iter().zip(expected_results.iter()) {
            let result = frequency_from_cents(frequency, *cents);
            assert!(f32s_are_equal(result, *expected_result));
        }
    }

    #[test]
    fn frequency_from_cents_returns_clamped_values_from_out_of_range_cents() {
        let frequency = 440.0;
        let test_cents = [i16::MAX, i16::MIN];
        let expected_results = [73_000_810_000.0, 2.650_493_3e-6];

        for (cents, expected_result) in test_cents.iter().zip(expected_results.iter()) {
            let result = frequency_from_cents(frequency, *cents);
            assert!(
                f32s_are_equal(result, *expected_result),
                "For: {:?}, Expected: {:?} but got {result:?}",
                *cents,
                *expected_result
            );
        }
    }

    #[test]
    fn frequency_from_cents_returns_returns_result_for_absolute_value_of_negative_frequency() {
        assert!(f32s_are_equal(frequency_from_cents(-440.0, 1200), 880.0));
    }

    #[test]
    fn frequency_from_cents_returns_returns_zero_for_infinite_frequency() {
        assert!(f32s_are_equal(
            frequency_from_cents(f32::INFINITY, 1200),
            0.0
        ));
        assert!(f32s_are_equal(
            frequency_from_cents(f32::NEG_INFINITY, 1200),
            0.0
        ));
    }

    #[test]
    fn frequency_from_cents_returns_zero_for_nan_frequency() {
        let frequency = f32::NAN;
        let cents = 100;

        let actual = frequency_from_cents(frequency, cents);
        let expected = 0.0;

        assert!(f32s_are_equal(actual, expected));
    }

    #[test]
    fn frequency_from_cents_returns_zero_for_zero_frequency() {
        assert!(f32s_are_equal(frequency_from_cents(0.0, 100), 0.0));
    }

    #[test]
    fn map_value_from_linear_to_exponential_scale_returns_correct_result_from_valid_negative_ranges()
     {
        let value = 5.0;
        let input_range = (0.0, 10.0);
        let output_range = (-10.0, 100.0);
        let expected_result = 0.000_099_9;

        let result = map_value_from_linear_to_exponential_scale(value, input_range, output_range);
        assert!(
            f32s_are_equal(result, expected_result),
            "For: {value}, Expected: {expected_result}, got: {result}"
        );
    }

    #[test]
    fn map_value_from_linear_to_exponential_scale_returns_correct_result_from_valid_tiny_ranges() {
        let value = 5.0;
        let input_range = (0.0, 10.0);
        let output_range = (10.0, 10.000_000_1);
        let expected_result = 10.0;

        let result = map_value_from_linear_to_exponential_scale(value, input_range, output_range);
        assert!(
            f32s_are_equal(result, expected_result),
            "For: {value}, Expected: {expected_result}, got: {result}"
        );
    }

    #[test]
    fn map_value_from_linear_to_exponential_scale_returns_correct_result_from_value_at_input_range_extremes()
     {
        let min_input = 0.0;
        let max_input = 10.0;
        let min_output = 10.0;
        let max_output = 100.0;
        let input_range = (0.0, 10.0);
        let output_range = (10.0, 100.0);

        let result_min =
            map_value_from_linear_to_exponential_scale(min_input, input_range, output_range);
        assert!(
            f32s_are_equal(result_min, min_output),
            "For: {min_input}, Expected: {min_output}, got: {result_min}"
        );

        let result_max =
            map_value_from_linear_to_exponential_scale(max_input, input_range, output_range);
        assert!(
            f32s_are_equal(result_max, max_output),
            "For: {result_max}, Expected: {max_output}, got: {result_max}"
        );
    }

    #[test]
    fn map_value_from_linear_to_exponential_scale_returns_correct_result_from_reversed_input_range()
    {
        let value = 5.0;
        let input_range = (10.0, 0.0);
        let output_range = (10.0, 100.0);
        let expected_result = 0.240_253;
        let result = map_value_from_linear_to_exponential_scale(value, input_range, output_range);
        assert!(
            f32s_are_equal(result, expected_result),
            "For: {value}, Expected: {expected_result}, got: {result}"
        );
    }

    #[test]
    fn map_value_from_linear_to_exponential_scale_returns_correct_result_from_reversed_output_range()
     {
        let value = 5.0;
        let input_range = (0.0, 10.0);
        let output_range = (100.0, 10.0);
        let expected_result = 0.240_253;
        let result = map_value_from_linear_to_exponential_scale(value, input_range, output_range);
        assert!(
            f32s_are_equal(result, expected_result),
            "For: {value}, Expected: {expected_result}, got: {result}"
        );
    }

    #[test]
    fn map_value_from_linear_to_exponential_scale_returns_value_when_input_range_is_zero_length() {
        let value = 5.0;
        let input_range = (10.0, 10.0);
        let output_range = (10.0, 100.0);
        let expected_result = 10.0;
        let result = map_value_from_linear_to_exponential_scale(value, input_range, output_range);
        assert!(
            f32s_are_equal(result, expected_result),
            "For: {value}, Expected: {expected_result}, got: {result}"
        );
    }

    #[test]
    fn map_value_from_linear_to_exponential_scale_returns_value_when_output_range_is_zero_length() {
        let value = 5.0;
        let input_range = (0.0, 10.0);
        let output_range = (100.0, 100.0);
        let expected_result = 100.0;
        let result = map_value_from_linear_to_exponential_scale(value, input_range, output_range);
        assert!(
            f32s_are_equal(result, expected_result),
            "For: {value}, Expected: {expected_result}, got: {result}"
        );
    }

    #[test]
    fn map_value_from_linear_to_exponential_scale_returns_epsilon_clamped_value_when_output_range_min_is_zero()
     {
        let value = 5.0;
        let input_range = (0.0, 10.0);
        let output_range = (0.0, 100.0);
        let expected_result = 0.000_099_9;
        let result = map_value_from_linear_to_exponential_scale(value, input_range, output_range);
        assert!(
            f32s_are_equal(result, expected_result),
            "For: {value}, Expected: {expected_result}, got: {result}"
        );
    }

    #[test]
    fn map_value_from_linear_to_exponential_scale_returns_range_clamped_value_when_value_is_outside_range()
     {
        let value = 15.0;
        let input_range = (0.0, 10.0);
        let output_range = (10.0, 100.0);
        let expected_result = 1.0;
        let result = map_value_from_linear_to_exponential_scale(value, input_range, output_range);
        assert!(
            f32s_are_equal(result, expected_result),
            "For: {value}, Expected: {expected_result}, got: {result}"
        );
    }

    #[test]
    fn normalize_midi_value_returns_correct_normal_value_for_valid_input() {
        let midi_value = 54;
        let expected_result = 0.425_196_8;
        let result = normalize_midi_value(midi_value);
        assert!(
            f32s_are_equal(result, expected_result),
            "For: {midi_value}, Expected: {expected_result}, got: {result}"
        );
    }

    #[test]
    fn normalize_midi_value_returns_half_for_center_value() {
        let midi_value = CENTER_MIDI_VALUE; // 64

        let actual = normalize_midi_value(midi_value);
        let expected = 0.5;

        assert!(
            f32s_are_equal(actual, expected),
            "For: {midi_value}, Expected: {expected}, got: {actual}"
        );
    }

    #[test]
    fn normalize_midi_value_returns_1_for_out_of_range_values() {
        let high_midi_value = 200;
        let high_expected_result = 1.0;

        let high_result = normalize_midi_value(high_midi_value);
        assert!(
            f32s_are_equal(high_result, high_expected_result),
            "For: {high_midi_value}, Expected: \
        {high_expected_result}, got: {high_result}"
        );
    }
}
