pub mod constants;
pub mod control_change;
pub mod device_monitor;
pub mod input_listener;

use crate::midi::constants::{
    DEFAULT_MIDI_PORT_INDEX, MIDI_INPUT_CLIENT_NAME, MIDI_MESSAGE_SENDER_CAPACITY,
};
use crate::midi::device_monitor::{DeviceUpdate, get_input_port_list, input_device_from_port};
use crate::midi::input_listener::create_midi_input_listener;
use anyhow::{Result, anyhow};
use control_change::CC;
use crossbeam_channel::{Receiver, Sender};
use midir::{MidiInput, MidiInputConnection, MidiInputPort, MidiInputPorts};
use std::sync::{Arc, Mutex, MutexGuard, PoisonError};
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

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum Event {
    NoteOn(u8, u8),
    NoteOff,
    ControlChange(CC),
    PitchBend(u16),
    ChannelPressure(u8),
}
#[derive(PartialEq, Clone)]
pub struct InputPort {
    pub name: String,
    pub port: MidiInputPort,
}

pub struct Midi {
    message_sender: Sender<Event>,
    message_receiver: Receiver<Event>,
    input_listener: Arc<Mutex<Option<MidiInputConnection<()>>>>,
    input_ports: Arc<Mutex<Option<Vec<String>>>>,
    current_note: Arc<Mutex<Option<u8>>>,
    current_channel: Arc<Mutex<Option<u8>>>,
    current_input: Arc<Mutex<Option<InputPort>>>,
}

impl Midi {
    pub fn new() -> Self {
        log::info!("Constructing Midi Module");

        let (midi_message_sender, midi_message_receiver) =
            crossbeam_channel::bounded(MIDI_MESSAGE_SENDER_CAPACITY);

        Self {
            message_sender: midi_message_sender,
            message_receiver: midi_message_receiver,
            input_listener: Arc::new(Mutex::new(None)),
            input_ports: Arc::new(Mutex::new(None)),
            current_note: Arc::new(Mutex::new(None)),
            current_channel: Arc::new(Mutex::new(None)),
            current_input: Arc::new(Mutex::new(None)),
        }
    }

    pub fn get_midi_message_receiver(&self) -> Receiver<Event> {
        self.message_receiver.clone()
    }

    pub fn run(&mut self) -> Result<()> {
        log::info!("Creating MIDI input connection listener.");

        let mut device_monitor = device_monitor::DeviceMonitor::new();
        let port_list_receiver = device_monitor.get_port_list_receiver();

        self.create_control_listener(port_list_receiver)?;
        device_monitor.run()?;

        self.create_midi_input_listener();

        Ok(())
    }

    fn create_midi_input_listener(&mut self) {
        let current_input_arc = self.current_input.clone();

        let current_input = current_input_arc
            .lock()
            .unwrap_or_else(PoisonError::into_inner);

        if let Some(input) = current_input.clone() {
            self.midi_input_listener(&input);

            log::info!(
                "run(): The MIDI input connection listener has been created for {}.",
                input.name
            );
        }
    }

    fn midi_input_listener(&mut self, input: &InputPort) {
        let mut input_listener = self
            .input_listener
            .lock()
            .unwrap_or_else(PoisonError::into_inner);

        *input_listener = create_midi_input_listener(
            input,
            self.current_channel.clone(),
            self.message_sender.clone(),
            self.current_note.clone(),
        );
    }

    fn create_control_listener(
        &mut self,
        port_list_receiver: Receiver<DeviceUpdate>,
    ) -> Result<()> {
        let input_port_list_arc = self.input_ports.clone();
        let current_input_arc = self.current_input.clone();
        let mut input_listener_arc = self.input_listener.clone();
        let current_channel_arc = self.current_channel.clone();
        let message_sender_arc = self.message_sender.clone();
        let current_note_arc = self.current_note.clone();
        let midi_input = MidiInput::new(MIDI_INPUT_CLIENT_NAME)?;

        thread::spawn(move || {
            while let Ok(event) = port_list_receiver.recv() {
                match event {
                    DeviceUpdate::InputPortList(new_port_list) => {
                        let current_port_list = update_current_port_list(
                            &input_port_list_arc,
                            &midi_input,
                            new_port_list,
                        );

                        let current_input = current_input_arc
                            .lock()
                            .unwrap_or_else(PoisonError::into_inner);

                        let new_input_port = update_current_input(
                            &mut input_listener_arc,
                            &midi_input,
                            current_port_list,
                            current_input,
                        );

                        if let Some(default_port) = new_input_port {
                            let mut input_listener = input_listener_arc
                                .lock()
                                .unwrap_or_else(PoisonError::into_inner);

                            *input_listener = create_midi_input_listener(
                                &default_port,
                                current_channel_arc.clone(),
                                message_sender_arc.clone(),
                                current_note_arc.clone(),
                            );
                        }
                    }
                }
            }
        });

        Ok(())
    }
}

fn update_current_input(
    mut input_listener_arc: &mut Arc<Mutex<Option<MidiInputConnection<()>>>>,
    midi_input: &MidiInput,
    current_port_list: Option<Vec<String>>,
    mut current_input: MutexGuard<Option<InputPort>>,
) -> Option<InputPort> {
    let new_input_port = match current_port_list.as_ref() {
        None => {
            *current_input = None;
            close_midi_input_connection(&mut input_listener_arc);
            log::warn!(
                "create_midi_control_listener(): Current Midi Input Port List is No Longer Available. Continuing without MIDI input."
            );
            None
        }
        Some(port_list) => {
            if let Some(input) = current_input.as_ref()
                && port_list.contains(&input.name)
            {
                return None
            }



            let default_input_port_name = port_list[DEFAULT_MIDI_PORT_INDEX].clone();

            let default_input_port =
                input_port_from_name(&default_input_port_name, &midi_input);

            (*current_input).clone_from(&default_input_port);

            log::debug!(
                "update_current_input_port(): MIDI port list changed and the current port is no longer available.\
                 Using the default:{default_input_port_name}"
            );

            default_input_port


        }
    };

    new_input_port
}

fn update_current_port_list(
    input_port_list_arc: &Arc<Mutex<Option<Vec<String>>>>,
    midi_input: &MidiInput,
    new_port_list: Option<MidiInputPorts>,
) -> Option<Vec<String>> {
    let mut current_port_list = input_port_list_arc
        .lock()
        .unwrap_or_else(PoisonError::into_inner);

    *current_port_list = match new_port_list {
        Some(device_list) => get_input_port_list(&device_list, &midi_input),
        None => None,
    };
    current_port_list.clone()
}

fn close_midi_input_connection(
    input_listener_arc: &mut Arc<Mutex<Option<MidiInputConnection<()>>>>,
) {
    let mut input_listener = input_listener_arc
        .lock()
        .unwrap_or_else(PoisonError::into_inner);
    *input_listener = None;
}

fn input_port_from_name(port_name: &str, midi_input: &MidiInput) -> Option<InputPort> {
    let port = midi_input
        .ports()
        .iter()
        .find(|port| midi_input.port_name(port).unwrap_or("".to_string()) == port_name.to_string())?
        .clone();

    Some(InputPort {
        name: port_name.to_string(),
        port,
    })
}

#[cfg(test)]
mod tests {}
