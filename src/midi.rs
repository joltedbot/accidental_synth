use anyhow::{Result, anyhow};
use crossbeam_channel::{Receiver, Sender};
use midir::{MidiInput, MidiInputConnection, MidiInputPort};
use std::sync::{Arc, Mutex};

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
const MIDI_MESSAGE_CHANNEL_CAPACITY: usize = 16;

const DEFAULT_NAME_FOR_UNNAMED_MIDI_PORTS: &str = "Name Not Available";

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

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum MidiMessage {
    NoteOn(u8, u8),
    NoteOff,
    ControlChange(u8, u8),
}

pub struct Midi {
    input_listener: Option<MidiInputConnection<()>>,
    midi_message_sender: Sender<MidiMessage>,
    midi_message_receiver: Receiver<MidiMessage>,
    current_note: Arc<Mutex<Option<u8>>>,
}

impl Midi {
    pub fn new() -> Self {
        log::info!("Constructing Midi Module");

        let (midi_message_sender, midi_message_receiver) =
            crossbeam_channel::bounded(MIDI_MESSAGE_CHANNEL_CAPACITY);

        Self {
            input_listener: None,
            midi_message_sender,
            midi_message_receiver,
            current_note: Default::default(),
        }
    }

    pub fn get_midi_message_receiver(&self) -> Receiver<MidiMessage> {
        self.midi_message_receiver.clone()
    }

    pub fn run(&mut self) -> Result<()> {
        log::info!("Creating MIDI input connection listener.");

        let default_midi_input_port = get_default_midi_device()?;

        match default_midi_input_port {
            Some((name, port)) => {
                self.input_listener = Some(create_midi_input_listener(
                    port,
                    self.midi_message_sender.clone(),
                    self.current_note.clone(),
                )?);
                log::debug!(
                    "create_midi_input_listener(): The MIDI input connection has been created for {name}."
                );
            }
            None => {
                log::warn!(
                    "run(): Could not find a default MIDI input port. Continuing without MIDI input."
                );
            }
        }

        Ok(())
    }
}

fn create_midi_input_listener(
    midi_input_port: MidiInputPort,
    midi_message_sender: Sender<MidiMessage>,
    current_note_arc: Arc<Mutex<Option<u8>>>,
) -> Result<MidiInputConnection<()>> {
    let midi_input = MidiInput::new(MIDI_INPUT_CLIENT_NAME)?;

    midi_input.connect(
        &midi_input_port,
        MIDI_INPUT_CONNECTION_NAME,
        move |_, message, _| {
            let update_message_to_send =
                match message_type_from_status_byte(&message[MIDI_STATUS_BYTE_INDEX]) {
                    MessageType::NoteOn => {
                        let midi_note = message[MIDI_NOTE_NUMBER_BYTE_INDEX];
                        let midi_velocity = message[MIDI_VELOCITY_BYTE_INDEX];

                        let mut current_note = current_note_arc.lock().unwrap_or_else(|poisoned| poisoned.into_inner());
                        *current_note = Some(midi_note);

                        if midi_velocity == 0 {
                            Some(MidiMessage::NoteOff)
                        } else {
                            Some(MidiMessage::NoteOn(midi_note, midi_velocity))
                        }
                    }
                    MessageType::NoteOff => {
                        let midi_note = message[MIDI_NOTE_NUMBER_BYTE_INDEX];

                        let mut current_note = current_note_arc.lock().unwrap_or_else(|poisoned| poisoned.into_inner());
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
                        Some(MidiMessage::ControlChange(cc_number, cc_value))
                    }
                    _ => None,
                };

            if let Some(message) = update_message_to_send {
                midi_message_sender.send(message).unwrap_or_else(|err| {
                    log::error!("create_midi_input_listener(): FATAL ERROR: midi message channel send failure. \
                    Exiting. Error: {err}.");
                    panic!("{PANIC_MESSAGE_MIDI_SENDER_FAILURE}");
                });
            }
        },
        (),
    ).map_err(|err| anyhow!(err))
}

fn get_default_midi_device() -> Result<Option<(String, MidiInputPort)>> {
    let midi_input = match MidiInput::new(MIDI_INPUT_CLIENT_NAME) {
        Ok(port) => port,
        Err(err) => {
            log::error!("get_default_midi_device(): Could not create a new MIDI input object.");
            return Err(anyhow!(err));
        }
    };

    match midi_input.ports().get(DEFAULT_MIDI_PORT_INDEX).cloned() {
        Some(port) => {
            let name = midi_input
                .port_name(&port)
                .unwrap_or(DEFAULT_NAME_FOR_UNNAMED_MIDI_PORTS.to_string());
            Ok(Some((name, port)))
        }
        None => Ok(None),
    }
}

fn message_type_from_status_byte(status: &u8) -> MessageType {
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
    use crossbeam_channel::internal::SelectHandle;

    #[test]
    fn new_returns_midi_with_correct_default_values() {
        let midi = Midi::new();

        assert!(midi.input_listener.is_none());
        assert!(midi.midi_message_sender.is_ready());
        assert_eq!(
            midi.midi_message_receiver.capacity(),
            Some(MIDI_MESSAGE_CHANNEL_CAPACITY)
        );
        assert!(
            midi.current_note
                .lock()
                .unwrap_or_else(|poisoned| poisoned.into_inner())
                .is_none()
        );
    }

    #[test]
    fn message_type_from_status_byte_returns_correct_status_for_note_on_0_channel() {
        let status_byte = 0x90;
        let expected_result = MessageType::NoteOn;
        let result = message_type_from_status_byte(&status_byte);
        assert_eq!(result, expected_result);
    }

    #[test]
    fn message_type_from_status_byte_returns_correct_status_for_note_on_with_a_channel() {
        let status_byte = 0x9A;
        let expected_result = MessageType::NoteOn;
        let result = message_type_from_status_byte(&status_byte);
        assert_eq!(result, expected_result);
    }

    #[test]
    fn message_type_from_status_byte_returns_correct_status_for_control_change() {
        let status_byte = 0xB3;
        let expected_result = MessageType::ControlChange;
        let result = message_type_from_status_byte(&status_byte);
        assert_eq!(result, expected_result);
    }

    #[test]
    fn message_type_from_status_byte_returns_correct_status_for_unknown() {
        let status_byte = 0x70; // Unknown/invalid status byte
        let expected_result = MessageType::Unknown;
        let result = message_type_from_status_byte(&status_byte);
        assert_eq!(result, expected_result);
    }

    #[test]
    fn message_type_from_status_byte_correctly_returns_unknown_from_zero_status_byte() {
        let status_byte = 0x00;
        let expected_result = MessageType::Unknown;
        let result = message_type_from_status_byte(&status_byte);
        assert_eq!(result, expected_result);
    }

    #[test]
    fn message_type_from_status_byte_correctly_returns_unknown_from_max_status_byte() {
        let status_byte = 0xFF;
        let expected_result = MessageType::Unknown;
        let result = message_type_from_status_byte(&status_byte);
        assert_eq!(result, expected_result);
    }
}
