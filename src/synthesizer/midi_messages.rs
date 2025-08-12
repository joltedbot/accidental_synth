use crate::modules::envelope::Envelope;
use crate::modules::filter::FilterSlope;
use crate::synthesizer::MidiNoteEvent;
use crate::synthesizer::constants::{MAXIMUM_FILTER_CUTOFF, MINIMUM_FILTER_CUTOFF};
use std::sync::MutexGuard;

pub fn process_midi_note_events(
    midi_events: Option<MidiNoteEvent>,
    amp_envelope: &mut MutexGuard<Envelope>,
    filter_envelope: &mut MutexGuard<Envelope>,
) {
    match midi_events {
        Some(MidiNoteEvent::NoteOn) => {
            amp_envelope.gate_on();
            filter_envelope.gate_on();
        }
        Some(MidiNoteEvent::NoteOff) => {
            amp_envelope.gate_off();
            filter_envelope.gate_off();
        }
        None => {}
    }
}

pub fn midi_value_to_f32_range(midi_value: u8, minimum: f32, maximum: f32) -> f32 {
    let range = maximum - minimum;
    let increment = range / 127.0;
    minimum + (midi_value as f32 * increment)
}

pub fn midi_value_to_f32_0_to_1(midi_value: u8) -> f32 {
    midi_value_to_f32_range(midi_value, 0.0, 1.0)
}

pub fn midi_value_to_f32_negative_1_to_1(midi_value: u8) -> f32 {
    midi_value_to_f32_range(midi_value, -1.0, 1.0)
}

pub fn midi_value_to_bool(midi_value: u8) -> bool {
    midi_value > 63
}

pub fn midi_value_to_filter_slope(midi_value: u8) -> FilterSlope {
    if midi_value < 32 {
        FilterSlope::Db6
    } else if midi_value < 64 {
        FilterSlope::Db12
    } else if midi_value < 96 {
        FilterSlope::Db18
    } else {
        FilterSlope::Db24
    }
}

pub fn midi_value_to_filter_cutoff(midi_value: u8) -> f32 {
    midi_value_to_f32_range(midi_value, MINIMUM_FILTER_CUTOFF, MAXIMUM_FILTER_CUTOFF)
}

#[cfg(test)]
mod tests {

    use super::*;

    fn f32_value_equality(value_1: f32, value_2: f32) -> bool {
        (value_1 - value_2).abs() <= f32::EPSILON
    }

    #[test]
    fn midi_value_to_f32_range_correctly_maps_edge_values() {
        assert!(f32_value_equality(
            midi_value_to_f32_range(0, 0.0, 1.0),
            0.0
        ));
        assert!(f32_value_equality(
            midi_value_to_f32_range(127, 0.0, 1.0),
            1.0
        ));
        assert!(f32_value_equality(
            midi_value_to_f32_range(0, -1.0, 1.0),
            -1.0
        ));
        assert!(f32_value_equality(
            midi_value_to_f32_range(127, -1.0, 1.0),
            1.0
        ));
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
        assert!(f32_value_equality(midi_value_to_f32_0_to_1(0), 0.0));
        assert!(f32_value_equality(midi_value_to_f32_0_to_1(127), 1.0));
    }

    #[test]
    fn midi_value_to_f32_negative_1_to_1_correctly_maps_values() {
        assert!(f32_value_equality(
            midi_value_to_f32_negative_1_to_1(0),
            -1.0
        ));

        assert!(f32_value_equality(
            midi_value_to_f32_negative_1_to_1(12),
            -0.8110236
        ));
        assert!(f32_value_equality(
            midi_value_to_f32_negative_1_to_1(127),
            1.0
        ));
    }

    #[test]
    fn midi_value_to_bool_correctly_converts_threshold() {
        assert!(!midi_value_to_bool(0));
        assert!(!midi_value_to_bool(63));
        assert!(midi_value_to_bool(64));
        assert!(midi_value_to_bool(127));
    }
}
