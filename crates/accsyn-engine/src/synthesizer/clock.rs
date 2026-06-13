use serde::{Deserialize, Serialize};
use std::sync::atomic::AtomicU16;
use std::time::Duration;

const PULSES_PER_THIRTY_SECOND_NOTE: u8 = 3; // 24 PPQN / 8 thirty-second notes per quarter note;
const MIN_BEATS_PER_MINUTE: f64 = 20.0;
const MAX_BEATS_PER_MINUTE: f64 = 400.0;

/// Parameters for the clock and clock synchronization
#[derive(Debug, Serialize, Deserialize, Default)]
pub struct ClockParameters {
    pub(crate) bpm: AtomicU16,
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
    let beats_per_minute = 60_000_000.0 / length_of_beat_in_microseconds as f64;
    beats_per_minute
        .round()
        .clamp(MIN_BEATS_PER_MINUTE, MAX_BEATS_PER_MINUTE) as u16
}
