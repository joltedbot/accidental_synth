use serde::{Deserialize, Serialize};
use std::sync::atomic::AtomicU16;
use std::time::Duration;

const PULSES_PER_THIRTY_SECOND_NOTE: u8 = 3; // 24 PPQN / 8 thirty-second notes per quarter note;
const MIN_BEATS_PER_MINUTE: f64 = 20.0;
const MAX_BEATS_PER_MINUTE: f64 = 400.0;
const MICROSECONDS_PER_MINUTE: f64 = 60.0 * 1_000_000.0;

/// Parameters for the clock and clock synchronization
#[derive(Debug, Serialize, Deserialize, Default)]
#[serde(default)]
pub struct ClockParameters {
    pub bpm: AtomicU16,
}

pub struct Clock {
    counter: u8,
}

impl Clock {
    pub fn new() -> Self {
        Self { counter: 0 }
    }

    pub fn tick_is_32nd_note(&mut self) -> bool {
        self.counter += 1;

        if self.counter == PULSES_PER_THIRTY_SECOND_NOTE {
            self.counter = 0;
            true
        } else {
            false
        }
    }
}

pub fn bpm_from_thirty_second_note_duration(length_of_thirty_second_note: Duration) -> u16 {
    let length_of_beat_in_microseconds = length_of_thirty_second_note.as_micros() * 8;

    // Beats per minute is constrained to a minimum value of 20 by convention and enforced by the application
    // That is 3_000_000_000 microseconds per beat well within an f64's range
    #[allow(clippy::cast_precision_loss)]
    let beats_per_minute = MICROSECONDS_PER_MINUTE / length_of_beat_in_microseconds as f64;

    // The value is clamped to constants that are well within u16 values 20-400
    #[allow(
        clippy::cast_possible_truncation,
        clippy::cast_sign_loss,
        clippy::cast_possible_truncation
    )]
    let clamped_beats_per_minute = beats_per_minute
        .round()
        .clamp(MIN_BEATS_PER_MINUTE, MAX_BEATS_PER_MINUTE)
        as u16;

    clamped_beats_per_minute
}
