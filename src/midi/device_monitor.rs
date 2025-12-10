use crate::midi::MidiDeviceUpdateEvents;
use crate::midi::constants::{
    DEFAULT_MIDI_PORT_INDEX, DEVICE_LIST_POLLING_INTERVAL, MIDI_INPUT_CLIENT_NAME,
    UNKNOWN_MIDI_PORT_NAME_MESSAGE,
};
use anyhow::Result;
use crossbeam_channel::Sender;
use midir::{MidiInput, MidiInputPort, MidiInputPorts};
use std::thread;
use std::thread::sleep;
use std::time::Duration;

pub struct DeviceMonitor {
    device_update_sender: Sender<MidiDeviceUpdateEvents>,
}

impl DeviceMonitor {
    pub fn new(device_update_sender: Sender<MidiDeviceUpdateEvents>) -> Self {
        Self {
            device_update_sender,
        }
    }

    pub fn run(&mut self) -> Result<()> {
        let input_port_sender = self.device_update_sender.clone();
        let midi_input = MidiInput::new(MIDI_INPUT_CLIENT_NAME)?;
        let mut current_port_list = MidiInputPorts::new();

        let mut current_port: Option<(usize, MidiInputPort)> = None;
        thread::spawn(move || {
            loop {
                let port_list_changed =
                    update_current_port_list_if_changed(&midi_input, &mut current_port_list);

                if port_list_changed {
                    let midi_port_names = current_port_list
                        .iter()
                        .filter_map(|port| midi_input.port_name(port).ok())
                        .collect::<Vec<String>>();

                    input_port_sender.send(MidiDeviceUpdateEvents::InputPortList(midi_port_names)).expect(
                        "run(): Could not send midi port list update to the input port sender. Exiting. ",
                    );

                    update_current_port(&current_port_list, &mut current_port);

                    input_port_sender.send(MidiDeviceUpdateEvents::InputPort(current_port.clone())).expect(
                        "run(): Could not send midi port update to the input port sender. Exiting. ",
                    );
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
        log::info!(
            target: "midi::device",
            old_count = current_port_list.len(),
            new_count = new_port_list.len();
            "Input port list changed"
        );
        *current_port_list = new_port_list;
        return true;
    }

    false
}

fn update_current_port(
    current_port_list: &[MidiInputPort],
    current_input_port: &mut Option<(usize, MidiInputPort)>,
) {
    if current_port_list.is_empty() {
        *current_input_port = None;
        return;
    }

    if let Some(index) = current_input_port
        .as_ref()
        .and_then(|(_, port)| current_port_list.iter().position(|p| p == port))
    {
        *current_input_port = Some((index, current_port_list[index].clone()));
        return;
    }
g
    let default_port = current_port_list[DEFAULT_MIDI_PORT_INDEX].clone();
    let port_name = get_input_port_name(&default_port);
    log::info!(
        target: "midi::device",
        port_name = port_name.as_str(),
        port_index = DEFAULT_MIDI_PORT_INDEX;
        "Using default input port"
    );

    *current_input_port = Some((DEFAULT_MIDI_PORT_INDEX, default_port));
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
