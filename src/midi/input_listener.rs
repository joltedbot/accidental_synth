use crate::midi::constants::{
    CC_MESSAGE_NUMBER_BYTE_INDEX, CC_MESSAGE_VALUE_BYTE_INDEX, CHANNEL_PRESSURE_VALUE_BYTE_INDEX,
    MESSAGE_STATUS_BYTE_CHANNEL_MASK, MESSAGE_STATUS_BYTE_INDEX, MESSAGE_STATUS_BYTE_TYPE_MASK,
    MESSAGE_TYPE_IGNORE_LIST, MIDI_INPUT_CLIENT_NAME, MIDI_INPUT_CONNECTION_NAME,
    NOTE_MESSAGE_NUMBER_BYTE_INDEX, NOTE_MESSAGE_VELOCITY_BYTE_INDEX,
    PITCH_BEND_MESSAGE_LSB_BYTE_INDEX, PITCH_BEND_MESSAGE_MSB_BYTE_INDEX,
    RAW_CHANNEL_TO_USER_READABLE_CHANNEL_OFFSET,
};
use crate::midi::{Event, MidiError, Status, control_change};
use anyhow::Result;
use crossbeam_channel::Sender;
use midir::{MidiInput, MidiInputConnection, MidiInputPort};
use std::sync::{Arc, Mutex, PoisonError};

pub fn create_midi_input_listener(
    input_port: &MidiInputPort,
    current_channel_arc: Arc<Mutex<Option<u8>>>,
    midi_message_sender: Sender<Event>,
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
    midi_message_sender: &Sender<Event>,
    current_note_arc: &Arc<Mutex<Option<u8>>>,
) {
    if !message_channel_matches_current_channel(message, current_channel_arc) {
        return;
    }

    if let Some(event) = event_from_message_status(message, current_note_arc) {
        midi_message_sender.send(event).unwrap_or_else(|err| {
            let midi_err = MidiError::MessageSendFailed;
            log::error!(
                target: "midi::message",
                error:% = midi_err,
                details:% = err;
                "Could not send MIDI message to the synthesizer module."
            );
            panic!("{midi_err}");
        });
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
) -> Option<Event> {
    match message_status_from_status_byte(message[MESSAGE_STATUS_BYTE_INDEX]) {
        Status::NoteOn => process_note_on_message(message, current_note_arc),
        Status::NoteOff => process_note_off_message(message, current_note_arc),
        Status::ControlChange => process_cc_message(message),
        Status::PitchBend => Some(process_pitch_bend_message(message)),
        Status::ChannelPressure => Some(process_channel_pressure_message(message)),
        _ => None,
    }
}

fn process_channel_pressure_message(message: &[u8]) -> Event {
    let pressure_amount = message[CHANNEL_PRESSURE_VALUE_BYTE_INDEX];
    Event::ChannelPressure(pressure_amount)
}

fn process_pitch_bend_message(message: &[u8]) -> Event {
    let amount_most_significant_byte = message[PITCH_BEND_MESSAGE_MSB_BYTE_INDEX];
    let amount_least_significant_byte = message[PITCH_BEND_MESSAGE_LSB_BYTE_INDEX];
    let pitch_bend_amount =
        u16::from(amount_most_significant_byte) << 7 | u16::from(amount_least_significant_byte);
    Event::PitchBend(pitch_bend_amount)
}

fn process_cc_message(message: &[u8]) -> Option<Event> {
    let cc_number = message[CC_MESSAGE_NUMBER_BYTE_INDEX];
    let cc_value = message[CC_MESSAGE_VALUE_BYTE_INDEX];
    control_change::get_supported_cc_from_cc_number(cc_number, cc_value).map(Event::ControlChange)
}

fn process_note_off_message(
    message: &[u8],
    current_note_arc: &Arc<Mutex<Option<u8>>>,
) -> Option<Event> {
    let midi_note = message[NOTE_MESSAGE_NUMBER_BYTE_INDEX];
    let mut current_note = current_note_arc
        .lock()
        .unwrap_or_else(PoisonError::into_inner);
    match *current_note {
        Some(note) if note == midi_note => {
            *current_note = None;
            Some(Event::NoteOff)
        }
        _ => None,
    }
}

fn process_note_on_message(
    message: &[u8],
    current_note_arc: &Arc<Mutex<Option<u8>>>,
) -> Option<Event> {
    let midi_note = message[NOTE_MESSAGE_NUMBER_BYTE_INDEX];
    let midi_velocity = message[NOTE_MESSAGE_VELOCITY_BYTE_INDEX];

    let mut current_note = current_note_arc
        .lock()
        .unwrap_or_else(PoisonError::into_inner);

    if midi_velocity > 0 {
        *current_note = Some(midi_note);
        return Some(Event::NoteOn(midi_note, midi_velocity));
    }

    if matches!(*current_note, Some(note) if note == midi_note) {
        *current_note = None;
        return Some(Event::NoteOff);
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
