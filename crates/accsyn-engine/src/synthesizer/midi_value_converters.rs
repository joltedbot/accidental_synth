use crate::modules::filter::NUMBER_OF_FILER_POLES;
use crate::modules::oscillator::{
    FIRST_WAVE_SHAPE_INDEX, LAST_WAVE_SHAPE_INDEX, OscillatorParameters,
};
use crate::synthesizer::constants::{
    CENTS_PER_SEMITONE, EXPONENTIAL_FILTER_COEFFICIENT, EXPONENTIAL_LEVEL_COEFFICIENT,
    EXPONENTIAL_LFO_COEFFICIENT, LEVEL_CURVE_LINEAR_RANGE, LINEAR_VELOCITY_CURVE_EXPONENT,
    MAX_MIDI_KEY_VELOCITY, MAX_VELOCITY_CURVE_EXPONENT, MIN_VELOCITY_CURVE_EXPONENT,
    NORMAL_TO_BOOL_SWITCH_ON_VALUE, PITCH_BEND_AMOUNT_MAX_VALUE, PITCH_BEND_AMOUNT_ZERO_POINT,
};
use accsyn_types::math::f32s_are_equal;
use std::sync::atomic::Ordering::Relaxed;

pub fn normal_value_to_f32_range(normal_value: f32, mut minimum: f32, mut maximum: f32) -> f32 {
    if maximum < minimum {
        core::mem::swap(&mut minimum, &mut maximum);
    }

    let range = maximum - minimum;
    minimum + (normal_value * range)
}

pub fn normal_value_to_unsigned_integer_range(
    normal_value: f32,
    mut minimum: u32,
    mut maximum: u32,
) -> u32 {
    if maximum < minimum {
        core::mem::swap(&mut minimum, &mut maximum);
    }

    let range = maximum - minimum;
    let clamped_value = normal_value.clamp(0.0, 1.0);
    let scaled_value = (f64::from(clamped_value) * f64::from(range)).round();

    (f64::from(minimum) + scaled_value).clamp(f64::from(minimum), f64::from(maximum)) as u32
}

pub fn normal_value_to_signed_integer_range(
    normal_value: f32,
    mut minimum: i32,
    mut maximum: i32,
) -> i32 {
    if maximum < minimum {
        core::mem::swap(&mut minimum, &mut maximum);
    }

    let range = maximum - minimum;
    let clamped_value = normal_value.clamp(0.0, 1.0);
    let scaled_value = (f64::from(clamped_value) * f64::from(range)).round();

    (f64::from(minimum) + scaled_value).clamp(f64::from(minimum), f64::from(maximum)) as i32
}

pub fn normal_value_to_bool(normal_value: f32) -> bool {
    normal_value >= NORMAL_TO_BOOL_SWITCH_ON_VALUE
}

pub fn normal_value_to_number_of_filter_poles(normal_value: f32) -> u8 {
    (NUMBER_OF_FILER_POLES * normal_value)
        .ceil()
        .clamp(1.0, NUMBER_OF_FILER_POLES) as u8
}

pub fn normal_value_to_wave_shape_index(normal_value: f32) -> u8 {
    normal_value_to_unsigned_integer_range(
        normal_value,
        FIRST_WAVE_SHAPE_INDEX,
        LAST_WAVE_SHAPE_INDEX,
    )
    .clamp(FIRST_WAVE_SHAPE_INDEX, LAST_WAVE_SHAPE_INDEX) as u8
}

pub fn exponential_curve_filter_cutoff_from_midi_value(normal_value: f32) -> f32 {
    if normal_value == 0.0 {
        return 0.0;
    }
    exponential_curve_from_normal_value_and_coefficient(
        normal_value,
        EXPONENTIAL_FILTER_COEFFICIENT,
    )
}

pub fn exponential_curve_lfo_frequency_from_normal_value(normal_value: f32) -> f32 {
    if normal_value == 0.0 {
        return 0.0;
    }
    exponential_curve_from_normal_value_and_coefficient(normal_value, EXPONENTIAL_LFO_COEFFICIENT)
        / 100.0
}

pub fn exponential_curve_envelope_time_from_normal_value(
    normal_value: f32,
    crossover_values: (f32, f32),
    minimum: u32,
    maximum: u32,
) -> u32 {
    if normal_value == 0.0 {
        return 0;
    }

    if normal_value < crossover_values.0 {
        let renormailzed_value = normal_value / crossover_values.0;
        let quadratic_curve = renormailzed_value.powi(2);
        minimum + ((crossover_values.1 - minimum as f32) * quadratic_curve) as u32
    } else {
        let renormailzed_value = (normal_value - crossover_values.0) / crossover_values.0;
        let quadratic_curve = renormailzed_value.powi(2);
        (crossover_values.1 + ((maximum as f32 - crossover_values.1) * quadratic_curve).round())
            as u32
    }
}

pub fn exponential_curve_level_adjustment_from_normal_value(normal_value: f32) -> f32 {
    if normal_value == 0.0 {
        return 0.0;
    }
    exponential_curve_from_normal_value_and_coefficient(normal_value, EXPONENTIAL_LEVEL_COEFFICIENT)
        / LEVEL_CURVE_LINEAR_RANGE
}

pub fn exponential_curve_from_normal_value_and_coefficient(
    normal_value: f32,
    exponential_coefficient: f32,
) -> f32 {
    // exponential_coefficient is the log of the effective range for the linear scale you want to map to exponential range
    // If the range max is 1000x then min, then the exponential_coefficient is log(1000) = 6.908
    (exponential_coefficient * normal_value).exp()
}

pub fn velocity_curve_from_normal_value(normal_value: f32) -> f32 {
    if normal_value == 0.0 {
        return 0.0;
    }

    let linear_normal_value = 0.5;

    if normal_value <= linear_normal_value {
        let renormaled_value = normal_value / linear_normal_value;
        normal_value_to_f32_range(
            renormaled_value,
            MIN_VELOCITY_CURVE_EXPONENT,
            LINEAR_VELOCITY_CURVE_EXPONENT,
        )
    } else {
        let renormaled_value = (normal_value - 0.5) / linear_normal_value;
        normal_value_to_f32_range(
            renormaled_value,
            LINEAR_VELOCITY_CURVE_EXPONENT,
            MAX_VELOCITY_CURVE_EXPONENT,
        )
    }
}

pub fn scaled_velocity_from_normal_value(velocity_curve: f32, velocity: f32) -> f32 {
    if f32s_are_equal(velocity_curve, 1.0) {
        return velocity;
    }
    if f32s_are_equal(velocity_curve, 0.0) {
        return MAX_MIDI_KEY_VELOCITY;
    }

    velocity.powf(velocity_curve)
}

pub fn update_current_note_from_midi_pitch_bend(
    pitch_bend_amount: u16,
    range_in_semitones: u8,
    oscillators: &[OscillatorParameters; 4],
) {
    let max_bend_in_cents = u16::from(range_in_semitones) * CENTS_PER_SEMITONE;
    for oscillator in oscillators {
        if pitch_bend_amount == PITCH_BEND_AMOUNT_ZERO_POINT {
            oscillator.pitch_bend.store(0, Relaxed);
        } else if pitch_bend_amount >= PITCH_BEND_AMOUNT_MAX_VALUE {
            oscillator
                .pitch_bend
                .store(max_bend_in_cents as i16, Relaxed);
        } else {
            let pitch_bend_in_cents =
                midi_value_to_pitch_bend_cents(pitch_bend_amount, max_bend_in_cents);
            oscillator.pitch_bend.store(pitch_bend_in_cents, Relaxed);
        }
    }
}

fn midi_value_to_pitch_bend_cents(pitch_bend_amount: u16, max_bend_in_cents: u16) -> i16 {
    ((f32::from(pitch_bend_amount) - f32::from(PITCH_BEND_AMOUNT_ZERO_POINT))
        / f32::from(PITCH_BEND_AMOUNT_ZERO_POINT)
        * f32::from(max_bend_in_cents)) as i16
}

#[cfg(test)]
mod tests {
    use super::*;
    use accsyn_types::math::{f32s_are_equal, normalize_midi_value};

    #[test]
    fn normalize_midi_value_correctly_maps_values() {
        assert!(f32s_are_equal(normalize_midi_value(0), 0.0));
        assert!(f32s_are_equal(normalize_midi_value(127), 1.0));
    }

    #[test]
    fn midi_value_to_bool_correctly_converts_threshold() {
        assert!(!normal_value_to_bool(0.0));
        assert!(!normal_value_to_bool(0.49));
        assert!(normal_value_to_bool(0.5));
        assert!(normal_value_to_bool(1.0));
    }

    // Tests for normal_value_to_f32_range
    #[test]
    fn test_normal_value_to_f32_range_standard() {
        let normal_value = 0.5;
        let min = 0.0;
        let max = 100.0;

        let actual = normal_value_to_f32_range(normal_value, min, max);
        let expected = 50.0;

        assert!(
            f32s_are_equal(actual, expected),
            "Expected {expected}, got {actual}"
        );
    }

    #[test]
    fn test_normal_value_to_f32_range_inverted_range() {
        let normal_value = 0.5;
        let min = 100.0;
        let max = 0.0; // Inverted

        let actual = normal_value_to_f32_range(normal_value, min, max);
        let expected = 50.0; // Should swap and produce same result

        assert!(
            f32s_are_equal(actual, expected),
            "Expected {expected}, got {actual}"
        );
    }

    #[test]
    fn test_normal_value_to_f32_range_boundary_values() {
        let min = 20.0;
        let max = 200.0;

        let actual_min = normal_value_to_f32_range(0.0, min, max);
        let expected_min = 20.0;

        let actual_max = normal_value_to_f32_range(1.0, min, max);
        let expected_max = 200.0;

        assert!(f32s_are_equal(actual_min, expected_min));
        assert!(f32s_are_equal(actual_max, expected_max));
    }

    // Tests for normal_value_to_integer_range
    #[test]
    fn test_normal_value_to_integer_range_standard() {
        let normal_value = 0.5;
        let min = 0;
        let max = 10;

        let actual = normal_value_to_unsigned_integer_range(normal_value, min, max);
        let expected = 5;

        assert_eq!(actual, expected);
    }

    #[test]
    fn test_normal_value_to_integer_range_inverted() {
        let normal_value = 0.5;
        let min = 10;
        let max = 0; // Inverted

        let actual = normal_value_to_unsigned_integer_range(normal_value, min, max);
        let expected = 5; // Should swap and produce correct result

        assert_eq!(actual, expected);
    }

    #[test]
    fn test_normal_value_to_integer_range_rounding() {
        let min = 0;
        let max = 10;

        // 0.24 * 10 = 2.4, rounds to 2
        let actual_low = normal_value_to_unsigned_integer_range(0.24, min, max);
        let expected_low = 2;

        // 0.26 * 10 = 2.6, rounds to 3
        let actual_high = normal_value_to_unsigned_integer_range(0.26, min, max);
        let expected_high = 3;

        assert_eq!(actual_low, expected_low);
        assert_eq!(actual_high, expected_high);
    }

    #[test]
    fn test_normal_value_to_integer_range_clamping() {
        let min = 0;
        let max = 10;

        // Values outside 0-1 should be clamped
        let actual_below = normal_value_to_unsigned_integer_range(-0.5, min, max);
        let expected_below = 0;

        let actual_above = normal_value_to_unsigned_integer_range(1.5, min, max);
        let expected_above = 10;

        assert_eq!(actual_below, expected_below);
        assert_eq!(actual_above, expected_above);
    }

    // Tests for normal_value_to_unsigned_integer_range (i32)
    #[test]
    fn test_normal_value_to_unsigned_integer_range_standard() {
        let normal_value = 0.5;
        let min = -50;
        let max = 50;

        let actual = normal_value_to_signed_integer_range(normal_value, min, max);
        let expected = 0; // -50 + 0.5 * 100 = 0

        assert_eq!(actual, expected);
    }

    #[test]
    fn test_normal_value_to_unsigned_integer_range_inverted() {
        let normal_value = 0.5;
        let min = 50;
        let max = -50; // Inverted

        let actual = normal_value_to_signed_integer_range(normal_value, min, max);
        let expected = 0; // Should swap and produce correct result

        assert_eq!(actual, expected);
    }

    // Tests for normal_value_to_number_of_filter_poles
    #[test]
    fn test_normal_value_to_number_of_filter_poles_boundaries() {
        // At 0.0, should return 1 (clamped minimum)
        let actual_min = normal_value_to_number_of_filter_poles(0.0);
        let expected_min = 1;

        // At 1.0, should return 4 (NUMBER_OF_FILER_POLES)
        let actual_max = normal_value_to_number_of_filter_poles(1.0);
        let expected_max = 4;

        assert_eq!(actual_min, expected_min);
        assert_eq!(actual_max, expected_max);
    }

    #[test]
    fn test_normal_value_to_number_of_filter_poles_ceiling() {
        // 0.3 * 4 = 1.2, ceiling = 2
        let actual = normal_value_to_number_of_filter_poles(0.3);
        let expected = 2;

        assert_eq!(actual, expected);
    }

    // Tests for exponential_curve_envelope_time_from_normal_value
    #[test]
    fn test_envelope_time_zero_value() {
        let normal_value = 0.0;
        let crossover = (0.5, 700.0);
        let min = 0;
        let max = 10000;

        let actual =
            exponential_curve_envelope_time_from_normal_value(normal_value, crossover, min, max);
        let expected = 0;

        assert_eq!(actual, expected);
    }

    #[test]
    fn test_envelope_time_below_crossover() {
        let normal_value = 0.25; // Below 0.5 crossover
        let crossover = (0.5, 700.0);
        let min = 0;
        let max = 10000;

        let actual =
            exponential_curve_envelope_time_from_normal_value(normal_value, crossover, min, max);
        // Renormalized: 0.25 / 0.5 = 0.5
        // Quadratic: 0.5^2 = 0.25
        // Result: 0 + (700 - 0) * 0.25 = 175
        let expected = 175;

        assert_eq!(actual, expected);
    }

    #[test]
    fn test_envelope_time_at_crossover() {
        let normal_value = 0.5;
        let crossover = (0.5, 700.0);
        let min = 0;
        let max = 10000;

        let actual =
            exponential_curve_envelope_time_from_normal_value(normal_value, crossover, min, max);
        // At crossover point, renormalized value is 0.0, quadratic is 0.0
        // Result: 700 + (10000 - 700) * 0.0 = 700
        let expected = 700;

        assert_eq!(actual, expected);
    }

    #[test]
    fn test_envelope_time_above_crossover() {
        let normal_value = 0.75; // Above 0.5 crossover
        let crossover = (0.5, 700.0);
        let min = 0;
        let max = 10000;

        let actual =
            exponential_curve_envelope_time_from_normal_value(normal_value, crossover, min, max);
        // Renormalized: (0.75 - 0.5) / 0.5 = 0.5
        // Quadratic: 0.5^2 = 0.25
        // Result: 700 + (10000 - 700) * 0.25 = 700 + 2325 = 3025
        let expected = 3025;

        assert_eq!(actual, expected);
    }

    // Tests for velocity_curve_from_normal_value
    #[test]
    fn test_velocity_curve_zero() {
        let normal_value = 0.0;

        let actual = velocity_curve_from_normal_value(normal_value);
        let expected = 0.0;

        assert!(f32s_are_equal(actual, expected));
    }

    #[test]
    fn test_velocity_curve_below_midpoint() {
        let normal_value = 0.25; // Below 0.5 midpoint

        let actual = velocity_curve_from_normal_value(normal_value);
        // Renormalized: 0.25 / 0.5 = 0.5
        // Maps 0.5 in range [0.25, 1.0]
        // Result: 0.25 + 0.5 * (1.0 - 0.25) = 0.25 + 0.375 = 0.625
        let expected = 0.625;

        assert!(f32s_are_equal(actual, expected));
    }

    #[test]
    fn test_velocity_curve_at_midpoint() {
        let normal_value = 0.5;

        let actual = velocity_curve_from_normal_value(normal_value);
        // At midpoint, renormalized to 1.0, maps to LINEAR_VELOCITY_CURVE_EXPONENT
        let expected = LINEAR_VELOCITY_CURVE_EXPONENT; // 1.0

        assert!(f32s_are_equal(actual, expected));
    }

    #[test]
    fn test_velocity_curve_above_midpoint() {
        let normal_value = 0.75; // Above 0.5 midpoint

        let actual = velocity_curve_from_normal_value(normal_value);
        // Renormalized: (0.75 - 0.5) / 0.5 = 0.5
        // Maps 0.5 in range [1.0, 4.0]
        // Result: 1.0 + 0.5 * (4.0 - 1.0) = 1.0 + 1.5 = 2.5
        let expected = 2.5;

        assert!(f32s_are_equal(actual, expected));
    }

    // Tests for scaled_velocity_from_normal_value
    #[test]
    fn test_scaled_velocity_linear_curve() {
        let velocity_curve = 1.0; // Linear
        let velocity = 0.5;

        let actual = scaled_velocity_from_normal_value(velocity_curve, velocity);
        let expected = 0.5; // No transformation

        assert!(f32s_are_equal(actual, expected));
    }

    #[test]
    fn test_scaled_velocity_zero_curve() {
        let velocity_curve = 0.0;
        let velocity = 0.5;

        let actual = scaled_velocity_from_normal_value(velocity_curve, velocity);
        let expected = MAX_MIDI_KEY_VELOCITY; // 1.0

        assert!(f32s_are_equal(actual, expected));
    }

    #[test]
    fn test_scaled_velocity_power_curve() {
        let velocity_curve = 2.0; // Squared
        let velocity = 0.5;

        let actual = scaled_velocity_from_normal_value(velocity_curve, velocity);
        let expected = 0.25; // 0.5^2

        assert!(f32s_are_equal(actual, expected));
    }

    // Tests for midi_value_to_pitch_bend_cents
    #[test]
    fn test_pitch_bend_at_zero_point() {
        let pitch_bend_amount = PITCH_BEND_AMOUNT_ZERO_POINT; // 8192
        let max_bend = 200; // 2 semitones

        let actual = midi_value_to_pitch_bend_cents(pitch_bend_amount, max_bend);
        let expected = 0; // No bend at zero point

        assert_eq!(actual, expected);
    }

    #[test]
    fn test_pitch_bend_max_up() {
        let pitch_bend_amount = PITCH_BEND_AMOUNT_MAX_VALUE; // 16383
        let max_bend = 200; // 2 semitones

        let actual = midi_value_to_pitch_bend_cents(pitch_bend_amount, max_bend);
        // (16383 - 8192) / 8192 * 200 = 8191/8192 * 200 ≈ 199.975... rounds to 199
        let expected = 199;

        assert_eq!(actual, expected);
    }

    #[test]
    fn test_pitch_bend_min_down() {
        let pitch_bend_amount = 0;
        let max_bend = 200; // 2 semitones

        let actual = midi_value_to_pitch_bend_cents(pitch_bend_amount, max_bend);
        // (0 - 8192) / 8192 * 200 = -1.0 * 200 = -200
        let expected = -200;

        assert_eq!(actual, expected);
    }

    #[test]
    fn test_pitch_bend_halfway_up() {
        let pitch_bend_amount = 12288; // Halfway between zero and max
        let max_bend = 200;

        let actual = midi_value_to_pitch_bend_cents(pitch_bend_amount, max_bend);
        // (12288 - 8192) / 8192 * 200 = 4096/8192 * 200 = 0.5 * 200 = 100
        let expected = 100;

        assert_eq!(actual, expected);
    }

    // Tests for exponential curve functions (zero check edge cases)
    #[test]
    fn test_exponential_filter_cutoff_zero() {
        let normal_value = 0.0;

        let actual = exponential_curve_filter_cutoff_from_midi_value(normal_value);
        let expected = 0.0;

        assert!(f32s_are_equal(actual, expected));
    }

    #[test]
    fn test_exponential_lfo_frequency_zero() {
        let normal_value = 0.0;

        let actual = exponential_curve_lfo_frequency_from_normal_value(normal_value);
        let expected = 0.0;

        assert!(f32s_are_equal(actual, expected));
    }

    #[test]
    fn test_exponential_level_adjustment_zero() {
        let normal_value = 0.0;

        let actual = exponential_curve_level_adjustment_from_normal_value(normal_value);
        let expected = 0.0;

        assert!(f32s_are_equal(actual, expected));
    }
}
