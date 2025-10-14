use anyhow::{Result, anyhow};
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use cpal::{Device, Host, Stream, default_host};
use crossbeam_channel::{Receiver, Sender, bounded};
use rtrb::{Consumer, Producer, RingBuffer};
use std::thread;
use std::thread::sleep;
use std::time::{Duration, Instant};
use thiserror::Error;

const DEFAULT_AUDIO_DEVICE_INDEX: usize = 0;
const DEFAULT_LEFT_CHANNEL_INDEX: usize = 0;
const DEFAULT_RIGHT_CHANNEL_INDEX: usize = 1;
const DEVICE_LIST_POLLING_INTERVAL: u64 = 2000;
const SAMPLE_BUFFER_SENDER_CAPACITY: usize = 5;
const SAMPLE_BUFFER_CAPACITY: usize = 8192;
pub const SAMPLE_BUFFER_CHUNK_SIZE: usize = 256;
const MONO_CHANNELS: u16 = 1;

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

#[derive(Debug, Clone, Error)]
pub enum AudioError {
    #[error("No Audio Output Devices Found")]
    NoAudioOutputDevices,

    #[error("No Audio Output Device Channels Found")]
    NoAudioOutputChannels,
}

pub struct Audio {
    sample_rate: u32,
    sample_buffer_sender: Sender<Producer<f32>>,
    sample_buffer_receiver: Receiver<Producer<f32>>,
}

impl Audio {
    pub fn new() -> Result<Self> {
        log::info!("Constructing Audio Module");
        let (output_device_sender, output_device_receiver) = bounded(SAMPLE_BUFFER_SENDER_CAPACITY);

        let default_output_device = default_audio_output_device()?;
        let sample_rate = default_output_device
            .default_output_config()?
            .sample_rate()
            .0;

        Ok(Self {
            sample_rate,
            sample_buffer_sender: output_device_sender,
            sample_buffer_receiver: output_device_receiver,
        })
    }

    pub fn get_sample_rate(&self) -> u32 {
        self.sample_rate
    }

    pub fn get_sample_buffer_receiver(&self) -> Receiver<Producer<f32>> {
        self.sample_buffer_receiver.clone()
    }

    pub fn run(&mut self) {
        let sample_producer_sender = self.sample_buffer_sender.clone();
        let host = default_host();
        let mut current_output_device_list = Vec::new();
        let mut current_output_device = None;

        thread::spawn(move || {
            let mut audio_output_stream = None;
            log::debug!("run(): Audio device monitor thread running");
            loop {
                let is_changed = update_current_output_device_list_if_changed(
                    &host,
                    &mut current_output_device_list,
                );

                if is_changed
                    && update_current_output_device_if_changed(
                        &host,
                        &current_output_device_list,
                        &mut current_output_device,
                    )
                {
                    if let Some(output_device) = &current_output_device {
                        log::debug!("run(): Output device changed. New device: {:?}", output_device.name);

                        let (sample_producer, sample_consumer) =
                            RingBuffer::<f32>::new(SAMPLE_BUFFER_CAPACITY);
                        sample_producer_sender.send(sample_producer).expect("run(): Could not send device update to the audio output device sender. Exiting. ");
                        drop(audio_output_stream);
                        audio_output_stream =
                            start_main_audio_output_loop(output_device, sample_consumer).ok();
                    } else {
                        audio_output_stream = None;
                    }
                }

                sleep(Duration::from_millis(DEVICE_LIST_POLLING_INTERVAL));
            }
        });
    }
}

fn start_main_audio_output_loop(
    output_device: &OutputDevice,
    mut sample_buffer: Consumer<f32>,
) -> Result<Stream> {
    let default_device_stream_config = output_device.device.default_output_config()?.config();
    let number_of_channels = output_device.channels.total;
    let left_channel_index = output_device.channels.left;
    let right_channel_index = output_device.channels.right;

    log::info!("Starting audio output loop");
    let stream = output_device.device.build_output_stream(
        &default_device_stream_config,
        move |buffer: &mut [f32], _: &cpal::OutputCallbackInfo| {
            let number_of_frames = buffer.len()/number_of_channels as usize;
            let mut samples = if let Ok(samples) = sample_buffer.read_chunk(number_of_frames*2) {
                samples.into_iter()
            } else {
                log::debug!("Audio output buffer dropout");
                return
            };

            for frame in buffer.chunks_mut(number_of_channels as usize){
                frame[left_channel_index] = samples.next().unwrap_or(0.0);
                let right_sample = samples.next().unwrap_or(0.0);
                if let Some(index) = right_channel_index {
                    frame[index] = right_sample;
                }
            }
        },
        |err| {
            log::error!("start_main_audio_output_loop(): Error in audio output stream: {err}");
        },
        None,
    )?;

    stream.play()?;

    log::debug!("start_main_audio_output_loop(): Main audio loop started and playing.");

    Ok(stream)
}

fn default_audio_output_device() -> Result<Device> {
    if let Some(device) = default_host().default_output_device() {
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

fn update_current_output_device_list_if_changed(
    host: &Host,
    current_device_list: &mut Vec<String>,
) -> bool {
    let new_device_list = output_audio_device_name_list(host);

    if *current_device_list != new_device_list {
        *current_device_list = new_device_list;
        return true;
    }

    false
}

fn update_current_output_device_if_changed(
    host: &Host,
    current_device_list: &[String],
    current_output_device: &mut Option<OutputDevice>,
) -> bool {
    if current_device_list.is_empty() {
        return if current_output_device.is_none() {
            false
        } else {
            *current_output_device = None;
            true
        };
    }

    if matches!(current_output_device, Some(output_device) if current_device_list.contains(&output_device.name))
    {
        false
    } else {
        let default_device = current_device_list[DEFAULT_AUDIO_DEVICE_INDEX].clone();
        *current_output_device = output_device_from_name(host, &default_device);

        log::info!("Audio Device List Changed. Using Default Device: {default_device}.");

        true
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
    let device_channels = device.default_output_config()?.channels();

    if device_channels == 0 {
        return Err(anyhow!(AudioError::NoAudioOutputChannels));
    }

    let right = if device_channels > MONO_CHANNELS {
        Some(DEFAULT_RIGHT_CHANNEL_INDEX)
    } else {
        None
    };

    Ok(Channels {
        total: device_channels,
        left: DEFAULT_LEFT_CHANNEL_INDEX,
        right,
    })
}
