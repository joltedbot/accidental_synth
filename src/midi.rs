pub mod constants;
pub mod control_change;
pub mod device_monitor;
pub mod input_listener;

use crate::midi::constants::{
    MESSAGE_TYPE_IGNORE_LIST, MIDI_INPUT_CLIENT_NAME, MIDI_INPUT_CONNECTION_NAME,
    MIDI_MESSAGE_SENDER_CAPACITY,
};
use crate::midi::input_listener::{create_midi_input_listener, process_midi_message};
use crate::ui::UIUpdates;

use anyhow::Result;
use control_change::CC;
use crossbeam_channel::{Receiver, Sender};
use midir::os::unix::VirtualInput;
use midir::{MidiInput, MidiInputConnection, MidiInputPort};
use std::sync::{Arc, Mutex, PoisonError};
use std::thread;
use thiserror::Error;

/// Errors that can occur during MIDI operations.
#[derive(Debug, Clone, Error)]
pub enum MidiError {
    #[error("Failed to create MIDI input connection")]
    InputConnectionFailed,

    #[error("MIDI message channel send failed")]
    MessageSendFailed,
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
enum Status {
    NoteOff,
    NoteOn,
    PolyphonicKeyPressure,
    ControlChange,
    ProgramChange,
    ChannelPressure,
    PitchBend,
    Unknown,
}

#[derive(PartialEq, Clone)]
pub enum MidiDeviceUpdateEvents {
    InputPortList(Vec<String>),
    InputPort(Option<(usize, MidiInputPort)>),
    UIMidiInputPort(String),
    UIMidiInputChannelIndex(String),
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum Event {
    NoteOn(u8, u8),
    NoteOff,
    ControlChange(CC),
    PitchBend(u16),
    ChannelPressure(u8),
}

pub struct Midi {
    message_sender: Sender<Event>,
    message_receiver: Receiver<Event>,
    ui_update_receiver: Receiver<MidiDeviceUpdateEvents>,
    device_update_sender: Sender<MidiDeviceUpdateEvents>,
    input_listener: Arc<Mutex<Option<MidiInputConnection<()>>>>,
    virtual_input_port: Arc<Mutex<Option<MidiInputConnection<()>>>>,
    current_note: Arc<Mutex<Option<u8>>>,
    current_channel: Arc<Mutex<Option<u8>>>,
}

impl Midi {
    pub fn new() -> Self {
        log::debug!(target: "midi", "Constructing Midi module");

        let (message_sender, message_receiver) =
            crossbeam_channel::bounded(MIDI_MESSAGE_SENDER_CAPACITY);

        let (ui_update_sender, ui_update_receiver) =
            crossbeam_channel::bounded(MIDI_MESSAGE_SENDER_CAPACITY);

        Self {
            message_sender,
            message_receiver,
            device_update_sender: ui_update_sender,
            ui_update_receiver,
            input_listener: Arc::new(Mutex::new(None)),
            virtual_input_port: Arc::new(Mutex::new(None)),
            current_note: Arc::new(Mutex::new(None)),
            current_channel: Arc::new(Mutex::new(None)),
        }
    }

    pub fn get_midi_message_receiver(&self) -> Receiver<Event> {
        self.message_receiver.clone()
    }

    pub fn get_device_update_sender(&self) -> Sender<MidiDeviceUpdateEvents> {
        self.device_update_sender.clone()
    }

    pub fn run(&mut self, ui_update_sender: Sender<UIUpdates>) -> Result<()> {
        log::debug!(target: "midi", "Starting MIDI module");

        log::debug!(target: "midi", "Creating input port monitor");
        let mut device_monitor =
            device_monitor::DeviceMonitor::new(self.get_device_update_sender());

        log::debug!(target: "midi", "Creating virtual input device");
        self.create_virtual_input_port()?;

        log::debug!(target: "midi", "Creating input connection listener");
        self.create_control_listener(self.ui_update_receiver.clone(), ui_update_sender);

        log::debug!(target: "midi", "Starting device monitor");
        device_monitor.run()?;

        Ok(())
    }

    fn create_virtual_input_port(&self) -> Result<()> {
        let virtual_input_port_arc = self.virtual_input_port.clone();
        let current_channel_arc = self.current_channel.clone();
        let message_sender_arc = self.message_sender.clone();
        let current_note_arc = self.current_note.clone();

        let new_virtual_input_port = create_midi_virtual_input(
            current_channel_arc.clone(),
            message_sender_arc.clone(),
            current_note_arc.clone(),
        )?;

        let mut virtual_input_port = virtual_input_port_arc
            .lock()
            .unwrap_or_else(PoisonError::into_inner);

        *virtual_input_port = Some(new_virtual_input_port);

        Ok(())
    }

    fn create_control_listener(
        &mut self,
        device_update_receiver: Receiver<MidiDeviceUpdateEvents>,
        ui_update_sender: Sender<UIUpdates>,
    ) {
        let mut input_listener_arc = self.input_listener.clone();
        let current_channel_arc = self.current_channel.clone();
        let message_sender_arc = self.message_sender.clone();
        let current_note_arc = self.current_note.clone();

        thread::spawn(move || {
            log::debug!(target: "midi::control", "Control listener thread started");

            while let Ok(update) = device_update_receiver.recv() {
                match update {
                    MidiDeviceUpdateEvents::InputPortList(input_ports) => {
                        log::debug!(
                            target: "midi::control",
                            port_count = input_ports.len();
                            "Received input port list"
                        );
                        ui_update_sender
                            .send(UIUpdates::MidiPortList(input_ports))
                            .expect(
                                "run(): Could not send midi port list update to the UI. Exiting.",
                            );
                    }
                    MidiDeviceUpdateEvents::InputPort(input_port) => {
                        log::debug!(
                            target: "midi::control",
                            has_port = input_port.is_some();
                            "Received input port update"
                        );
                        if let Some(port) = input_port {
                            reload_midi_input_listener(
                                &mut input_listener_arc,
                                &current_channel_arc,
                                &message_sender_arc,
                                &current_note_arc,
                                &port.1,
                            );

                            ui_update_sender
                                .send(UIUpdates::MidiPortIndex(port.0 as i32))
                                .expect(
                                    "run(): Could not send midi port list update to the UI. Exiting.");
                        } else {
                            close_midi_input_connection(&mut input_listener_arc);
                        }
                    }
                    MidiDeviceUpdateEvents::UIMidiInputPort(port_name) => {
                        log::debug!(
                            target: "midi::control",
                            port_name = port_name.as_str();
                            "UI requested port change"
                        );
                        if let Some(port) = midi_port_from_port_name(&port_name) {
                            reload_midi_input_listener(
                                &mut input_listener_arc,
                                &current_channel_arc,
                                &message_sender_arc,
                                &current_note_arc,
                                &port.1,
                            );

                            ui_update_sender
                                .send(UIUpdates::MidiPortIndex(port.0 as i32))
                                .expect(
                                    "run(): Could not send midi port index update to the UI. Exiting.");
                        } else {
                            log::warn!(
                                target: "midi::control",
                                port_name = port_name.as_str();
                                "Requested port not found"
                            );
                            close_midi_input_connection(&mut input_listener_arc);
                        }
                    }
                    MidiDeviceUpdateEvents::UIMidiInputChannelIndex(channel_index) => {
                        log::debug!(
                            target: "midi::control",
                            channel = channel_index.as_str();
                            "Channel filter changed"
                        );
                        let mut current_channel = current_channel_arc
                            .lock()
                            .unwrap_or_else(PoisonError::into_inner);
                        *current_channel = channel_index.parse().ok();

                        let channel_index_number = i32::from(current_channel.unwrap_or(0));
                        ui_update_sender
                            .send(UIUpdates::MidiChannelIndex(channel_index_number))
                            .expect(
                                "run(): Could not send midi channel update to the UI. Exiting.",
                            );
                    }
                }
            }
        });
    }
}

fn reload_midi_input_listener(
    input_listener_arc: &mut Arc<Mutex<Option<MidiInputConnection<()>>>>,
    current_channel_arc: &Arc<Mutex<Option<u8>>>,
    message_sender_arc: &Sender<Event>,
    current_note_arc: &Arc<Mutex<Option<u8>>>,
    port: &MidiInputPort,
) {
    log::info!(target: "midi::input", "Reloading input listener");

    let new_input_listener = match create_midi_input_listener(
        port,
        current_channel_arc.clone(),
        message_sender_arc.clone(),
        current_note_arc.clone(),
    ) {
        Ok(listener) => listener,
        Err(err) => {
            let midi_err = MidiError::InputConnectionFailed;
            log::error!(
                target: "midi::input",
                error:% = midi_err,
                details:% = err;
                "Failed to create the input listener"
            );
            panic!("{midi_err}");
        }
    };

    let mut input_listener = input_listener_arc
        .lock()
        .unwrap_or_else(PoisonError::into_inner);

    *input_listener = Some(new_input_listener);
}

fn close_midi_input_connection(
    input_listener_arc: &mut Arc<Mutex<Option<MidiInputConnection<()>>>>,
) {
    let mut input_listener = input_listener_arc
        .lock()
        .unwrap_or_else(PoisonError::into_inner);
    *input_listener = None;
    log::info!(target: "midi::input", "Input connection closed");
}

fn midi_port_from_port_name(port_name: &str) -> Option<(usize, MidiInputPort)> {
    let midi_input = MidiInput::new(MIDI_INPUT_CLIENT_NAME).ok()?;
    midi_input
        .ports()
        .iter()
        .enumerate()
        .find(|port| midi_input.port_name(port.1).unwrap_or_default() == port_name)
        .map(|port| (port.0, port.1.clone()))
}

pub fn create_midi_virtual_input(
    current_channel_arc: Arc<Mutex<Option<u8>>>,
    midi_message_sender: Sender<Event>,
    current_note_arc: Arc<Mutex<Option<u8>>>,
) -> Result<MidiInputConnection<()>> {
    let mut midi_input = MidiInput::new(MIDI_INPUT_CLIENT_NAME)?;
    midi_input.ignore(MESSAGE_TYPE_IGNORE_LIST);

    let connection_result = midi_input.create_virtual(
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
