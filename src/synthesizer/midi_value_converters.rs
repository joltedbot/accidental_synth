use crate::modules::oscillator::{NUMBER_OF_WAVE_SHAPES, OscillatorParameters};
use crate::synthesizer::constants::{
    ENVELOPE_MAX_MILLISECONDS, ENVELOPE_MIN_MILLISECONDS, EXPONENTIAL_FILTER_COEFFICIENT,
    EXPONENTIAL_LEVEL_COEFFICIENT, EXPONENTIAL_LFO_COEFFICIENT, LEVEL_CURVE_LINEAR_RANGE,
    MAX_MIDI_VALUE, MIDI_CENTER_VALUE, MIDI_SWITCH_MAX_OFF_VALUE,
    OSCILLATOR_COURSE_TUNE_MAX_INTERVAL, OSCILLATOR_COURSE_TUNE_MIN_INTERVAL,
    OSCILLATOR_FINE_TUNE_MAX_CENTS, OSCILLATOR_FINE_TUNE_MIN_CENTS, PITCH_BEND_AMOUNT_CENTS,
    PITCH_BEND_AMOUNT_MAX_VALUE, PITCH_BEND_AMOUNT_ZERO_POINT,
};
use std::cmp::Ordering;
use std::sync::atomic::Ordering::Relaxed;

pub fn midi_value_to_f32_range(midi_value: u8, minimum: f32, maximum: f32) -> f32 {
    let range = maximum - minimum;
    let increment = range / f32::from(MAX_MIDI_VALUE);
    minimum + (f32::from(midi_value) * increment)
}

pub fn midi_value_to_u32_range(midi_value: u8, minimum: u32, maximum: u32) -> u32 {
    let range = maximum - minimum;
    let increment = range as f32 / f32::from(MAX_MIDI_VALUE);
    minimum + (f32::from(midi_value) * increment).ceil() as u32
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

pub fn midi_value_to_fine_tune_cents(midi_value: u8) -> i16 {
    midi_value_to_f32_range(
        midi_value,
        f32::from(OSCILLATOR_FINE_TUNE_MIN_CENTS),
        f32::from(OSCILLATOR_FINE_TUNE_MAX_CENTS),
    ) as i16
}

pub fn midi_value_to_course_tune_intervals(midi_value: u8) -> i8 {
    midi_value_to_f32_range(
        midi_value,
        f32::from(OSCILLATOR_COURSE_TUNE_MIN_INTERVAL),
        f32::from(OSCILLATOR_COURSE_TUNE_MAX_INTERVAL),
    ) as i8
}

pub fn midi_value_to_envelope_milliseconds(midi_value: u8) -> f32 {
    midi_value_to_f32_range(
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
    midi_value_to_u32_range(midi_value, 1, u32::from(NUMBER_OF_WAVE_SHAPES)) as u8
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
    (exponential_coefficient * (f32::from(midi_value) / f32::from(MAX_MIDI_VALUE))).exp()
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
        / f32::from(MAX_MIDI_VALUE).powf(curve_exponent)
}

pub fn update_current_note_from_midi_pitch_bend(
    pitch_bend_amount: i16,
    oscillators: &[OscillatorParameters; 4],
) {
    for oscillator in oscillators {
        if pitch_bend_amount == PITCH_BEND_AMOUNT_ZERO_POINT {
            oscillator.pitch_bend.store(0, Relaxed);
        } else if pitch_bend_amount == PITCH_BEND_AMOUNT_MAX_VALUE {
            oscillator
                .pitch_bend
                .store(PITCH_BEND_AMOUNT_CENTS, Relaxed);
        } else {
            let pitch_bend_in_cents = f32::from(pitch_bend_amount - PITCH_BEND_AMOUNT_ZERO_POINT)
                / f32::from(PITCH_BEND_AMOUNT_ZERO_POINT)
                * f32::from(PITCH_BEND_AMOUNT_CENTS);
            oscillator
                .pitch_bend
                .store(pitch_bend_in_cents as i16, Relaxed);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::math::f32s_are_equal;

    #[test]
    fn midi_value_to_f32_range_correctly_maps_edge_values() {
        assert!(f32s_are_equal(midi_value_to_f32_range(0, 0.0, 1.0), 0.0));
        assert!(f32s_are_equal(midi_value_to_f32_range(127, 0.0, 1.0), 1.0));
        assert!(f32s_are_equal(midi_value_to_f32_range(0, -1.0, 1.0), -1.0));
        assert!(f32s_are_equal(midi_value_to_f32_range(127, -1.0, 1.0), 1.0));
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
