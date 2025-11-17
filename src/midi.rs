pub mod constants;
pub mod control_change;
pub mod device_monitor;
pub mod input_listener;

use crate::midi::input_listener::{create_midi_input_listener, create_midi_virtual_input};
use crate::midi::constants::{MIDI_INPUT_CLIENT_NAME, MIDI_MESSAGE_SENDER_CAPACITY};
use crate::ui::UIUpdates;

use anyhow::Result;
use control_change::CC;
use crossbeam_channel::{Receiver, Sender};
use midir::{MidiInput, MidiInputConnection, MidiInputPort};
use std::sync::{Arc, Mutex, PoisonError};
use std::thread;

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
pub enum MidiDeviceEvent {
    InputPortListUpdated(Vec<String>),
    InputPortUpdated(Option<(usize, MidiInputPort)>),
    UIMidiInputPortUpdated(String),
    UIMidiInputChannelIndexUpdated(String),
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
    ui_update_receiver: Receiver<MidiDeviceEvent>,
    device_update_sender: Sender<MidiDeviceEvent>,
    input_listener: Arc<Mutex<Option<MidiInputConnection<()>>>>,
    virtual_input_port: Arc<Mutex<Option<MidiInputConnection<()>>>>,
    current_note: Arc<Mutex<Option<u8>>>,
    current_channel: Arc<Mutex<Option<u8>>>,
}

impl Midi {
    pub fn new() -> Self {
        log::info!("Constructing Midi Module");

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

    pub fn get_device_update_sender(&self) -> Sender<MidiDeviceEvent> {
        self.device_update_sender.clone()
    }

    pub fn run(&mut self, ui_update_sender: Sender<UIUpdates>) -> Result<()> {
        log::debug!("Creating MIDI input port monitor.");
        let mut device_monitor =
            device_monitor::DeviceMonitor::new(self.get_device_update_sender());

        log::debug!("run(): Creating Virutal Midi Input Device.");
        self.create_virtual_input_port()?;

        log::debug!("Creating MIDI input connection listener.");
        self.create_control_listener(self.ui_update_receiver.clone(), ui_update_sender);

        log::debug!("run(): Running the midi device monitor");
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
        device_update_receiver: Receiver<MidiDeviceEvent>,
        ui_update_sender: Sender<UIUpdates>,
    ) {
        let mut input_listener_arc = self.input_listener.clone();
        let current_channel_arc = self.current_channel.clone();
        let message_sender_arc = self.message_sender.clone();
        let current_note_arc = self.current_note.clone();

        thread::spawn(move || {
            log::debug!("create_control_listener(): Midi control listener thread running");

            while let Ok(update) = device_update_receiver.recv() {
                match update {
                    MidiDeviceEvent::InputPortListUpdated(input_ports) => {
                        ui_update_sender
                            .send(UIUpdates::MidiPortList(input_ports))
                            .expect(
                                "run(): Could not send midi port list update to the UI. Exiting.",
                            );
                    }
                    MidiDeviceEvent::InputPortUpdated(input_port) => {
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
                        };
                    }
                    MidiDeviceEvent::UIMidiInputPortUpdated(port_name) => {
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
                            close_midi_input_connection(&mut input_listener_arc);
                        };
                    }
                    MidiDeviceEvent::UIMidiInputChannelIndexUpdated(channel_index) => {
                        let mut current_channel = current_channel_arc
                            .lock()
                            .unwrap_or_else(PoisonError::into_inner);
                        *current_channel = channel_index.parse().ok();

                        let channel_index_number = current_channel.unwrap_or(0) as i32;
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
    let new_input_listener = create_midi_input_listener(
        &port,
        current_channel_arc.clone(),
        message_sender_arc.clone(),
        current_note_arc.clone(),
    ).expect("create_control_listener(): FATAL ERROR: midi input connection creation failure. Exiting. Error: {err}.");

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
    log::info!("close_midi_input_connection(): MIDI input connection closed.");
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
