const MAX_MIDI_NOTE_NUMBER: i8 = 127;
const MIN_MIDI_NOTE_NUMBER: i8 = 0;
const MAX_NOTE_FREQUENCY: f32 = 12543.854;
const MIN_NOTE_FREQUENCY: f32 = 8.175;

pub fn tune(
    mut note_number: u8,
    interval: Option<i8>,
    cents: Option<i16>,
    pitch_bend: Option<i16>,
) -> f32 {
    if let Some(interval) = interval {
        note_number = (note_number as i8)
            .saturating_add(interval)
            .clamp(MIN_MIDI_NOTE_NUMBER, MAX_MIDI_NOTE_NUMBER) as u8;
    }

    let mut note_frequency = midi_note_to_frequency(note_number);

    if let Some(pitch_bend_cents) = pitch_bend {
        note_frequency = frequency_from_cents(note_frequency, pitch_bend_cents)
            .clamp(MIN_NOTE_FREQUENCY, MAX_NOTE_FREQUENCY);
    }

    if let Some(cents) = cents {
        note_frequency = frequency_from_cents(note_frequency, cents);
    }
    note_frequency
}

fn midi_note_to_frequency(note_number: u8) -> f32 {
    MIDI_NOTE_FREQUENCIES[note_number as usize].0
}

fn frequency_from_cents(frequency: f32, cents: i16) -> f32 {
    frequency * (2.0f32.powf(cents as f32 / 1200.0))
}

pub const MIDI_NOTE_FREQUENCIES: [(f32, &str); 128] = [
    (8.175, "C-1"),
    (8.662, "C#-1/Db-1"),
    (9.177, "D-1"),
    (9.722, "D#-1/Eb-1"),
    (10.300, "E-1"),
    (10.913, "F-1"),
    (11.562, "F#-1/Gb-1"),
    (12.249, "G-1"),
    (12.978, "G#-1/Ab-1"),
    (13.750, "A-1"),
    (14.567, "A#-1/Bb-1"),
    (15.433, "B-1"),
    (16.351, "C0"),
    (17.323, "C#0/Db0"),
    (18.354, "D0"),
    (19.445, "D#0/Eb0"),
    (20.601, "E0"),
    (21.826, "F0"),
    (23.124, "F#0/Gb0"),
    (24.499, "G0"),
    (25.956, "G#0/Ab0"),
    (27.500, "A0"),
    (29.135, "A#0/Bb0"),
    (30.867, "B0"),
    (32.703, "C1"),
    (34.647, "C#1/Db1"),
    (36.708, "D1"),
    (38.890, "D#1/Eb1"),
    (41.203, "E1"),
    (43.653, "F1"),
    (46.249, "F#1/Gb1"),
    (48.999, "G1"),
    (51.913, "G#1/Ab1"),
    (55.000, "A1"),
    (58.270, "A#1/Bb1"),
    (61.735, "B1"),
    (65.406, "C2"),
    (69.295, "C#2/Db2"),
    (73.416, "D2"),
    (77.781, "D#2/Eb2"),
    (82.406, "E2"),
    (87.307, "F2"),
    (92.498, "F#2/Gb2"),
    (97.998, "G2"),
    (103.826, "G#2/Ab2"),
    (110.000, "A2"),
    (116.540, "A#2/Bb2"),
    (123.470, "B2"),
    (130.812, "C3"), // 48
    (138.591, "C#3/Db3"),
    (146.832, "D3"),
    (155.563, "D#3/Eb3"),
    (164.813, "E3"),
    (174.614, "F3"),
    (184.997, "F#3/Gb3"),
    (195.997, "G3"),
    (207.652, "G#3/Ab3"),
    (220.000, "A3"),
    (233.081, "A#3/Bb3"),
    (246.941, "B3"),
    (261.625, "C4 (middle C)"), // 60
    (277.182, "C#4/Db4"),
    (293.664, "D4"),
    (311.127, "D#4/Eb4"),
    (329.627, "E4"),
    (349.228, "F4"),
    (369.994, "F#4/Gb4"),
    (391.995, "G4"),
    (415.304, "G#4/Ab4"),
    (440.000, "A4"),
    (466.163, "A#4/Bb4"),
    (493.883, "B4"),
    (523.251, "C5"), // 72
    (554.365, "C#5/Db5"),
    (587.329, "D5"),
    (622.254, "D#5/Eb5"),
    (659.255, "E5"),
    (698.456, "F5"),
    (739.988, "F#5/Gb5"),
    (783.990, "G5"),
    (830.609, "G#5/Ab5"),
    (880.000, "A5"),
    (932.327, "A#5/Bb5"),
    (987.766, "B5"),
    (1046.502, "C6"), // 84
    (1_108.73, "C#6/Db6"),
    (1174.659, "D6"),
    (1244.507, "D#6/Eb6"),
    (1_318.51, "E6"),
    (1396.912, "F6"),
    (1479.977, "F#6/Gb6"),
    (1567.981, "G6"),
    (1661.218, "G#6/Ab6"),
    (1760.000, "A6"),
    (1864.655, "A#6/Bb6"),
    (1975.533, "B6"),
    (2093.004, "C7"), // 96
    (2217.461, "C#7/Db7"),
    (2349.318, "D7"),
    (2489.015, "D#7/Eb7"),
    (2_637.02, "E7"),
    (2793.825, "F7"),
    (2959.955, "F#7/Gb7"),
    (3135.963, "G7"),
    (3322.437, "G#7/Ab7"),
    (3520.000, "A7"),
    (3_729.31, "A#7/Bb7"),
    (3951.066, "B7"),
    (4186.009, "C8"),
    (4434.922, "C#8/Db8"),
    (4698.636, "D8"),
    (4978.031, "D#8/Eb8"),
    (5_274.04, "E8"),
    (5587.651, "F8"),
    (5_919.91, "F#8/Gb8"),
    (6271.927, "G8"),
    (6644.875, "G#8/Ab8"),
    (7040.000, "A8"),
    (7_458.62, "A#8/Bb8"),
    (7902.132, "B8"),
    (8372.018, "C9"),
    (8869.844, "C#9/Db9"),
    (9397.272, "D9"),
    (9956.063, "D#9/Eb9"),
    (10548.081, "E9"),
    (11175.303, "F9"),
    (11839.821, "F#9/Gb9"),
    (12543.854, "G9"),
];

#[cfg(test)]
mod tests {
    use super::*;

    fn f32_value_equality(value_1: f32, value_2: f32) -> bool {
        (value_1 - value_2).abs() <= f32::EPSILON
    }

    #[test]
    fn midi_note_to_frequency_returns_correct_values_for_note_numbers() {
        let notes: [u8; 4] = [0, 21, 72, 127];
        let expected_frequencies: [f32; 4] = [8.175, 27.5, 523.251, 12543.854];

        for i in 0..notes.len() {
            assert!(f32_value_equality(
                midi_note_to_frequency(notes[i]),
                expected_frequencies[i]
            ));
        }
    }

    #[test]
    fn frequency_from_cents_returns_correct_values_for_value_frequencies_and_cents() {
        let frequencies: [f32; 4] = [8.175, 27.5, 523.251, 12543.854];
        let cents = 50;
        let expected_frequencies: [f32; 4] = [8.414546, 28.30581, 538.5834, 12911.417];

        for i in 0..frequencies.len() {
            assert!(f32_value_equality(
                frequency_from_cents(frequencies[i], cents),
                expected_frequencies[i]
            ));
        }
    }

    #[test]
    fn frequency_from_cents_returns_correct_values_for_value_frequencies_and_negative_cents() {
        let frequencies: [f32; 4] = [8.175, 27.5, 523.251, 12543.854];
        let cents = -50;
        let expected_frequencies: [f32; 4] = [7.9422736, 26.717129, 508.35504, 12186.754];

        for i in 0..frequencies.len() {
            assert!(f32_value_equality(
                frequency_from_cents(frequencies[i], cents),
                expected_frequencies[i]
            ));
        }
    }

    #[test]
    fn tune_returns_correct_values_without_interval_or_cents() {
        let note_number = 60;
        let interval = None;
        let cents = None;
        let pitch_bend = None;
        let expected_frequency = 261.625;
        assert!(f32_value_equality(
            tune(note_number, interval, cents, pitch_bend),
            expected_frequency
        ));
    }

    #[test]
    fn tune_returns_correct_values_with_interval_but_no_cents() {
        let note_number = 60;
        let interval = Some(7);
        let cents = None;
        let pitch_bend = None;
        let expected_frequency = 391.995;
        assert!(f32_value_equality(
            tune(note_number, interval, cents, pitch_bend),
            expected_frequency
        ));
    }

    #[test]
    fn tune_returns_correct_values_without_interval_but_with_cents() {
        let note_number = 60;
        let interval = None;
        let cents = Some(-12);
        let pitch_bend = None;
        let expected_frequency = 259.8178;
        assert!(f32_value_equality(
            tune(note_number, interval, cents, pitch_bend),
            expected_frequency
        ));
    }

    #[test]
    fn tune_returns_correct_values_with_interval_and_cents() {
        let note_number = 60;
        let interval = Some(7);
        let cents = Some(-12);
        let pitch_bend = None;
        let expected_frequency = 389.2873;
        assert!(f32_value_equality(
            tune(note_number, interval, cents, pitch_bend),
            expected_frequency
        ));
    }

    #[test]
    fn tune_correctly_clamps_note_plus_interval_range_minimum_to_zero() {
        let note_number = 0;
        let interval = Some(-127);
        let cents = None;
        let pitch_bend = None;
        let expected_frequency = 8.175;
        assert!(f32_value_equality(
            tune(note_number, interval, cents, pitch_bend),
            expected_frequency
        ));
    }

    #[test]
    fn tune_correctly_clamps_note_plus_interval_range_maximum_to_127() {
        let note_number = 127;
        let interval = Some(127);
        let cents = None;
        let pitch_bend = None;
        let expected_frequency = 12543.854;
        assert!(f32_value_equality(
            tune(note_number, interval, cents, pitch_bend),
            expected_frequency
        ));
    }
}
