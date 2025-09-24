use crate::midi::constants::DEVICE_LIST_POLLING_INTERVAL;
use crate::midi::constants::{MIDI_INPUT_CLIENT_NAME, PANIC_MESSAGE_PORT_LIST_SENDER_FAILURE};
use anyhow::Result;
use crossbeam_channel::{Receiver, Sender, unbounded};
use midir::{MidiInput, MidiInputPorts};
use std::thread;
use std::thread::sleep;
use std::time::Duration;

pub enum DeviceUpdate {
    InputPortList(Option<MidiInputPorts>),
}

pub struct DeviceMonitor {
    port_list_sender: Sender<DeviceUpdate>,
    port_list_receiver: Receiver<DeviceUpdate>,
}

impl DeviceMonitor {
    pub fn new() -> Self {
        let (port_list_sender, port_list_receiver) = unbounded();

        Self {
            port_list_sender,
            port_list_receiver,
        }
    }

    pub fn get_port_list_receiver(&self) -> Receiver<DeviceUpdate> {
        self.port_list_receiver.clone()
    }

    pub fn get_port_list_sender(&self) -> Sender<DeviceUpdate> {
        self.port_list_sender.clone()
    }

    pub fn run(&mut self) -> Result<()> {
        let ports_list_sender = self.get_port_list_sender();
        let midi_input = MidiInput::new(MIDI_INPUT_CLIENT_NAME)?;
        let mut current_port_list = MidiInputPorts::new();

        thread::spawn(move || {
            loop {
                let new_port_list = midi_input.ports();
                if current_port_list != new_port_list {
                    let input_port_list = if new_port_list.is_empty() {
                        None
                    } else {
                        Some(new_port_list.clone())
                    };

                    ports_list_sender.send(DeviceUpdate::InputPortList(input_port_list)).unwrap_or_else(|err| {
                        log::error!("run(): FATAL ERROR: port list sender failure. Exiting. Error: {err}.");
                        panic!("{PANIC_MESSAGE_PORT_LIST_SENDER_FAILURE}");
                    });

                    current_port_list = new_port_list;
                    log::info!("run(): Midi Input Port List Changed. Updating Current Port List.");
                }

                sleep(Duration::from_millis(DEVICE_LIST_POLLING_INTERVAL));
            }
        });

        Ok(())
    }
}

pub fn get_input_port_list(
    ports_list: &MidiInputPorts,
    midi_input: &MidiInput,
) -> Option<Vec<String>> {
    let filtered_port_list: Vec<String> = ports_list
        .iter()
        .filter_map(|port| midi_input.port_name(port).ok())
        .collect();

    if filtered_port_list.is_empty() {
        log::warn!("input_port_map(): No MIDI input ports found. Continuing without MIDI input.");
        return None;
    }

    Some(filtered_port_list)
}



#[cfg(test)]
mod tests {}
