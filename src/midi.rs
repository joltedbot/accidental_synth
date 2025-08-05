use anyhow::{Result, anyhow};
use crossbeam_channel::{Receiver, Sender};
use midir::{MidiInput, MidiInputConnection, MidiInputPort};

const MIDI_INPUT_CLIENT_NAME: &str = "Accidental Synth MIDI Input";
const DEFAULT_MIDI_PORT_INDEX: usize = 0;
const MIDI_INPUT_CONNECTION_NAME: &str = "Accidental Synth MIDI Input Connection";
const MIDI_STATUS_BYTE_INDEX: usize = 0;
const MIDI_NOTE_NUMBER_BYTE_INDEX: usize = 1;
const MIDI_VELOCITY_BYTE_INDEX: usize = 2;
const MIDI_CC_NUMBER_BYTE_INDEX: usize = 1;
const MIDI_CC_VALUE_BYTE_INDEX: usize = 2;
const MIDI_MESSAGE_CHANNEL_CAPACITY: usize = 16;

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
    NoteOff(u8, u8),
    ControlChange(u8, u8),
}

pub struct Midi {
    input_listener: Option<MidiInputConnection<()>>,
    midi_message_sender: Sender<MidiMessage>,
    midi_message_receiver: Receiver<MidiMessage>,
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
        }
    }

    pub fn get_midi_message_receiver(&self) -> Receiver<MidiMessage> {
        self.midi_message_receiver.clone()
    }

    pub fn run(&mut self) -> Result<()> {
        log::info!("Creating MIDI input connection listener.");

        let default_midi_input_port = get_default_midi_device()?;

        match default_midi_input_port {
            Some(midi_input_port) => {
                self.input_listener = Some(create_midi_input_listener(
                    midi_input_port,
                    self.midi_message_sender.clone(),
                )?);
                log::debug!("run(): The MIDI input connection has been created.");
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
) -> Result<MidiInputConnection<()>> {
    let midi_in = MidiInput::new(MIDI_INPUT_CLIENT_NAME)?;
    let midi_input_connection = midi_in.connect(
        &midi_input_port,
        MIDI_INPUT_CONNECTION_NAME,
        move |_, message, _| {
            let update_message_to_send =
                match message_type_from_status_byte(&message[MIDI_STATUS_BYTE_INDEX]) {
                    MessageType::NoteOn => {
                        let note = message[MIDI_NOTE_NUMBER_BYTE_INDEX];
                        let velocity = message[MIDI_VELOCITY_BYTE_INDEX];

                        if velocity == 0 {
                            Some(MidiMessage::NoteOff(note, velocity))
                        } else {
                            Some(MidiMessage::NoteOn(note, velocity))
                        }
                    }
                    MessageType::NoteOff => {
                        let note = message[MIDI_NOTE_NUMBER_BYTE_INDEX];
                        let velocity = message[MIDI_VELOCITY_BYTE_INDEX];
                        Some(MidiMessage::NoteOff(note, velocity))
                    }
                    MessageType::ControlChange => {
                        let number = message[MIDI_CC_NUMBER_BYTE_INDEX];
                        let value = message[MIDI_CC_VALUE_BYTE_INDEX];
                        Some(MidiMessage::ControlChange(number, value))
                    }
                    _ => None,
                };

            if let Some(message) = update_message_to_send {
                if let Err(err) = midi_message_sender.send(message) {
                    log::error!(
                        "create_midi_input_listener(): Could not send MIDI messge over the channel."
                    );
                }
            }
        },
        (),
    )?;

    Ok(midi_input_connection)
}

fn get_default_midi_device() -> Result<Option<MidiInputPort>> {
    let midi_input = match MidiInput::new(MIDI_INPUT_CLIENT_NAME) {
        Ok(port) => port,
        Err(err) => {
            log::error!("get_default_midi_device(): Could not create a new MIDI input object.");
            return Err(anyhow!(err));
        }
    };

    Ok(midi_input.ports().get(DEFAULT_MIDI_PORT_INDEX).cloned())
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
