use crate::modules::filter::NUMBER_OF_FILER_POLES;
use crate::modules::oscillator::{NUMBER_OF_WAVE_SHAPES, OscillatorParameters};
use crate::synthesizer::constants::{
    CENTS_PER_SEMITONE, ENVELOPE_MAX_MILLISECONDS, ENVELOPE_MIN_MILLISECONDS,
    EXPONENTIAL_FILTER_COEFFICIENT, EXPONENTIAL_LEVEL_COEFFICIENT, EXPONENTIAL_LFO_COEFFICIENT,
    LEVEL_CURVE_LINEAR_RANGE, MAX_MIDI_VALUE, MIDI_CENTER_VALUE, NORMAL_TO_BOOL_SWITCH_ON_VALUE,
    PITCH_BEND_AMOUNT_MAX_VALUE, PITCH_BEND_AMOUNT_ZERO_POINT,
};
use std::cmp::Ordering;
use std::sync::atomic::Ordering::Relaxed;
/*

    midi module:
     - receives midi events and filters out unsupported messages
     - Passed supported midi messages to the synthesizer module

    synthesizer module:
     - receives the messages
     - filters the messages by type and sends to a processing function for each type
     - the processing function either directly processes the received value or for CCs breaks it down by cc to process
     - processing function: (midi_messages.rs)
       - normalize the midi value
       - wraps a more general processing function shared with the ui interaction
     - storing function:
       - takes the normal value and output range
       - translates the normal value to a scaled value for the property it represents
       - stores it to the correct location
       - calls a UI callback function that will talk to the UI module to update the UI with the stored real value

*/

pub fn normal_value_to_f32_range(normal_value: f32, mut minimum: f32, mut maximum: f32) -> f32 {
    if maximum < minimum {
        core::mem::swap(&mut minimum, &mut maximum);
    }

    let range = maximum - minimum;
    minimum + (normal_value * range)
}

pub fn normal_value_to_integer_range(normal_value: f32, mut minimum: u32, mut maximum: u32) -> u32 {
    if maximum < minimum {
        core::mem::swap(&mut minimum, &mut maximum);
    }

    let range = maximum - minimum;
    let clamped_value = normal_value.clamp(0.0, 1.0);
    let scaled_value = (f64::from(clamped_value) * f64::from(range)).round();

    (f64::from(minimum) + scaled_value).clamp(f64::from(minimum), f64::from(maximum)) as u32
}

pub fn normal_value_to_unsigned_integer_range(
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

pub fn normal_value_to_envelope_milliseconds(normal_value: f32) -> u32 {
    normal_value_to_integer_range(
        normal_value,
        ENVELOPE_MIN_MILLISECONDS,
        ENVELOPE_MAX_MILLISECONDS,
    )
}

pub fn normal_value_to_number_of_filter_poles(normal_value: f32) -> u8 {
    (f32::from(NUMBER_OF_FILER_POLES) * normal_value).ceil() as u8
}

pub fn normal_value_to_wave_shape_index(normal_value: f32) -> u8 {
    let first_shape_index = 1;
    normal_value_to_integer_range(
        normal_value,
        first_shape_index,
        u32::from(NUMBER_OF_WAVE_SHAPES),
    )
    .clamp(first_shape_index, u32::from(NUMBER_OF_WAVE_SHAPES)) as u8
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

pub fn exponential_curve_level_adjustment_from_normal_value(normal_value: f32) -> f32 {
    if normal_value == 0.0 {
        return 0.0;
    }
    exponential_curve_from_normal_value_and_coefficient(normal_value, EXPONENTIAL_LEVEL_COEFFICIENT)
        / LEVEL_CURVE_LINEAR_RANGE
}

fn exponential_curve_from_normal_value_and_coefficient(
    normal_value: f32,
    exponential_coefficient: f32,
) -> f32 {
    // exponential_coefficient is the log of the effective range for the linear scale you want to map to exponential range
    // If the range max is 1000x then min, then the exponential_coefficient is log(1000) = 6.908
    (exponential_coefficient * normal_value).exp()
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
    use crate::math::{f32s_are_equal, normalize_midi_value};

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
}
