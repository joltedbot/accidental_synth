use anyhow::{Result, anyhow};
use crossbeam_channel::{Receiver, Sender};
use midir::{Ignore, MidiInput, MidiInputConnection, MidiInputPort};
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
const MIDI_CHANNEL_PRESSURE_VALUE_BYTE_INDEX: usize = 2;
const MIDI_PITCH_BEND_MSB_BYTE_INDEX: usize = 2;
const MIDI_PITCH_BEND_LSB_BYTE_INDEX: usize = 1;

const MIDI_MESSAGE_CHANNEL_CAPACITY: usize = 16;

const DEFAULT_NAME_FOR_UNNAMED_MIDI_PORTS: &str = "Name Not Available";
const MIDI_MESSAGE_IGNORE_VALUE: Ignore = Ignore::SysexAndTime;

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum CC {
    ModWheel(u8),
    VelocityCurve(u8),
    Volume(u8),
    Balance(u8),
    SubOscillatorShapeParameter1(u8),
    SubOscillatorShapeParameter2(u8),
    Oscillator1ShapeParameter1(u8),
    Oscillator1ShapeParameter2(u8),
    Oscillator2ShapeParameter1(u8),
    Oscillator2ShapeParameter2(u8),
    Oscillator3ShapeParameter1(u8),
    Oscillator3ShapeParameter2(u8),
    OscillatorKeySyncEnabled(u8),
    SubOscillatorShape(u8),
    Oscillator1Shape(u8),
    Oscillator2Shape(u8),
    Oscillator3Shape(u8),
    SubOscillatorCourseTune(u8),
    Oscillator1CourseTune(u8),
    Oscillator2CourseTune(u8),
    Oscillator3CourseTune(u8),
    SubOscillatorFineTune(u8),
    Oscillator1FineTune(u8),
    Oscillator2FineTune(u8),
    Oscillator3FineTune(u8),
    SubOscillatorLevel(u8),
    Oscillator1Level(u8),
    Oscillator2Level(u8),
    Oscillator3Level(u8),
    SubOscillatorMute(u8),
    Oscillator1Mute(u8),
    Oscillator2Mute(u8),
    Oscillator3Mute(u8),
    SubOscillatorBalance(u8),
    Oscillator1Balance(u8),
    Oscillator2Balance(u8),
    Oscillator3Balance(u8),
    FilterPoles(u8),
    FilterResonance(u8),
    FilterCutoff(u8),
    AmpEGReleaseTime(u8),
    AmpEGAttackTime(u8),
    AmpEGDecayTime(u8),
    AmpEGSustainLevel(u8),
    AmpEGInverted(u8),
    FilterEGAttackTime(u8),
    FilterEGDecayTime(u8),
    FilterEGSustainLevel(u8),
    FilterEGReleaseTime(u8),
    FilterEGInverted(u8),
    FilterEGAmount(u8),
    LFO1Frequency(u8),
    LFO1CenterValue(u8),
    LFO1Range(u8),
    LFO1WaveShape(u8),
    LFO1Phase(u8),
    LFO1Reset,
    FilterModLFOFrequency(u8),
    FilterModLFOAmount(u8),
    FilterModLFOWaveShape(u8),
    FilterModLFOPhase(u8),
    FilterModLFOReset,
    AllNotesOff,
}

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
    PitchBend(i16),
    ChannelPressure(u8),
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
                log::info!(
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
                        get_supported_cc_from_cc_number(cc_number, cc_value).map(MidiMessage::ControlChange)
                    }
                    MessageType::PitchBend => {
                        let amount_msb = message[MIDI_PITCH_BEND_MSB_BYTE_INDEX];
                        let amount_lsb = message[MIDI_PITCH_BEND_LSB_BYTE_INDEX];
                        let pitch_bend_amount = (amount_msb as u16) << 7 | amount_lsb as u16;
                        Some(MidiMessage::PitchBend(pitch_bend_amount as i16))
                    }
                    MessageType::ChannelPressure => {
                        let pressure_amount = message[MIDI_CHANNEL_PRESSURE_VALUE_BYTE_INDEX];
                        Some(MidiMessage::ChannelPressure(pressure_amount))
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
    let mut midi_input = match MidiInput::new(MIDI_INPUT_CLIENT_NAME) {
        Ok(port) => port,
        Err(err) => {
            log::error!("get_default_midi_device(): Could not create a new MIDI input object.");
            return Err(anyhow!(err));
        }
    };

    midi_input.ignore(MIDI_MESSAGE_IGNORE_VALUE);

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

fn get_supported_cc_from_cc_number(cc_number: u8, cc_value: u8) -> Option<CC> {
    match cc_number {
        1 => Some(CC::ModWheel(cc_value)),
        3 => Some(CC::VelocityCurve(cc_value)),
        7 => Some(CC::Volume(cc_value)),
        10 => Some(CC::Balance(cc_value)),
        12 => Some(CC::SubOscillatorShapeParameter1(cc_value)),
        13 => Some(CC::SubOscillatorShapeParameter2(cc_value)),
        14 => Some(CC::Oscillator1ShapeParameter1(cc_value)),
        15 => Some(CC::Oscillator1ShapeParameter2(cc_value)),
        16 => Some(CC::Oscillator2ShapeParameter1(cc_value)),
        17 => Some(CC::Oscillator2ShapeParameter2(cc_value)),
        18 => Some(CC::Oscillator3ShapeParameter1(cc_value)),
        19 => Some(CC::Oscillator3ShapeParameter2(cc_value)),
        20 => Some(CC::OscillatorKeySyncEnabled(cc_value)),
        40 => Some(CC::SubOscillatorShape(cc_value)),
        41 => Some(CC::Oscillator1Shape(cc_value)),
        42 => Some(CC::Oscillator2Shape(cc_value)),
        43 => Some(CC::Oscillator3Shape(cc_value)),
        44 => Some(CC::SubOscillatorCourseTune(cc_value)),
        45 => Some(CC::Oscillator1CourseTune(cc_value)),
        46 => Some(CC::Oscillator2CourseTune(cc_value)),
        47 => Some(CC::Oscillator3CourseTune(cc_value)),
        48 => Some(CC::SubOscillatorFineTune(cc_value)),
        49 => Some(CC::Oscillator1FineTune(cc_value)),
        50 => Some(CC::Oscillator2FineTune(cc_value)),
        51 => Some(CC::Oscillator3FineTune(cc_value)),
        52 => Some(CC::SubOscillatorLevel(cc_value)),
        53 => Some(CC::Oscillator1Level(cc_value)),
        54 => Some(CC::Oscillator2Level(cc_value)),
        55 => Some(CC::Oscillator3Level(cc_value)),
        56 => Some(CC::SubOscillatorMute(cc_value)),
        57 => Some(CC::Oscillator1Mute(cc_value)),
        58 => Some(CC::Oscillator2Mute(cc_value)),
        59 => Some(CC::Oscillator3Mute(cc_value)),
        60 => Some(CC::SubOscillatorBalance(cc_value)),
        61 => Some(CC::Oscillator1Balance(cc_value)),
        62 => Some(CC::Oscillator2Balance(cc_value)),
        63 => Some(CC::Oscillator3Balance(cc_value)),
        70 => Some(CC::FilterPoles(cc_value)),
        71 => Some(CC::FilterResonance(cc_value)),
        72 => Some(CC::AmpEGReleaseTime(cc_value)),
        73 => Some(CC::AmpEGAttackTime(cc_value)),
        74 => Some(CC::FilterCutoff(cc_value)),
        75 => Some(CC::AmpEGDecayTime(cc_value)),
        79 => Some(CC::AmpEGSustainLevel(cc_value)),
        80 => Some(CC::AmpEGInverted(cc_value)),
        85 => Some(CC::FilterEGAttackTime(cc_value)),
        86 => Some(CC::FilterEGDecayTime(cc_value)),
        87 => Some(CC::FilterEGSustainLevel(cc_value)),
        88 => Some(CC::FilterEGReleaseTime(cc_value)),
        89 => Some(CC::FilterEGInverted(cc_value)),
        90 => Some(CC::FilterEGAmount(cc_value)),
        102 => Some(CC::LFO1Frequency(cc_value)),
        103 => Some(CC::LFO1CenterValue(cc_value)),
        104 => Some(CC::LFO1Range(cc_value)),
        105 => Some(CC::LFO1WaveShape(cc_value)),
        106 => Some(CC::LFO1Phase(cc_value)),
        107 => Some(CC::LFO1Reset),
        108 => Some(CC::FilterModLFOFrequency(cc_value)),
        109 => Some(CC::FilterModLFOAmount(cc_value)),
        110 => Some(CC::FilterModLFOWaveShape(cc_value)),
        111 => Some(CC::FilterModLFOPhase(cc_value)),
        112 => Some(CC::FilterModLFOReset),
        123 => Some(CC::AllNotesOff),
        _ => None,
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
