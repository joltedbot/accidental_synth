use crate::midi::constants::{
    DEFAULT_MIDI_PORT_INDEX, DEVICE_LIST_POLLING_INTERVAL, INPUT_PORT_SENDER_CAPACITY,
    MIDI_INPUT_CLIENT_NAME, UNKNOWN_MIDI_PORT_NAME_MESSAGE,
};
use anyhow::Result;
use crossbeam_channel::{Receiver, Sender, bounded};
use midir::{MidiInput, MidiInputPort, MidiInputPorts};
use std::thread;
use std::thread::sleep;
use std::time::Duration;

pub struct DeviceMonitor {
    input_port_sender: Sender<Option<MidiInputPort>>,
    input_port_receiver: Receiver<Option<MidiInputPort>>,
}

impl DeviceMonitor {
    pub fn new() -> Self {
        let (input_port_sender, input_port_receiver) = bounded(INPUT_PORT_SENDER_CAPACITY);

        Self {
            input_port_sender,
            input_port_receiver,
        }
    }

    pub fn get_input_port_receiver(&self) -> Receiver<Option<MidiInputPort>> {
        self.input_port_receiver.clone()
    }

    pub fn run(&mut self) -> Result<()> {
        let input_port_sender = self.input_port_sender.clone();
        let midi_input = MidiInput::new(MIDI_INPUT_CLIENT_NAME)?;
        let mut current_port_list = MidiInputPorts::new();
        let mut current_port: Option<MidiInputPort> = None;

        thread::spawn(move || {
            loop {
                let is_changed =
                    update_current_port_list_if_changed(&midi_input, &mut current_port_list);

                if is_changed
                    && update_current_port_if_changed(&current_port_list, &mut current_port)
                {
                    input_port_sender.send(current_port.clone()).expect("Midi Device \
                    Monitor run(): Could not send device update to the input port sender. Exiting. ");
                }

                sleep(Duration::from_millis(DEVICE_LIST_POLLING_INTERVAL));
            }
        });

        Ok(())
    }
}

fn update_current_port_list_if_changed(
    midi_input: &MidiInput,
    current_port_list: &mut Vec<MidiInputPort>,
) -> bool {
    let new_port_list: Vec<MidiInputPort> = midi_input.ports();
    if *current_port_list != new_port_list {
        *current_port_list = new_port_list;
        log::info!("run(): Midi Input Port List Changed. Updating Current Port List.");
        return true;
    }
    false
}

fn update_current_port_if_changed(
    current_port_list: &[MidiInputPort],
    current_input_port: &mut Option<MidiInputPort>,
) -> bool {
    match current_input_port {
        None => {
            if current_port_list.is_empty() {
                false
            } else {
                let default_port = current_port_list[DEFAULT_MIDI_PORT_INDEX].clone();
                log::info!(
                    "run(): Midi Input Port Changed. Using Default Port: {}.",
                    get_input_port_name(&default_port)
                );
                *current_input_port = Some(default_port);
                true
            }
        }
        Some(input_port) => {
            if current_port_list.is_empty() {
                *current_input_port = None;
                true
            } else if current_port_list.contains(input_port) {
                false
            } else {
                let default_port = current_port_list[DEFAULT_MIDI_PORT_INDEX].clone();
                log::info!(
                    "run(): Midi Input Port Changed. Using Default Port: {}.",
                    get_input_port_name(&default_port)
                );
                *current_input_port = Some(default_port);
                true
            }
        }
    }
}

fn get_input_port_name(input_port: &MidiInputPort) -> String {
    if let Ok(midi_input) = MidiInput::new(MIDI_INPUT_CLIENT_NAME) {
        midi_input
            .port_name(input_port)
            .unwrap_or(UNKNOWN_MIDI_PORT_NAME_MESSAGE.to_string())
    } else {
        UNKNOWN_MIDI_PORT_NAME_MESSAGE.to_string()
    }
}

#[cfg(test)]
mod tests {}
