pub mod constants;
pub mod control_change;
pub mod device_monitor;
pub mod input_listener;

use crate::midi::constants::MIDI_MESSAGE_SENDER_CAPACITY;
use crate::midi::input_listener::{create_midi_input_listener, create_midi_virtual_input};
use anyhow::Result;
use control_change::CC;
use crossbeam_channel::{Receiver, Sender};
use midir::{MidiInputConnection, MidiInputPort};
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
    input_listener: Arc<Mutex<Option<MidiInputConnection<()>>>>,
    virtual_input_port: Arc<Mutex<Option<MidiInputConnection<()>>>>,
    current_note: Arc<Mutex<Option<u8>>>,
    current_channel: Arc<Mutex<Option<u8>>>,
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
            virtual_input_port: Arc::new(Mutex::new(None)),
            current_note: Arc::new(Mutex::new(None)),
            current_channel: Arc::new(Mutex::new(None)),
        }
    }

    pub fn get_midi_message_receiver(&self) -> Receiver<Event> {
        self.message_receiver.clone()
    }

    pub fn run(&mut self, show_menu: bool) -> Result<()> {
        log::info!("Creating MIDI input port monitor.");
        let mut device_monitor = device_monitor::DeviceMonitor::new();

        log::debug!("run(): Creating Virutal Midi Input Device.");
        self.create_virtual_input_port()?;

        log::info!("run(): Creating MIDI input connection listener.");
        let input_port_receiver = device_monitor.get_input_port_receiver();
        self.create_control_listener(input_port_receiver);

        log::info!("run(): Running the midi device monitor");
        device_monitor.run(show_menu)?;

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

    fn create_control_listener(&mut self, input_port_receiver: Receiver<Option<MidiInputPort>>) {
        let mut input_listener_arc = self.input_listener.clone();
        let current_channel_arc = self.current_channel.clone();
        let message_sender_arc = self.message_sender.clone();
        let current_note_arc = self.current_note.clone();

        thread::spawn(move || {
            log::debug!("create_control_listener(): Midi control listener thread running");

            while let Ok(new_port) = input_port_receiver.recv() {
                match new_port {
                    Some(input_port) => {
                        let new_input_listener = create_midi_input_listener(
                            &input_port,
                            current_channel_arc.clone(),
                            message_sender_arc.clone(),
                            current_note_arc.clone(),
                        ).expect("create_control_listener(): FATAL ERROR: midi input connection creation failure. Exiting. Error: {err}.");

                        let mut input_listener = input_listener_arc
                            .lock()
                            .unwrap_or_else(PoisonError::into_inner);

                        *input_listener = Some(new_input_listener);
                    }
                    None => {
                        close_midi_input_connection(&mut input_listener_arc);
                    }
                }
            }
        });
    }
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
