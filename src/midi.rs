mod constants;
pub mod control_change;

use anyhow::{Result, anyhow};
use constants::{
    DEFAULT_MIDI_PORT_INDEX, DEFAULT_NAME_FOR_UNNAMED_MIDI_PORTS, MIDI_CC_NUMBER_BYTE_INDEX,
    MIDI_CC_VALUE_BYTE_INDEX, MIDI_CHANNEL_PRESSURE_VALUE_BYTE_INDEX, MIDI_INPUT_CLIENT_NAME,
    MIDI_INPUT_CONNECTION_NAME, MIDI_MESSAGE_IGNORE_LIST, MIDI_MESSAGE_SENDER_CAPACITY,
    MIDI_NOTE_NUMBER_BYTE_INDEX, MIDI_PITCH_BEND_LSB_BYTE_INDEX, MIDI_PITCH_BEND_MSB_BYTE_INDEX,
    MIDI_STATUS_BYTE_INDEX, MIDI_VELOCITY_BYTE_INDEX, PANIC_MESSAGE_MIDI_SENDER_FAILURE,
    RAW_CHANNEL_TO_USER_READABLE_CHANNEL_OFFSET, STATUS_BYTE_CHANNEL_MASK,
    STATUS_BYTE_MESSAGE_TYPE_MASK,
};
use control_change::CC;
use crossbeam_channel::{Receiver, Sender};
use midir::{MidiInput, MidiInputConnection, MidiInputPort};
use std::collections::HashMap;
use std::sync::{Arc, Mutex, PoisonError};

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
enum MessageType {
    NoteOff,
    NoteOn,
    PolyphonicKeyPressure,
    ControlChange,
    ProgramChange,
    ChannelPressure,
    PitchBend,
    Unknown,
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum MidiMessage {
    NoteOn(u8, u8),
    NoteOff,
    ControlChange(CC),
    PitchBend(u16),
    ChannelPressure(u8),
}
#[derive(PartialEq, Clone)]
pub struct InputPort {
    id: String,
    name: String,
    port: MidiInputPort,
}

pub struct Midi {
    message_sender: Sender<MidiMessage>,
    message_receiver: Receiver<MidiMessage>,
    input_listener: Option<MidiInputConnection<()>>,
    input_ports: HashMap<String, String>,
    current_note: Arc<Mutex<Option<u8>>>,
    current_channel: Arc<Mutex<Option<u8>>>,
    current_input: Option<InputPort>,
}

impl Midi {
    pub fn new() -> Self {
        log::info!("Constructing Midi Module");

        let (midi_message_sender, midi_message_receiver) =
            crossbeam_channel::bounded(MIDI_MESSAGE_SENDER_CAPACITY);

        Self {
            message_sender: midi_message_sender,
            message_receiver: midi_message_receiver,
            input_listener: None,
            input_ports: HashMap::new(),
            current_note: Arc::default(),
            current_channel: Arc::new(Mutex::new(Some(7))),
            current_input: None,
        }
    }

    pub fn get_midi_message_receiver(&self) -> Receiver<MidiMessage> {
        self.message_receiver.clone()
    }

    pub fn run(&mut self) -> Result<()> {
        log::info!("Creating MIDI input connection listener.");

        self.input_ports = midi_input_ports()?;
        log::debug!("run(): Found MIDI input ports: {:?}", self.input_ports);

        let input_port = midi_port_from_id(None)?;
        self.current_input = input_device_from_port(input_port)?;

        if let Some(input) = &self.current_input {
            self.input_listener = Some(gcreate_midi_input_listener(
                input,
                self.current_channel.clone(),
                self.message_sender.clone(),
                self.current_note.clone(),
            )?);

            log::info!(
                "run(): The MIDI input connection listener has been created for {}.",
                input.name
            );
        }

        Ok(())
    }

}

fn create_midi_input_listener(
    input_port: &InputPort,
    current_channel_arc: Arc<Mutex<Option<u8>>>,
    midi_message_sender: Sender<MidiMessage>,
    current_note_arc: Arc<Mutex<Option<u8>>>,
) -> Result<MidiInputConnection<()>> {
    let midi_input = new_midi_input()?;

    midi_input
        .connect(
            &input_port.port,
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
        )
        .map_err(|err| anyhow!(err))
}

fn process_midi_message(
    message: &[u8],
    current_channel_arc: &Arc<Mutex<Option<u8>>>,
    midi_message_sender: &Sender<MidiMessage>,
    current_note_arc: &Arc<Mutex<Option<u8>>>,
) {
    let message_channel = channel_from_status_byte(message[MIDI_STATUS_BYTE_INDEX]);

    let current_channel = current_channel_arc.lock().unwrap_or_else(PoisonError::into_inner);
    if matches!(*current_channel, Some(channel) if channel != message_channel) {
        return;
    }
    drop(current_channel);

    if let Some(message) = message_type_from_message_byte(message, current_note_arc) {
        midi_message_sender.send(message).unwrap_or_else(|err| {
            log::error!(
                "process_midi_message(): FATAL ERROR: midi message sender failure. Exiting. Error: {err}."
            );
            panic!("{PANIC_MESSAGE_MIDI_SENDER_FAILURE}");
        });
    }
}

fn message_type_from_message_byte(
    message: &[u8],
    current_note_arc: &Arc<Mutex<Option<u8>>>,
) -> Option<MidiMessage> {
    match message_type_from_status_byte(message[MIDI_STATUS_BYTE_INDEX]) {
        MessageType::NoteOn => {
            let midi_note = message[MIDI_NOTE_NUMBER_BYTE_INDEX];
            let midi_velocity = message[MIDI_VELOCITY_BYTE_INDEX];

            let mut current_note = current_note_arc
                .lock()
                .unwrap_or_else(PoisonError::into_inner);
            *current_note = Some(midi_note);

            if midi_velocity == 0 {
                Some(MidiMessage::NoteOff)
            } else {
                Some(MidiMessage::NoteOn(midi_note, midi_velocity))
            }
        }
        MessageType::NoteOff => {
            let midi_note = message[MIDI_NOTE_NUMBER_BYTE_INDEX];
            let mut current_note = current_note_arc
                .lock()
                .unwrap_or_else(PoisonError::into_inner);
            match *current_note {
                Some(note) if note == midi_note => {
                    *current_note = None;
                    Some(MidiMessage::NoteOff)
                }
                _ => None,
            }
        }
        MessageType::ControlChange => {
            let cc_number = message[MIDI_CC_NUMBER_BYTE_INDEX];
            let cc_value = message[MIDI_CC_VALUE_BYTE_INDEX];
            control_change::get_supported_cc_from_cc_number(cc_number, cc_value)
                .map(MidiMessage::ControlChange)
        }
        MessageType::PitchBend => {
            let amount_most_significant_byte = message[MIDI_PITCH_BEND_MSB_BYTE_INDEX];
            let amount_least_significant_byte = message[MIDI_PITCH_BEND_LSB_BYTE_INDEX];
            let pitch_bend_amount = u16::from(amount_most_significant_byte) << 7
                | u16::from(amount_least_significant_byte);
            Some(MidiMessage::PitchBend(pitch_bend_amount))
        }
        MessageType::ChannelPressure => {
            let pressure_amount = message[MIDI_CHANNEL_PRESSURE_VALUE_BYTE_INDEX];
            Some(MidiMessage::ChannelPressure(pressure_amount))
        }
        _ => None,
    }
}

fn input_device_from_port(input_port: Option<MidiInputPort>) -> Result<Option<InputPort>> {
    let midi_input = new_midi_input()?;

    if let Some(port) = input_port {
        let id = port.id();
        let name = midi_input
            .port_name(&port)
            .unwrap_or(DEFAULT_NAME_FOR_UNNAMED_MIDI_PORTS.to_string());

        log::info!("input_device_from_port(): Using MIDI input port {name}");

        Ok(Some(InputPort { id, name, port }))
    } else {
        log::warn!(
            "input_device_from_port(): Could not find a default MIDI input port. Continuing without MIDI input."
        );
        Ok(None)
    }
}

fn midi_port_from_id(id: Option<String>) -> Result<Option<MidiInputPort>> {
    let mut midi_input = new_midi_input()?;
    midi_input.ignore(MIDI_MESSAGE_IGNORE_LIST);

    let input_port = match id {
        None => {
            log::info!(
                "midi_port_from_id(): No id provided, falling back to default MIDI input port.",
            );
            midi_input.ports().get(DEFAULT_MIDI_PORT_INDEX).cloned()
        }
        Some(id) => midi_input.find_port_by_id(id),
    };

    Ok(input_port)
}

fn new_midi_input() -> Result<MidiInput> {
    match MidiInput::new(MIDI_INPUT_CLIENT_NAME) {
        Ok(input) => Ok(input),
        Err(err) => {
            log::error!("new_midi_input(): Could not create a new MIDI input object.");
            Err(anyhow!(err))
        }
    }
}

fn midi_input_ports() -> Result<HashMap<String, String>> {
    let midi_input = new_midi_input()?;
    let port_list = midi_input.ports();
    let mut midi_input_ports: HashMap<String, String> = HashMap::new();

    for port in &port_list {
        let name = midi_input
            .port_name(port)
            .unwrap_or(DEFAULT_NAME_FOR_UNNAMED_MIDI_PORTS.to_string());
        let id = port.id();
        midi_input_ports.insert(id.to_string(), name);
    }

    Ok(midi_input_ports)
}

fn channel_from_status_byte(status: u8) -> u8 {
    let raw_message_channel = status & STATUS_BYTE_CHANNEL_MASK;
    raw_message_channel + RAW_CHANNEL_TO_USER_READABLE_CHANNEL_OFFSET
}

fn message_type_from_status_byte(status: u8) -> MessageType {
    let status_type = status & STATUS_BYTE_MESSAGE_TYPE_MASK;
    match status_type {
        0x80 => MessageType::NoteOff,
        0x90 => MessageType::NoteOn,
        0xA0 => MessageType::PolyphonicKeyPressure,
        0xB0 => MessageType::ControlChange,
        0xC0 => MessageType::ProgramChange,
        0xD0 => MessageType::ChannelPressure,
        0xE0 => MessageType::PitchBend,
        _ => MessageType::Unknown,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::midi::control_change::get_supported_cc_from_cc_number;
    use crossbeam_channel::internal::SelectHandle;

    #[test]
    fn new_returns_midi_with_correct_default_values() {
        let midi = Midi::new();

        assert!(midi.input_listener.is_none());
        assert!(midi.message_sender.is_ready());
        assert_eq!(
            midi.message_receiver.capacity(),
            Some(MIDI_MESSAGE_SENDER_CAPACITY)
        );
        assert!(
            midi.current_note
                .lock()
                .unwrap_or_else(PoisonError::into_inner)
                .is_none()
        );
    }

    #[test]
    fn message_type_from_status_byte_returns_correct_status_for_note_on_0_channel() {
        let status_byte = 0x90;
        let expected_result = MessageType::NoteOn;
        let result = message_type_from_status_byte(status_byte);
        assert_eq!(result, expected_result);
    }

    #[test]
    fn message_type_from_status_byte_returns_correct_status_for_note_on_with_a_channel() {
        let status_byte = 0x9A;
        let expected_result = MessageType::NoteOn;
        let result = message_type_from_status_byte(status_byte);
        assert_eq!(result, expected_result);
    }

    #[test]
    fn message_type_from_status_byte_returns_correct_status_for_control_change() {
        let status_byte = 0xB3;
        let expected_result = MessageType::ControlChange;
        let result = message_type_from_status_byte(status_byte);
        assert_eq!(result, expected_result);
    }

    #[test]
    fn message_type_from_status_byte_returns_correct_status_for_unknown() {
        let status_byte = 0x70; // Unknown/invalid status byte
        let expected_result = MessageType::Unknown;
        let result = message_type_from_status_byte(status_byte);
        assert_eq!(result, expected_result);
    }

    #[test]
    fn message_type_from_status_byte_correctly_returns_unknown_from_zero_status_byte() {
        let status_byte = 0x00;
        let expected_result = MessageType::Unknown;
        let result = message_type_from_status_byte(status_byte);
        assert_eq!(result, expected_result);
    }

    #[test]
    fn message_type_from_status_byte_correctly_returns_unknown_from_max_status_byte() {
        let status_byte = 0xFF;
        let expected_result = MessageType::Unknown;
        let result = message_type_from_status_byte(status_byte);
        assert_eq!(result, expected_result);
    }

    #[test]
    fn get_supported_cc_returns_some_for_known_ccs() {
        assert_eq!(
            get_supported_cc_from_cc_number(1, 64),
            Some(CC::ModWheel(64))
        );
        assert_eq!(
            get_supported_cc_from_cc_number(74, 100),
            Some(CC::FilterCutoff(100))
        );
        assert_eq!(get_supported_cc_from_cc_number(107, 0), Some(CC::LFO1Reset));
        assert_eq!(
            get_supported_cc_from_cc_number(123, 0),
            Some(CC::AllNotesOff)
        );
    }

    #[test]
    fn get_supported_cc_returns_none_for_out_of_range_cc_number() {
        assert_eq!(get_supported_cc_from_cc_number(200, 127), None);
    }
}
