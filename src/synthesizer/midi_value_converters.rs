use crate::modules::oscillator::{NUMBER_OF_WAVE_SHAPES, OscillatorParameters};
use crate::synthesizer::constants::{
    CENTS_PER_SEMITONE, ENVELOPE_MAX_MILLISECONDS, ENVELOPE_MIN_MILLISECONDS,
    EXPONENTIAL_FILTER_COEFFICIENT, EXPONENTIAL_LEVEL_COEFFICIENT, EXPONENTIAL_LFO_COEFFICIENT,
    LEVEL_CURVE_LINEAR_RANGE, MIDI_CENTER_VALUE, MIDI_SWITCH_MAX_OFF_VALUE, MIDI_VALUE_RANGE,
    PITCH_BEND_AMOUNT_MAX_VALUE, PITCH_BEND_AMOUNT_ZERO_POINT,
};
use std::cmp::Ordering;
use std::sync::atomic::Ordering::Relaxed;


pub fn midi_value_to_f32_range(midi_value: u8, minimum: f32, maximum: f32) -> f32 {
    let range = maximum - minimum;
    minimum + (f32::from(midi_value) * range / f32::from(MIDI_VALUE_RANGE))
}

pub fn midi_value_to_u8_range(midi_value: u8, mut minimum: u8, mut maximum: u8) -> u8 {
    if maximum < minimum {
        core::mem::swap(&mut minimum, &mut maximum);
    }

    let target_range = u16::from(maximum - minimum);
    let output_range_value = u16::from(midi_value.min(MIDI_VALUE_RANGE)) * target_range / u16::from(MIDI_VALUE_RANGE);
    minimum + output_range_value as u8
}

pub fn midi_value_to_i8_range(midi_value: u8, mut minimum: i8, mut maximum: i8) -> i8 {
    if maximum < minimum {
        core::mem::swap(&mut minimum, &mut maximum);
    }

    let target_range = i16::from(maximum - minimum);
    let output_range_value = i16::from(midi_value) * target_range / i16::from(MIDI_VALUE_RANGE);
    minimum + output_range_value as i8
}

pub fn midi_value_to_u16_range(midi_value: u8, mut minimum: u16, mut maximum: u16) -> u16 {
    if maximum < minimum {
        core::mem::swap(&mut minimum, &mut maximum);
    }

    let target_range = u32::from(maximum - minimum);
    let output_range_value = u32::from(midi_value) * target_range / u32::from(MIDI_VALUE_RANGE);
    minimum + output_range_value as u16
}

pub fn midi_value_to_u32_range(midi_value: u8, mut minimum: u32, mut maximum: u32) -> u32 {
    if maximum < minimum {
        core::mem::swap(&mut minimum, &mut maximum);
    }

    let target_range = u64::from(maximum - minimum);
    let output_range_value = u64::from(midi_value) * target_range / u64::from(MIDI_VALUE_RANGE);
    minimum + output_range_value as u32
}

pub fn midi_value_to_f32_0_to_1(midi_value: u8) -> f32 {
    midi_value_to_f32_range(midi_value, 0.0, 1.0)
}

pub fn midi_value_to_f32_negative_1_to_1(midi_value: u8) -> f32 {
    midi_value_to_f32_range(midi_value, -1.0, 1.0)
}

pub fn midi_value_to_bool(midi_value: u8) -> bool {
    midi_value > MIDI_SWITCH_MAX_OFF_VALUE
}

pub fn midi_value_to_envelope_milliseconds(midi_value: u8) -> u32 {
    midi_value_to_u32_range(
        midi_value,
        ENVELOPE_MIN_MILLISECONDS,
        ENVELOPE_MAX_MILLISECONDS,
    )
}

pub fn midi_value_to_number_of_filter_poles(midi_value: u8) -> u8 {
    if midi_value < 32 {
        1
    } else if midi_value < 64 {
        2
    } else if midi_value < 96 {
        3
    } else {
        4
    }
}

pub fn midi_value_to_wave_shape_index(midi_value: u8) -> u8 {
    midi_value_to_u8_range(midi_value, 1, NUMBER_OF_WAVE_SHAPES)
}

pub fn exponential_curve_filter_cutoff_from_midi_value(midi_value: u8) -> f32 {
    if midi_value == 0 {
        return 0.0;
    }
    exponential_curve_from_midi_value_and_coefficient(midi_value, EXPONENTIAL_FILTER_COEFFICIENT)
}

pub fn exponential_curve_lfo_frequency_from_midi_value(midi_value: u8) -> f32 {
    if midi_value == 0 {
        return 0.0;
    }
    exponential_curve_from_midi_value_and_coefficient(midi_value, EXPONENTIAL_LFO_COEFFICIENT)
        / 100.0
}

pub fn exponential_curve_level_adjustment_from_midi_value(midi_value: u8) -> f32 {
    if midi_value == 0 {
        return 0.0;
    }
    exponential_curve_from_midi_value_and_coefficient(midi_value, EXPONENTIAL_LEVEL_COEFFICIENT)
        / LEVEL_CURVE_LINEAR_RANGE
}

fn exponential_curve_from_midi_value_and_coefficient(
    midi_value: u8,
    exponential_coefficient: f32,
) -> f32 {
    // exponential_coefficient is the log of the effective range for the linear scale you want to map to exponential range
    // If the range max is 1000x then min, then the exponential_coefficient is log(1000) = 6.908
    (exponential_coefficient * (f32::from(midi_value) / f32::from(MIDI_VALUE_RANGE))).exp()
}

pub fn continuously_variable_curve_mapping_from_midi_value(
    mut slope_midi_value: u8,
    input_midi_value: u8,
) -> f32 {
    if slope_midi_value == 0 {
        slope_midi_value = 1;
    }

    let curve_exponent = match slope_midi_value.cmp(&MIDI_CENTER_VALUE) {
        Ordering::Less => f32::from(slope_midi_value) / 64f32,
        Ordering::Greater => (f32::from(slope_midi_value - MIDI_CENTER_VALUE) / 63f32) * 7.0,
        Ordering::Equal => 1.0,
    };

    f32::from(input_midi_value).powf(curve_exponent)
        / f32::from(MIDI_VALUE_RANGE).powf(curve_exponent)
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
    ((i32::from(pitch_bend_amount) - i32::from(PITCH_BEND_AMOUNT_ZERO_POINT))
        / i32::from(PITCH_BEND_AMOUNT_ZERO_POINT)
        * i32::from(max_bend_in_cents)) as i16
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::math::f32s_are_equal;

    #[test]
    fn midi_value_to_f32_range_correctly_maps_edge_values() {
        let value_zero = midi_value_to_f32_range(0, 0.0, 1.0);
        let expected_result = 0.0;
        assert!(f32s_are_equal(value_zero, expected_result), "Expected: {expected_result} but got {value_zero}");

        let value_max = midi_value_to_f32_range(127, 0.0, 1.0);
        let expected_result = 1.0;
        assert!(f32s_are_equal(value_max, 1.0), "Expected: {expected_result} but got {value_max}");

        let value_zero_negative_range = midi_value_to_f32_range(0, -1.0, 1.0);
        let expected_result = 1.0;
        assert!(f32s_are_equal(value_zero_negative_range, -1.0), "Expected: {expected_result} but got {value_zero_negative_range}");

        let value_max_negative_range = midi_value_to_f32_range(127, -1.0, 1.0);
        let expected_result = 1.0;
        assert!(f32s_are_equal(value_max_negative_range, 1.0), "Expected: {expected_result} but got {value_max_negative_range}");
    }

    #[test]
    fn midi_value_to_u16_range_returns_correct_value_for_valid_input() {
        let test_value = 64;
        let test_minimum = 100;
        let test_maximum = 200;
        let expected_output = 150;
        let output = midi_value_to_u16_range(test_value, test_minimum, test_maximum);
        assert_eq!(output, expected_output);
    }

    #[test]
    fn midi_value_to_i8_range_returns_correct_value_for_valid_input() {
        let test_value = 127;
        let test_minimum = -10;
        let test_maximum = 100;
        let expected_output = 100;
        let output = midi_value_to_i8_range(test_value, test_minimum, test_maximum);
        assert_eq!(output, expected_output);
    }

    #[test]
    fn midi_value_to_u8_range_returns_correct_value_for_valid_input() {
        let test_value = 127;
        let test_minimum = 10;
        let test_maximum = 100;
        let expected_output = 100;
        let output = midi_value_to_u8_range(test_value, test_minimum, test_maximum);
        assert_eq!(output, expected_output);
    }

    #[test]
    fn midi_value_to_u32_range_returns_correct_value_for_valid_input() {
        let test_value = 64;
        let test_minimum = 100;
        let test_maximum = 200;
        let expected_output = 150;
        let output = midi_value_to_u32_range(test_value, test_minimum, test_maximum);
        assert_eq!(output, expected_output);
    }

    #[test]
    fn midi_value_to_u32_range_returns_correct_value_for_valid_but_reversed_min_max_inputs() {
        let test_value = 64;
        let test_minimum = 200;
        let test_maximum = 100;
        let expected_output = 150;
        let output = midi_value_to_u32_range(test_value, test_minimum, test_maximum);
        assert_eq!(output, expected_output);
    }

    #[test]
    fn midi_value_to_f32_range_correctly_maps_middle_values() {
        let middle_value1 = midi_value_to_f32_range(64, 0.0, 1.0);
        assert!(middle_value1 > 0.5 && middle_value1 < 0.51);

        let middle_value2 = midi_value_to_f32_range(64, -1.0, 1.0);
        assert!(middle_value2 > 0.0 && middle_value2 < 0.01);

        let middle_value3 = midi_value_to_f32_range(64, 20.0, 800.0);
        assert!(middle_value3 > 410.0 && middle_value3 < 415.0);

        let middle_value4 = midi_value_to_f32_range(64, -800.0, 20.0);
        assert!(middle_value4 < -386.0 && middle_value1 > -387.0);
    }

    #[test]
    fn midi_value_to_f32_0_to_1_correctly_maps_values() {
        assert!(f32s_are_equal(midi_value_to_f32_0_to_1(0), 0.0));
        assert!(f32s_are_equal(midi_value_to_f32_0_to_1(127), 1.0));
    }

    #[test]
    fn midi_value_to_f32_negative_1_to_1_correctly_maps_values() {
        assert!(f32s_are_equal(midi_value_to_f32_negative_1_to_1(0), -1.0));

        assert!(f32s_are_equal(
            midi_value_to_f32_negative_1_to_1(12),
            -0.811_023_6
        ));
        assert!(f32s_are_equal(midi_value_to_f32_negative_1_to_1(127), 1.0));
    }

    #[test]
    fn midi_value_to_bool_correctly_converts_threshold() {
        assert!(!midi_value_to_bool(0));
        assert!(!midi_value_to_bool(63));
        assert!(midi_value_to_bool(64));
        assert!(midi_value_to_bool(127));
    }
}
