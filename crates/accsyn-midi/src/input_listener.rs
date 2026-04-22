use crate::constants::{
    CC_MESSAGE_NUMBER_BYTE_INDEX, CC_MESSAGE_VALUE_BYTE_INDEX, CHANNEL_PRESSURE_VALUE_BYTE_INDEX,
    MESSAGE_STATUS_BYTE_CHANNEL_MASK, MESSAGE_STATUS_BYTE_INDEX, MESSAGE_STATUS_BYTE_TYPE_MASK,
    MESSAGE_TYPE_IGNORE_LIST, MIDI_INPUT_CLIENT_NAME, MIDI_INPUT_CONNECTION_NAME,
    NOTE_MESSAGE_NUMBER_BYTE_INDEX, NOTE_MESSAGE_VELOCITY_BYTE_INDEX,
    PITCH_BEND_MESSAGE_LSB_BYTE_INDEX, PITCH_BEND_MESSAGE_MSB_BYTE_INDEX,
    PROGRAM_CHANGE_VALUE_BYTE_INDEX, RAW_CHANNEL_TO_USER_READABLE_CHANNEL_OFFSET,
};
use crate::{Status, control_change};
use accsyn_types::midi_events::MidiEvent;
use anyhow::Result;
use crossbeam_channel::Sender;
use midir::{MidiInput, MidiInputConnection, MidiInputPort};
use std::sync::{Arc, Mutex, PoisonError};

pub(crate) fn create_midi_input_listener(
    input_port: &MidiInputPort,
    current_channel_arc: Arc<Mutex<Option<u8>>>,
    midi_message_sender: Sender<MidiEvent>,
    current_note_arc: Arc<Mutex<Option<u8>>>,
) -> Result<MidiInputConnection<()>> {
    let mut midi_input = MidiInput::new(MIDI_INPUT_CLIENT_NAME)?;
    midi_input.ignore(MESSAGE_TYPE_IGNORE_LIST);

    let connection_result = midi_input.connect(
        input_port,
        MIDI_INPUT_CONNECTION_NAME,
        move |_, message, ()| {
            process_midi_message(
                message,
                &current_channel_arc,
                &midi_message_sender,
                &current_note_arc,
            );
        },
        (),
    )?;

    Ok(connection_result)
}

pub(crate) fn process_midi_message(
    message: &[u8],
    current_channel_arc: &Arc<Mutex<Option<u8>>>,
    midi_message_sender: &Sender<MidiEvent>,
    current_note_arc: &Arc<Mutex<Option<u8>>>,
) {
    if message.is_empty() {
        return;
    }

    if !message_channel_matches_current_channel(message, current_channel_arc) {
        log::trace!(target: "midi::input", "Dropping message for non-matching channel");
        return;
    }

    if let Some(event) = event_from_message_status(message, current_note_arc)
        && let Err(err) = midi_message_sender.send(event)
    {
        log::error!(
            target: "midi::input",
            "Could not send MIDI message to the synthesizer module: {err}"
        );
    }
}

fn message_channel_matches_current_channel(
    message: &[u8],
    current_channel_arc: &Arc<Mutex<Option<u8>>>,
) -> bool {
    let message_channel = channel_from_status_byte(message[MESSAGE_STATUS_BYTE_INDEX]);
    let current_channel = current_channel_arc
        .lock()
        .unwrap_or_else(PoisonError::into_inner);

    if current_channel.is_none() {
        return true;
    }

    if let Some(channel) = *current_channel
        && channel == message_channel
    {
        return true;
    }

    false
}

fn event_from_message_status(
    message: &[u8],
    current_note_arc: &Arc<Mutex<Option<u8>>>,
) -> Option<MidiEvent> {
    match message_status_from_status_byte(message[MESSAGE_STATUS_BYTE_INDEX]) {
        Status::NoteOn => process_note_on_message(message, current_note_arc),
        Status::NoteOff => process_note_off_message(message, current_note_arc),
        Status::ControlChange => process_cc_message(message),
        Status::PitchBend => process_pitch_bend_message(message),
        Status::ProgramChange => process_program_change_message(message),
        Status::ChannelPressure => process_channel_pressure_message(message),
        _ => {
            log::debug!(target: "midi::input", "Unhandled MIDI status type: 0x{:02X}", message[MESSAGE_STATUS_BYTE_INDEX]);
            None
        }
    }
}

fn process_program_change_message(message: &[u8]) -> Option<MidiEvent> {
    let program_number = *message.get(PROGRAM_CHANGE_VALUE_BYTE_INDEX)?;
    Some(MidiEvent::ProgramChange(program_number))
}

fn process_channel_pressure_message(message: &[u8]) -> Option<MidiEvent> {
    let pressure_amount = *message.get(CHANNEL_PRESSURE_VALUE_BYTE_INDEX)?;
    Some(MidiEvent::ChannelPressure(pressure_amount))
}

fn process_pitch_bend_message(message: &[u8]) -> Option<MidiEvent> {
    let amount_most_significant_byte = *message.get(PITCH_BEND_MESSAGE_MSB_BYTE_INDEX)?;
    let amount_least_significant_byte = *message.get(PITCH_BEND_MESSAGE_LSB_BYTE_INDEX)?;
    let pitch_bend_amount =
        u16::from(amount_most_significant_byte) << 7 | u16::from(amount_least_significant_byte);
    Some(MidiEvent::PitchBend(pitch_bend_amount))
}

fn process_cc_message(message: &[u8]) -> Option<MidiEvent> {
    let cc_number = *message.get(CC_MESSAGE_NUMBER_BYTE_INDEX)?;
    let cc_value = *message.get(CC_MESSAGE_VALUE_BYTE_INDEX)?;
    control_change::get_supported_cc_from_cc_number(cc_number, cc_value)
        .map(MidiEvent::ControlChange)
}

fn process_note_off_message(
    message: &[u8],
    current_note_arc: &Arc<Mutex<Option<u8>>>,
) -> Option<MidiEvent> {
    let midi_note = *message.get(NOTE_MESSAGE_NUMBER_BYTE_INDEX)?;
    let mut current_note = current_note_arc
        .lock()
        .unwrap_or_else(PoisonError::into_inner);
    match *current_note {
        Some(note) if note == midi_note => {
            *current_note = None;
            Some(MidiEvent::NoteOff)
        }
        _ => None,
    }
}

fn process_note_on_message(
    message: &[u8],
    current_note_arc: &Arc<Mutex<Option<u8>>>,
) -> Option<MidiEvent> {
    let midi_note = *message.get(NOTE_MESSAGE_NUMBER_BYTE_INDEX)?;
    let midi_velocity = *message.get(NOTE_MESSAGE_VELOCITY_BYTE_INDEX)?;

    let mut current_note = current_note_arc
        .lock()
        .unwrap_or_else(PoisonError::into_inner);

    if midi_velocity > 0 {
        *current_note = Some(midi_note);
        return Some(MidiEvent::NoteOn(midi_note, midi_velocity));
    }

    if matches!(*current_note, Some(note) if note == midi_note) {
        *current_note = None;
        return Some(MidiEvent::NoteOff);
    }

    None
}

fn channel_from_status_byte(status: u8) -> u8 {
    let raw_message_channel = status & MESSAGE_STATUS_BYTE_CHANNEL_MASK;
    raw_message_channel + RAW_CHANNEL_TO_USER_READABLE_CHANNEL_OFFSET
}

fn message_status_from_status_byte(status: u8) -> Status {
    let status_type = status & MESSAGE_STATUS_BYTE_TYPE_MASK;
    match status_type {
        0x80 => Status::NoteOff,
        0x90 => Status::NoteOn,
        0xA0 => Status::PolyphonicKeyPressure,
        0xB0 => Status::ControlChange,
        0xC0 => Status::ProgramChange,
        0xD0 => Status::ChannelPressure,
        0xE0 => Status::PitchBend,
        _ => Status::Unknown,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::{Arc, Mutex};

    #[test]
    fn process_midi_message_does_not_panic_on_empty_message() {
        use crossbeam_channel::unbounded;
        let (tx, _rx) = unbounded::<MidiEvent>();
        let channel = Arc::new(Mutex::new(None::<u8>));
        let note = Arc::new(Mutex::new(None::<u8>));
        // must not panic
        process_midi_message(&[], &channel, &tx, &note);
    }

    #[test]
    fn process_midi_message_does_not_panic_on_single_byte_note_on_status() {
        use crossbeam_channel::unbounded;
        let (tx, _rx) = unbounded::<MidiEvent>();
        let channel = Arc::new(Mutex::new(None::<u8>));
        let note = Arc::new(Mutex::new(None::<u8>));
        // 0x90 = Note On ch 1, but no subsequent bytes
        process_midi_message(&[0x90], &channel, &tx, &note);
    }

    #[test]
    fn process_midi_message_does_not_panic_on_two_byte_note_on_message() {
        use crossbeam_channel::unbounded;
        let (tx, _rx) = unbounded::<MidiEvent>();
        let channel = Arc::new(Mutex::new(None::<u8>));
        let note = Arc::new(Mutex::new(None::<u8>));
        // 0x90 = Note On, note=60, missing velocity byte
        process_midi_message(&[0x90, 60], &channel, &tx, &note);
    }

    #[test]
    fn message_type_from_status_byte_returns_correct_status_for_note_on_0_channel() {
        let status_byte = 0x90;
        let expected_result = Status::NoteOn;
        let result = message_status_from_status_byte(status_byte);
        assert_eq!(result, expected_result);
    }

    #[test]
    fn message_type_from_status_byte_returns_correct_status_for_note_on_with_a_channel() {
        let status_byte = 0x9A;
        let expected_result = Status::NoteOn;
        let result = message_status_from_status_byte(status_byte);
        assert_eq!(result, expected_result);
    }

    #[test]
    fn message_type_from_status_byte_returns_correct_status_for_control_change() {
        let status_byte = 0xB3;
        let expected_result = Status::ControlChange;
        let result = message_status_from_status_byte(status_byte);
        assert_eq!(result, expected_result);
    }

    #[test]
    fn message_type_from_status_byte_returns_correct_status_for_unknown() {
        let status_byte = 0x70; // Unknown/invalid status byte
        let expected_result = Status::Unknown;
        let result = message_status_from_status_byte(status_byte);
        assert_eq!(result, expected_result);
    }

    #[test]
    fn message_type_from_status_byte_correctly_returns_unknown_from_zero_status_byte() {
        let status_byte = 0x00;
        let expected_result = Status::Unknown;
        let result = message_status_from_status_byte(status_byte);
        assert_eq!(result, expected_result);
    }

    #[test]
    fn message_type_from_status_byte_correctly_returns_unknown_from_max_status_byte() {
        let status_byte = 0xFF;
        let expected_result = Status::Unknown;
        let result = message_status_from_status_byte(status_byte);
        assert_eq!(result, expected_result);
    }
}
