mod callbacks;
mod constants;
mod structs;

use super::{AccidentalSynth, AudioDevice, MidiPort};
use crossbeam_channel::{Receiver, Sender, bounded};
use slint::{ModelRc, SharedString, VecModel, Weak};
use std::sync::{Arc, Mutex, MutexGuard};
use std::thread;

use crate::ui::callbacks::register_callbacks;
use crate::ui::constants::{
    AUDIO_DEVICE_CHANNEL_INDEX_TO_NAME_OFFSET, AUDIO_DEVICE_CHANNEL_NULL_VALUE, MIDI_CHANNEL_LIST,
};
use crate::ui::structs::{UIAudioDevice, UIMidiPort};
use anyhow::Result;

const UI_UPDATE_CHANNEL_CAPACITY: usize = 10;

#[derive(Debug, Clone, PartialEq)]
pub enum UIUpdates {
    MidiPortList(Vec<String>),
    AudioDeviceList(Vec<String>),
    AudioDeviceIndex(usize),
    AudioDeviceChannels {
        left: usize,
        right: Option<usize>,
        count: u16,
    },
}

pub struct UI {
    ui_update_sender: Sender<UIUpdates>,
    ui_update_receiver: Receiver<UIUpdates>,
    audio_device_values: Arc<Mutex<UIAudioDevice>>,
    midi_port_values: Arc<Mutex<UIMidiPort>>,
}

impl UI {
    pub fn new() -> Self {
        log::info!("Constructing UI Module");

        let (ui_update_sender, ui_update_receiver) = bounded(UI_UPDATE_CHANNEL_CAPACITY);

        let midi_port_values = UIMidiPort {
            channels: MIDI_CHANNEL_LIST
                .iter()
                .map(|channel| channel.to_string())
                .collect(),
            ..Default::default()
        };

        let audio_device_values = UIAudioDevice {
            left_channel_index: 0,
            right_channel_index: 1,
            ..Default::default()
        };

        Self {
            ui_update_sender,
            ui_update_receiver,
            audio_device_values: Arc::new(Mutex::new(audio_device_values)),
            midi_port_values: Arc::new(Mutex::new(midi_port_values)),
        }
    }

    pub fn get_ui_update_sender(&self) -> Sender<UIUpdates> {
        self.ui_update_sender.clone()
    }

    pub fn run(&mut self, ui_weak: Weak<AccidentalSynth>) -> Result<()> {
        let ui_update_receiver = self.ui_update_receiver.clone();
        register_callbacks(ui_weak.clone())?;
        self.set_ui_default_values(ui_weak.clone())?;

        self.start_ui_update_listener(ui_update_receiver, ui_weak.clone())?;

        Ok(())
    }

    fn set_ui_default_values(&self, ui_weak: Weak<AccidentalSynth>) -> Result<()> {
        let midi_port_values = self.midi_port_values.clone();
        let audio_device_values = self.audio_device_values.clone();

        ui_weak.upgrade_in_event_loop(move |ui| {
            ui.set_version(SharedString::from(env!("CARGO_PKG_VERSION")));

            let midi_ports = midi_port_values
                .lock()
                .unwrap_or_else(|poisoned| poisoned.into_inner());
            let audio_devices = audio_device_values
                .lock()
                .unwrap_or_else(|poisoned| poisoned.into_inner());

            ui.set_midi_port_values(slint_midi_port_from_ui_midi_port(midi_ports.clone()));
            ui.set_audio_device_values(slint_audio_device_from_ui_audio_device(
                audio_devices.clone(),
            ));
        })?;

        Ok(())
    }

    fn start_ui_update_listener(
        &self,
        ui_update_receiver: Receiver<UIUpdates>,
        ui_weak: Weak<AccidentalSynth>,
    ) -> Result<()> {
        let midi_port_values = self.midi_port_values.clone();
        let audio_device_values = self.audio_device_values.clone();
        let ui_weak_thread = ui_weak.clone();

        thread::spawn(move || {
            log::debug!("start_ui_update_listener(): spawned thread to receive ui update events");
            while let Ok(update) = ui_update_receiver.recv() {
                match update {
                    UIUpdates::MidiPortList(port_list) => {
                        set_midi_port_list(&ui_weak_thread, &midi_port_values, port_list);
                    }
                    UIUpdates::AudioDeviceList(device_list) => {
                        set_audio_device_list(&ui_weak_thread, &audio_device_values, device_list);
                    }
                    UIUpdates::AudioDeviceIndex(index) => {
                        set_audio_device_index(&ui_weak_thread, &audio_device_values, index);
                    }
                    UIUpdates::AudioDeviceChannels { left, right, count } => {
                        set_audio_device_channels(
                            &ui_weak_thread,
                            &audio_device_values,
                            (left, right, count),
                        );
                    }
                }
            }
        });

        Ok(())
    }
}

fn set_midi_port_list(
    ui_weak_thread: &Weak<AccidentalSynth>,
    midi_port_values: &Arc<Mutex<UIMidiPort>>,
    port_list: Vec<String>,
) {
    let mut midi_ports = midi_port_values
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    midi_ports.input_ports = port_list;
    let ui_midi_ports = midi_ports.clone();
    let _ = ui_weak_thread.upgrade_in_event_loop(move |ui| {
        ui.set_midi_port_values(slint_midi_port_from_ui_midi_port(ui_midi_ports));
    });
}

fn set_audio_device_list(
    ui_weak_thread: &Weak<AccidentalSynth>,
    audio_device_values: &Arc<Mutex<UIAudioDevice>>,
    device_list: Vec<String>,
) {
    let mut audio_devices = audio_device_values
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    audio_devices.output_devices = device_list;
    let ui_audio_devices = audio_devices.clone();
    let _ = ui_weak_thread.upgrade_in_event_loop(move |ui| {
        ui.set_audio_device_values(slint_audio_device_from_ui_audio_device(ui_audio_devices));
    });
}

fn set_audio_device_index(
    ui_weak: &Weak<AccidentalSynth>,
    audio_device_values: &Arc<Mutex<UIAudioDevice>>,
    audio_device_index: usize,
) {
    println!("{:?}", audio_device_index);
    let mut audio_devices = audio_device_values.lock().unwrap_or_else(|poisoned| poisoned.into_inner());
    audio_devices.output_device_index = audio_device_index as i32;
    let ui_audio_devices = audio_devices.clone();
    let _ = ui_weak.upgrade_in_event_loop(move |ui| {
        ui.set_audio_device_values(slint_audio_device_from_ui_audio_device(ui_audio_devices));
    });
}

fn set_audio_device_channels(
    ui_weak_thread: &Weak<AccidentalSynth>,
    audio_device_values: &Arc<Mutex<UIAudioDevice>>,
    channels: (usize, Option<usize>, u16),
) {
    let mut audio_devices = audio_device_values
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());

    let mut device_channels: Vec<String> = vec![];
    for channel in 0..channels.2 {
        device_channels.push((channel + AUDIO_DEVICE_CHANNEL_INDEX_TO_NAME_OFFSET).to_string());
    }

    audio_devices.left_channel_index = channels.0 as i32;
    audio_devices.left_channels = device_channels.clone();
    if let Some(channel) = channels.1 {
        audio_devices.right_channels = device_channels;
        audio_devices.right_channel_index = channel as i32;
    } else {
        audio_devices.right_channels = vec![];
        audio_devices.right_channel_index = AUDIO_DEVICE_CHANNEL_NULL_VALUE;
    };

    let ui_audio_devices = audio_devices.clone();
    let _ = ui_weak_thread.upgrade_in_event_loop(move |ui| {
        ui.set_audio_device_values(slint_audio_device_from_ui_audio_device(ui_audio_devices));
    });
}

fn slint_audio_device_from_ui_audio_device(audio_device_values: UIAudioDevice) -> AudioDevice {
    AudioDevice {
        output_device_index: audio_device_values.output_device_index,
        left_channel_index: audio_device_values.left_channel_index,
        right_channel_index: audio_device_values.right_channel_index,
        sample_rate_index: audio_device_values.sample_rate_index,
        output_devices: vec_to_modelrc(&audio_device_values.output_devices),
        left_channels: vec_to_modelrc(&audio_device_values.left_channels),
        right_channels: vec_to_modelrc(&audio_device_values.right_channels),
        sample_rates: vec_to_modelrc(&audio_device_values.sample_rates),
    }
}

fn slint_midi_port_from_ui_midi_port(midi_port_values: UIMidiPort) -> MidiPort {
    MidiPort {
        input_ports: vec_to_modelrc(&midi_port_values.input_ports),
        channels: vec_to_modelrc(&midi_port_values.channels),
        input_port_index: midi_port_values.input_port_index,
        channel_index: midi_port_values.channel_index,
    }
}

fn vec_to_modelrc(input_values: &Vec<String>) -> ModelRc<SharedString> {
    ModelRc::new(VecModel::from(
        input_values
            .iter()
            .map(|value| SharedString::from(value))
            .collect::<Vec<SharedString>>(),
    ))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crossbeam_channel::internal::SelectHandle;

    #[test]
    fn new_returns_correct_object_contents() {
        let ui = UI::new();
        let ui_update_sender = ui.get_ui_update_sender();
        assert!(ui_update_sender.is_ready());
    }
}
