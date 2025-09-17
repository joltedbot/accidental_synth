pub mod control_change;

use anyhow::{Result, anyhow};
use control_change::CC;
use crossbeam_channel::{Receiver, Sender};
use midir::{Ignore, MidiInput, MidiInputConnection, MidiInputPort};
use std::collections::HashMap;
use std::sync::{Arc, Mutex, PoisonError};

const PANIC_MESSAGE_MIDI_SENDER_FAILURE: &str =
    "Could not send MIDI message to the synthesizer engine.";
const MIDI_INPUT_CLIENT_NAME: &str = "Accidental Synth MIDI Input";
const DEFAULT_MIDI_PORT_INDEX: usize = 0;
const MIDI_INPUT_CONNECTION_NAME: &str = "Accidental Synth MIDI Input Connection";
const MIDI_STATUS_BYTE_INDEX: usize = 0;
const MIDI_NOTE_NUMBER_BYTE_INDEX: usize = 1;
const MIDI_VELOCITY_BYTE_INDEX: usize = 2;
const MIDI_CC_NUMBER_BYTE_INDEX: usize = 1;
const MIDI_CC_VALUE_BYTE_INDEX: usize = 2;
const MIDI_CHANNEL_PRESSURE_VALUE_BYTE_INDEX: usize = 1;
const MIDI_PITCH_BEND_MSB_BYTE_INDEX: usize = 2;
const MIDI_PITCH_BEND_LSB_BYTE_INDEX: usize = 1;

const MIDI_MESSAGE_CHANNEL_CAPACITY: usize = 16;

const DEFAULT_NAME_FOR_UNNAMED_MIDI_PORTS: &str = "Name Not Available";
const MIDI_MESSAGE_IGNORE_LIST: Ignore = Ignore::SysexAndTime;

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
    input_listener: Option<MidiInputConnection<()>>,
    message_sender: Sender<MidiMessage>,
    message_receiver: Receiver<MidiMessage>,
    current_note: Arc<Mutex<Option<u8>>>,
    input_ports: HashMap<String, String>,
    current_input: Option<InputPort>,
}

impl Midi {
    pub fn new() -> Self {
        log::info!("Constructing Midi Module");

        let (midi_message_sender, midi_message_receiver) =
            crossbeam_channel::bounded(MIDI_MESSAGE_CHANNEL_CAPACITY);

        Self {
            input_listener: None,
            message_sender: midi_message_sender,
            message_receiver: midi_message_receiver,
            current_note: Arc::default(),
            input_ports: HashMap::new(),
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

        self.current_input = default_midi_device()?;

        if let Some(input) = &self.current_input {
            self.input_listener = Some(create_midi_input_listener(
                    &input.port,
                    self.message_sender.clone(),
                    self.current_note.clone(),
                )?);

                log::info!(
                    "create_midi_input_listener(): The MIDI input connection listener has been created for {}.",
                    input.name
                );
        };

        Ok(())
    }
}

fn create_midi_input_listener(
    midi_input_port: &MidiInputPort,
    midi_message_sender: Sender<MidiMessage>,
    current_note_arc: Arc<Mutex<Option<u8>>>,
) -> Result<MidiInputConnection<()>> {
    let midi_input = new_midi_input()?;

    midi_input.connect(
        midi_input_port,
        MIDI_INPUT_CONNECTION_NAME,
        move |_, message, ()| {
            process_midi_message(message, &midi_message_sender, &current_note_arc);
        },
            (),
    ).map_err(|err| anyhow!(err))
}

fn process_midi_message(message: &[u8], midi_message_sender: &Sender<MidiMessage>, current_note_arc: &Arc<Mutex<Option<u8>>>) {

    let message_type = midi_message_type_from_message_byte(message, current_note_arc);

    if let Some(message) = message_type {
        midi_message_sender.send(message).unwrap_or_else(|err| {
            log::error!("create_midi_input_listener(): FATAL ERROR: midi message channel send failure. \
                Exiting. Error: {err}.");
            panic!("{PANIC_MESSAGE_MIDI_SENDER_FAILURE}");
        });
    }
}

fn midi_message_type_from_message_byte(message: &[u8], current_note_arc: &Arc<Mutex<Option<u8>>>) -> Option<MidiMessage> {

        match message_type_from_status_byte(message[MIDI_STATUS_BYTE_INDEX]) {
            MessageType::NoteOn => {
                let midi_note = message[MIDI_NOTE_NUMBER_BYTE_INDEX];
                let midi_velocity = message[MIDI_VELOCITY_BYTE_INDEX];

                let mut current_note = current_note_arc.lock().unwrap_or_else(PoisonError::into_inner);
                *current_note = Some(midi_note);

                if midi_velocity == 0 {
                    Some(MidiMessage::NoteOff)
                } else {
                    Some(MidiMessage::NoteOn(midi_note, midi_velocity))
                }
            }
            MessageType::NoteOff => {
                let midi_note = message[MIDI_NOTE_NUMBER_BYTE_INDEX];
                let mut current_note = current_note_arc.lock().unwrap_or_else(PoisonError::into_inner);
                match *current_note {
                    Some(note) if note == midi_note => {
                        *current_note = None;
                        Some(MidiMessage::NoteOff)
                    },
                    _ => None,
                }
            }
            MessageType::ControlChange => {
                let cc_number = message[MIDI_CC_NUMBER_BYTE_INDEX];
                let cc_value = message[MIDI_CC_VALUE_BYTE_INDEX];
                control_change::get_supported_cc_from_cc_number(cc_number, cc_value).map(MidiMessage::ControlChange)
            }
            MessageType::PitchBend => {
                let amount_most_significant_byte = message[MIDI_PITCH_BEND_MSB_BYTE_INDEX];
                let amount_least_significant_byte = message[MIDI_PITCH_BEND_LSB_BYTE_INDEX];
                let pitch_bend_amount = u16::from(amount_most_significant_byte) << 7 | u16::from(amount_least_significant_byte);
                Some(MidiMessage::PitchBend(pitch_bend_amount))
            }
            MessageType::ChannelPressure => {
                let pressure_amount = message[MIDI_CHANNEL_PRESSURE_VALUE_BYTE_INDEX];
                Some(MidiMessage::ChannelPressure(pressure_amount))
            }
            _ => None,
        }
}

fn default_midi_device() -> Result<Option<InputPort>> {
    let mut midi_input = new_midi_input()?;

    midi_input.ignore(MIDI_MESSAGE_IGNORE_LIST);

    match midi_input.ports().get(DEFAULT_MIDI_PORT_INDEX).cloned() {
        Some(port) => {
            let id = port.id();
            let name = midi_input
                .port_name(&port)
                .unwrap_or(DEFAULT_NAME_FOR_UNNAMED_MIDI_PORTS.to_string());
            Ok(Some(InputPort { id, name, port }))
        }
        None => {
            log::warn!("default_midi_device(): Could not find a default MIDI input port. Continuing without MIDI input.");
            Ok(None)
        }
    }
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

    port_list.iter().for_each(|port| {
        let name = midi_input
            .port_name(&port)
            .unwrap_or(DEFAULT_NAME_FOR_UNNAMED_MIDI_PORTS.to_string());
        let id = port.id();
        midi_input_ports.insert(id.to_string(), name);
    });

    Ok(midi_input_ports)
}



fn message_type_from_status_byte(status: u8) -> MessageType {
    let status_type = status & 0xF0;
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
            Some(MIDI_MESSAGE_CHANNEL_CAPACITY)
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
