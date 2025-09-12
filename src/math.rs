#![allow(dead_code)]
use std::sync::atomic::AtomicU32;
use std::sync::atomic::Ordering::Relaxed;

const ALTERNATE_EPSILON: f32 = 1e-6;
const DBFS_SILENCE_LEVEL: f32 = -70.0;
const CENTS_PER_OCTAVE: f32 = 1200.0;

pub fn dbfs_to_f32_sample(dbfs: f32) -> f32 {
    if !dbfs.is_finite() || dbfs <= DBFS_SILENCE_LEVEL {
        return 0.0;
    }

    10.0_f32.powf(dbfs / 20.0)
}

pub fn f32_sample_to_dbfs(sample: f32) -> f32 {
    if sample.is_nan() || sample == f32::NEG_INFINITY {
        return f32::NEG_INFINITY
    }

    let sample_absolute_value = sample.abs();
    if sample_absolute_value <= ALTERNATE_EPSILON {
        return f32::NEG_INFINITY;
    }
    20.0 * sample_absolute_value.log10()
}

pub fn f32s_are_equal(value_1: f32, value_2: f32) -> bool {
    if value_1.is_nan() && value_2.is_nan() {
        return true;
    }
    if value_1.is_infinite() && value_2.is_infinite() {
        return true;
    }

    (value_1 - value_2).abs() <= ALTERNATE_EPSILON
}

pub fn store_f32_as_atomic_u32(atomic: &AtomicU32, value: f32) {
    atomic.store(value.to_bits(), Relaxed);
}

pub fn load_f32_from_atomic_u32(atomic: &AtomicU32) -> f32 {
    f32::from_bits(atomic.load(Relaxed))
}

pub fn frequency_from_cents(frequency: f32, cents: i16) -> f32 {
    if !frequency.is_finite() {
        return 0.0;
    }

    frequency.abs() * (2.0f32.powf(f32::from(cents) / CENTS_PER_OCTAVE))
}

fn map_value_from_linear_to_exponential_scale(
    mut input_value: f32,
    mut input_range: (f32, f32),
    mut output_range: (f32, f32),
) -> f32 {

    if f32s_are_equal(input_value, input_range.0)  {
        return output_range.0;
    }

    if f32s_are_equal(input_value, input_range.1)  {
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

    let scaling_coeficient = output_range.0 / (exponential_rate * input_range.0).exp();

    let output_value = scaling_coeficient * (exponential_rate * input_value).exp();
    let scaled_output_value = (output_value - output_range.0) / (output_range.1 - output_range.0);

    scaled_output_value.clamp(0.0, 1.0)
}

fn swap_tuple_order(tuple: &mut (f32, f32)) {
    std::mem::swap(&mut tuple.0, &mut tuple.1);
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
        let sample_values: [f32; 6] = [
            -1.0,
            -0.5,
            -0.000_123_1,
            0.0,
            0.5,
            1.0,
        ];
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
    fn f32_sample_to_dbfs_returns_correct_values_for_outof_range_input() {
        let sample_values: [f32; 4] = [
            f32::NAN,
            f32::NEG_INFINITY,
            f32::EPSILON,
            f32::INFINITY,
        ];
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
    fn f32s_are_equal_returns_true_for_identical_values() {
        let equal_number = 1.0;
        assert!(f32s_are_equal(equal_number, equal_number));
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
    fn atomic_load_and_store_of_atomicu32_converts_to_and_from_f32_values() {
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
    fn frequency_from_cents_returns_lower_frequency_for_negative_cents() {
        assert!(f32s_are_equal(frequency_from_cents(440.0, -1200), 220.0));
    }

    #[test]
    fn frequency_from_cents_returns_returns_result_for_absolute_value_of_negaitve_frequency() {
        assert!(f32s_are_equal(frequency_from_cents(-440.0, 1200), 880.0));
    }

    #[test]
    fn frequency_from_cents_returns_zero_for_zero_frequency() {
        assert!(f32s_are_equal(frequency_from_cents(0.0, 100), 0.0));
    }

    #[test]
    fn frequency_from_cents_returns_same_frequency_for_zero_cents() {
        assert!(f32s_are_equal(frequency_from_cents(440.0, 0), 440.0));
    }

    #[test]
    fn map_value_from_linear_to_exponential_scale_returns_correct_result_from_valid_ranges() {
        let value = 5.0;
        let input_range = (0.0, 10.0);
        let output_range = (10.0, 100.0);
        let expected_result = 0.240_253_06;

        let result = map_value_from_linear_to_exponential_scale(value, input_range, output_range);
        assert!(f32s_are_equal(result, expected_result), "For: {value}, Expected: {expected_result}, got: {result}");
    }

    #[test]
    fn map_value_from_linear_to_exponential_scale_returns_correct_result_from_valid_negative_ranges()
     {
        let value = 5.0;
        let input_range = (0.0, 10.0);
        let output_range = (-10.0, 100.0);
        let expected_result = 0.000_099_9;

        let result = map_value_from_linear_to_exponential_scale(value, input_range, output_range);
        assert!(f32s_are_equal(result, expected_result), "For: {value}, Expected: {expected_result}, got: {result}");
    }

    #[test]
    fn map_value_from_linear_to_exponential_scale_returns_correct_result_from_valid_tiny_ranges() {
        let value = 5.0;
        let input_range = (0.0, 10.0);
        let output_range = (10.0, 10.000_000_1);
        let expected_result = 10.0;

        let result = map_value_from_linear_to_exponential_scale(value, input_range, output_range);
        assert!(f32s_are_equal(result, expected_result), "For: {value}, Expected: {expected_result}, got: {result}");
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
        assert!(f32s_are_equal(result_min, min_output), "For: {min_input}, Expected: {min_output}, got: {result_min}");

        let result_max =
            map_value_from_linear_to_exponential_scale(max_input, input_range, output_range);
        assert!(f32s_are_equal(result_max, max_output), "For: {result_max}, Expected: {max_output}, got: {result_max}");
    }

    #[test]
    fn map_value_from_linear_to_exponential_scale_returns_correct_result_from_reversed_ranges() {
        let value = 5.0;
        let input_range = (0.0, 10.0);
        let output_range = (100.0, 10.0);
        let expected_result = 0.240_253;
        let result = map_value_from_linear_to_exponential_scale(value, input_range, output_range);
        assert!(f32s_are_equal(result, expected_result), "For: {value}, Expected: {expected_result}, got: {result}");
    }

    #[test]
    fn map_value_from_linear_to_exponential_scale_returns_value_when_input_range_is_zero_length() {
        let value = 5.0;
        let input_range = (10.0, 10.0);
        let output_range = (10.0, 100.0);
        let expected_result = 10.0;
        let result = map_value_from_linear_to_exponential_scale(value, input_range, output_range);
        assert!(f32s_are_equal(result, expected_result), "For: {value}, Expected: {expected_result}, got: {result}");
    }

    #[test]
    fn map_value_from_linear_to_exponential_scale_returns_value_when_output_range_is_zero_length() {
        let value = 5.0;
        let input_range = (0.0, 10.0);
        let output_range = (100.0, 100.0);
        let expected_result = 100.0;
        let result = map_value_from_linear_to_exponential_scale(value, input_range, output_range);
        assert!(f32s_are_equal(result, expected_result), "For: {value}, Expected: {expected_result}, got: {result}");
    }

    #[test]
    fn map_value_from_linear_to_exponential_scale_returns_epsilon_clamped_value_when_output_range_min_is_zero()
     {
        let value = 5.0;
        let input_range = (0.0, 10.0);
        let output_range = (0.0, 100.0);
        let expected_result = 0.000_099_9;
        let result = map_value_from_linear_to_exponential_scale(value, input_range, output_range);
        assert!(f32s_are_equal(result, expected_result), "For: {value}, Expected: {expected_result}, got: {result}");
    }

    #[test]
    fn map_value_from_linear_to_exponential_scale_returns_range_clamped_value_when_value_is_outside_range()
     {
        let value = 15.0;
        let input_range = (0.0, 10.0);
        let output_range = (10.0, 100.0);
        let expected_result = 1.0;
        let result = map_value_from_linear_to_exponential_scale(value, input_range, output_range);
        assert!(f32s_are_equal(result, expected_result), "For: {value}, Expected: {expected_result}, got: {result}");
    }

    #[test]
    fn swap_tuple_order_correctly_swaps_the_order_and_back_again() {
        let mut test_tuple = (1.0, 2.0);
        let expected_result_1 = (2.0, 1.0);
        let expected_result_2 = (1.0, 2.0);

        swap_tuple_order(&mut test_tuple);
        assert_eq!(test_tuple, expected_result_1);

        swap_tuple_order(&mut test_tuple);
        assert_eq!(test_tuple, expected_result_2);
    }
}
