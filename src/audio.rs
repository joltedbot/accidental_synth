use anyhow::{Result, anyhow};
use cpal::traits::{DeviceTrait, HostTrait};
use cpal::{Device, Host, default_host, SampleRate, StreamConfig};
use crossbeam_channel::{Receiver, Sender, bounded};
use std::thread;
use std::thread::sleep;
use std::time::Duration;
use thiserror::Error;

const INPUT_PORT_SENDER_CAPACITY: usize = 5;
const DEVICE_LIST_POLLING_INTERVAL: u64 = 2000;
const DEFAULT_AUDIO_DEVICE_INDEX: usize = 0;
const DEFAULT_LEFT_CHANNEL_INDEX: usize = 0;
const DEFAULT_RIGHT_CHANNEL_INDEX: usize = 1;

#[derive(Debug, Clone)]
pub struct Channels {
    pub left: usize,
    pub right: Option<usize>,
    pub total: u16,
}

#[derive(Clone)]
pub struct OutputDevice {
    pub name: String,
    pub channels: Channels,
    pub device: Device,
}

#[derive(Debug, Clone, Error, PartialEq, Eq)]
pub enum AudioError {
    #[error("No Audio Output Devices Found")]
    NoAudioOutputDevices,

    #[error("No Audio Output Device Channels Found")]
    NoAudioOutputChannels,
}

pub struct Audio {
    sample_rate: u32,
    output_device_sender: Sender<Option<OutputDevice>>,
    output_device_receiver: Receiver<Option<OutputDevice>>,
}

impl Audio {
    pub fn new() -> Result<Self> {
        log::info!("Constructing Audio Module");
        let default_output_device = default_audio_output_device()?;
        let (output_device_sender, output_device_receiver) = bounded(INPUT_PORT_SENDER_CAPACITY);

        let sample_rate = default_output_device
            .default_output_config()?
            .sample_rate()
            .0;

        Ok(Self {
            sample_rate,
            output_device_sender,
            output_device_receiver,
        })
    }

    pub fn sample_rate(&self) -> u32 {
        self.sample_rate
    }

    pub fn get_output_device_receiver(&self) -> Receiver<Option<OutputDevice>> {
        self.output_device_receiver.clone()
    }

    pub fn run(&mut self) {
        let output_device_sender = self.output_device_sender.clone();
        let host = default_host();
        let mut current_output_device_list = Vec::new();
        let mut current_output_device = None;

        thread::spawn(move || {
            loop {
                let is_changed =
                    update_current_output_device_list(&host, &mut current_output_device_list);

                if is_changed
                    && update_current_output_device(
                        &host,
                        &current_output_device_list,
                        &mut current_output_device,
                    )
                {
                    output_device_sender.send(current_output_device.clone()).expect("Midi Device \
                    Monitor run(): Could not send device update to the input port sender. Exiting. ");
                }

                sleep(Duration::from_millis(DEVICE_LIST_POLLING_INTERVAL));
            }
        });
    }
}

fn default_audio_output_device() -> Result<Device> {
    let default_output_device = default_host().default_output_device();
    if let Some(device) = default_output_device {
        log::debug!(
            "default_audio_output_device(): Using default audio output device: {}",
            device.name().unwrap_or("Unknown".to_string())
        );
        Ok(device)
    } else {
        log::error!("default_audio_output_device(): No default audio output device found.");
        Err(anyhow!(AudioError::NoAudioOutputDevices))
    }
}

fn update_current_output_device_list(host: &Host, current_device_list: &mut Vec<String>) -> bool {
    let new_device_list = output_audio_device_name_list(host);

    if *current_device_list == new_device_list {
        return false;
    }

    *current_device_list = new_device_list;
    true
}

fn update_current_output_device(
    host: &Host,
    current_device_list: &[String],
    current_output_device: &mut Option<OutputDevice>,
) -> bool {
    match current_output_device {
        None => {
            if current_device_list.is_empty() {
                false
            } else {
                let default_port = current_device_list[DEFAULT_AUDIO_DEVICE_INDEX].clone();
                log::info!("Audio Device List Changed. Using Default Device: {default_port}.");
                *current_output_device = output_device_from_name(host, &default_port);
                true
            }
        }
        Some(output_device) => {
            if current_device_list.is_empty() {
                *current_output_device = None;
                true
            } else if current_device_list.contains(&output_device.name) {
                false
            } else {
                let default_device = current_device_list[DEFAULT_AUDIO_DEVICE_INDEX].clone();
                log::info!(
                    "update_current_output_device(): Audio Device List Changed. Using Default Device: {default_device}."
                );
                *current_output_device = output_device_from_name(host, &default_device);
                true
            }
        }
    }
}

fn output_audio_device_name_list(host: &Host) -> Vec<String> {
    if let Ok(device_list) = host.output_devices() {
        device_list
            .filter_map(|device| device.name().ok())
            .collect()
    } else {
        Vec::new()
    }
}

fn output_device_from_name(host: &Host, name: &str) -> Option<OutputDevice> {
    let mut output_devices = host.output_devices().ok()?;

    output_devices.find_map(|device| {
        if device.name().ok()? == name {
            let channels = default_channels_from_device(&device).ok()?;
            Some(OutputDevice {
                name: name.to_string(),
                channels,
                device,
            })
        } else {
            None
        }
    })
}


fn default_channels_from_device(device: &Device) -> Result<Channels> {
    let total_channels = device.default_output_config()?.channels();

    if total_channels == 0 {
        return Err(anyhow!(AudioError::NoAudioOutputChannels));
    }

    let right = if total_channels > 1 {
        Some(DEFAULT_RIGHT_CHANNEL_INDEX)
    } else {
        None
    };

    Ok(Channels {
        total: total_channels,
        left: DEFAULT_LEFT_CHANNEL_INDEX,
        right,
    })
}
