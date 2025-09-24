use crate::midi::constants::{
    MIDI_CC_NUMBER_BYTE_INDEX, MIDI_CC_VALUE_BYTE_INDEX, MIDI_CHANNEL_PRESSURE_VALUE_BYTE_INDEX,
    MIDI_INPUT_CLIENT_NAME, MIDI_INPUT_CONNECTION_NAME, MIDI_MESSAGE_IGNORE_LIST,
    MIDI_NOTE_NUMBER_BYTE_INDEX, MIDI_PITCH_BEND_LSB_BYTE_INDEX, MIDI_PITCH_BEND_MSB_BYTE_INDEX,
    MIDI_STATUS_BYTE_INDEX, MIDI_VELOCITY_BYTE_INDEX, PANIC_MESSAGE_MIDI_SENDER_FAILURE,
    RAW_CHANNEL_TO_USER_READABLE_CHANNEL_OFFSET, STATUS_BYTE_CHANNEL_MASK,
    STATUS_BYTE_MESSAGE_TYPE_MASK,
};
use crate::midi::{Event, Status, control_change};
use crossbeam_channel::Sender;
use midir::{MidiInput, MidiInputConnection, MidiInputPort};
use std::sync::{Arc, Mutex, PoisonError};

pub fn create_midi_input_listener(
    input_port_name: &str,
    current_channel_arc: Arc<Mutex<Option<u8>>>,
    midi_message_sender: Sender<Event>,
    current_note_arc: Arc<Mutex<Option<u8>>>,
) -> Option<MidiInputConnection<()>> {
    let mut midi_input = match MidiInput::new(MIDI_INPUT_CLIENT_NAME) {
        Ok(midi_input) => midi_input,
        Err(err) => {
            log::error!(
                "create_midi_input_listener(): Could not create MIDI input. Returning None.  Error: {err}."
            );
            return None;
        }
    };

    midi_input.ignore(MIDI_MESSAGE_IGNORE_LIST);


    let Some(input_port) = input_port_from_port_name(input_port_name, &midi_input) else {
            log::error!(
                "create_midi_input_listener(): Could not find MIDI input port{input_port_name}. Continuing without Midi."
            );
            return None;
        };

    let connection_result = midi_input.connect(
        &input_port,
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
    );

    match connection_result {
        Ok(connection) => {
            log::info!(
                "create_midi_control_listener(): The MIDI input connection listener has been created for {input_port_name}."
            );
            Some(connection)
        }
        Err(err) => {
            log::error!(
                "create_midi_input_listener(): Could not connect to MIDI input port. Returning None.  Error: {err}."
            );
            None
        }
    }
}

fn process_midi_message(
    message: &[u8],
    current_channel_arc: &Arc<Mutex<Option<u8>>>,
    midi_message_sender: &Sender<Event>,
    current_note_arc: &Arc<Mutex<Option<u8>>>,
) {
    if !message_channel_matches_current_channel(message, current_channel_arc) {
        return;
    }

    if let Some(message) = event_from_message_status(message, current_note_arc) {
        midi_message_sender.send(message).unwrap_or_else(|err| {
            log::error!(
                "process_midi_message(): FATAL ERROR: midi message sender failure. Exiting. Error: {err}."
            );
            panic!("{PANIC_MESSAGE_MIDI_SENDER_FAILURE}");
        });
    }
}

fn message_channel_matches_current_channel(
    message: &[u8],
    current_channel_arc: &Arc<Mutex<Option<u8>>>,
) -> bool {
    let message_channel = channel_from_status_byte(message[MIDI_STATUS_BYTE_INDEX]);
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
    match message_status_from_status_byte(message[MIDI_STATUS_BYTE_INDEX]) {
        Status::NoteOn => {
            let midi_note = message[MIDI_NOTE_NUMBER_BYTE_INDEX];
            let midi_velocity = message[MIDI_VELOCITY_BYTE_INDEX];

            let mut current_note = current_note_arc
                .lock()
                .unwrap_or_else(PoisonError::into_inner);

            if midi_velocity == 0 {
                match *current_note {
                    Some(note) if note == midi_note => {
                        *current_note = None;
                        Some(Event::NoteOff)
                    }
                    _ => None,
                }
            } else {
                *current_note = Some(midi_note);
                Some(Event::NoteOn(midi_note, midi_velocity))
            }
        }
        Status::NoteOff => {
            let midi_note = message[MIDI_NOTE_NUMBER_BYTE_INDEX];
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
        Status::ControlChange => {
            let cc_number = message[MIDI_CC_NUMBER_BYTE_INDEX];
            let cc_value = message[MIDI_CC_VALUE_BYTE_INDEX];
            control_change::get_supported_cc_from_cc_number(cc_number, cc_value)
                .map(Event::ControlChange)
        }
        Status::PitchBend => {
            let amount_most_significant_byte = message[MIDI_PITCH_BEND_MSB_BYTE_INDEX];
            let amount_least_significant_byte = message[MIDI_PITCH_BEND_LSB_BYTE_INDEX];
            let pitch_bend_amount = u16::from(amount_most_significant_byte) << 7
                | u16::from(amount_least_significant_byte);
            Some(Event::PitchBend(pitch_bend_amount))
        }
        Status::ChannelPressure => {
            let pressure_amount = message[MIDI_CHANNEL_PRESSURE_VALUE_BYTE_INDEX];
            Some(Event::ChannelPressure(pressure_amount))
        }
        _ => None,
    }
}

fn channel_from_status_byte(status: u8) -> u8 {
    let raw_message_channel = status & STATUS_BYTE_CHANNEL_MASK;
    raw_message_channel + RAW_CHANNEL_TO_USER_READABLE_CHANNEL_OFFSET
}

fn message_status_from_status_byte(status: u8) -> Status {
    let status_type = status & STATUS_BYTE_MESSAGE_TYPE_MASK;
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

pub fn input_port_from_port_name(
    input_port_name: &str,
    midi_input: &MidiInput,
) -> Option<MidiInputPort> {

    midi_input.ports().into_iter().find(|port| midi_input.port_name(port).unwrap_or_default() == input_port_name)

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
