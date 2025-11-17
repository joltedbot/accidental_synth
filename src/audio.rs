use crate::ui::UIUpdates;
use anyhow::{Result, anyhow};
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use cpal::{Device, Host, Stream, default_host};
use crossbeam_channel::{Receiver, Sender, bounded};
use rtrb::{Consumer, Producer, RingBuffer};
use std::sync::Arc;
use std::sync::atomic::Ordering::Relaxed;
use std::sync::atomic::{AtomicI32, AtomicU16};
use std::thread;
use thiserror::Error;

const DEFAULT_AUDIO_DEVICE_INDEX: usize = 0;
const DEFAULT_LEFT_CHANNEL_INDEX: i32 = 0;
const DEFAULT_RIGHT_CHANNEL_INDEX: i32 = 1;
const SAMPLE_BUFFER_SENDER_CAPACITY: usize = 5;
const AUDIO_MESSAGE_SENDER_CAPACITY: usize = 10;
const SAMPLE_BUFFER_CAPACITY: usize = 8192;
const OUTPUT_CHANNEL_DISABLED_VALUE: i32 = -1;
const USER_CHANNEL_TO_CHANNEL_INDEX_OFFSET: i32 = 1;
const MONO_CHANNEL_COUNT: u16 = 1;

pub struct OutputDevice {
    pub name: String,
    pub channel_count: AtomicU16,
    pub device: Device,
    pub device_index: usize,
}

pub struct OutputChannels {
    left: AtomicI32,
    right: AtomicI32,
}

#[derive(Debug, Clone, Error)]
pub enum AudioError {
    #[error("No Audio Output Devices Found")]
    NoAudioOutputDevices,

    #[error("No Audio Output Channels Found")]
    NoAudioOutputChannels,
}
pub enum AudioDeviceEvent {
    UIOutputDeviceUpdate(String),
    UIOutputDeviceLeftChannelUpdate(i32),
    UIOutputDeviceRightChannelUpdate(i32),
    OutputDeviceUpdate(Option<OutputDevice>),
}

pub struct Audio {
    sample_rate: u32,
    sample_buffer_sender: Sender<Producer<f32>>,
    sample_buffer_receiver: Receiver<Producer<f32>>,
    ui_update_receiver: Receiver<AudioDeviceEvent>,
    device_update_sender: Sender<AudioDeviceEvent>,
    current_output_channels: Arc<OutputChannels>,
}

impl Audio {
    pub fn new() -> Result<Self> {
        log::info!("Constructing Audio Module");
        let (output_device_sender, output_device_receiver) = bounded(SAMPLE_BUFFER_SENDER_CAPACITY);
        let (device_update_sender, ui_update_receiver) = bounded(AUDIO_MESSAGE_SENDER_CAPACITY);

        let default_output_device =
            default_audio_output_device().ok_or(AudioError::NoAudioOutputDevices)?;
        let sample_rate = default_output_device
            .device
            .default_output_config()?
            .sample_rate()
            .0;

        Ok(Self {
            sample_rate,
            sample_buffer_sender: output_device_sender,
            sample_buffer_receiver: output_device_receiver,
            ui_update_receiver,
            device_update_sender,
            current_output_channels: Arc::new(OutputChannels {
                left: AtomicI32::new(DEFAULT_LEFT_CHANNEL_INDEX),
                right: AtomicI32::new(DEFAULT_RIGHT_CHANNEL_INDEX),
            }),
        })
    }

    pub fn get_sample_rate(&self) -> u32 {
        self.sample_rate
    }

    pub fn get_sample_buffer_receiver(&self) -> Receiver<Producer<f32>> {
        self.sample_buffer_receiver.clone()
    }

    pub fn get_device_update_sender(&self) -> Sender<AudioDeviceEvent> {
        self.device_update_sender.clone()
    }

    pub fn run(&mut self, ui_update_sender: Sender<UIUpdates>) {
        create_device_monitor(&ui_update_sender, &self.device_update_sender);
        create_control_listener(
            ui_update_sender,
            self.ui_update_receiver.clone(),
            self.sample_buffer_sender.clone(),
            self.current_output_channels.clone(),
        );
    }
}

fn create_device_monitor(
    ui_update_sender: &Sender<UIUpdates>,
    device_update_sender: &Sender<AudioDeviceEvent>,
) {
    let host = default_host();
    let mut current_output_device_list = Vec::new();
    let mut current_output_device = None;

    // TODO: Add a thread using coreaudio-rs to listen for device updates rather than polling because CoreAudio is weird
    log::debug!("create_device_monitor(): Audio device monitor thread running");

    let is_changed =
        update_current_output_device_list_if_changed(&host, &mut current_output_device_list);

    if is_changed {
        ui_update_sender
            .send(UIUpdates::AudioDeviceList(
                current_output_device_list.clone(),
            ))
            .expect("create_device_monitor(): Could not send audio device list update to the UI. Exiting.");

        update_current_output_device(
            &host,
            &current_output_device_list,
            &mut current_output_device,
        );

        log::debug!("create_device_monitor(): Output device changed");
        device_update_sender
            .send(AudioDeviceEvent::OutputDeviceUpdate(current_output_device))
            .expect(
                "create_device_monitor(): Could not send audio device update to the UI. Exiting.",
            );
    }
}

fn restart_main_audio_loop_with_new_device(
    ui_update_sender: &Sender<UIUpdates>,
    sample_producer_sender: &Sender<Producer<f32>>,
    output_device: OutputDevice,
    current_output_channels: Arc<OutputChannels>,
) -> Option<Stream> {
    log::debug!(
        "restart_main_audio_loop_with_new_device(): New Audio Output Device: {:?}",
        output_device.name
    );

    let (sample_producer, sample_consumer) = RingBuffer::<f32>::new(SAMPLE_BUFFER_CAPACITY);
    sample_producer_sender
        .send(sample_producer)
        .expect("restart_main_audio_loop_with_new_device(): Could not send device update to the audio output device sender. Exiting. ");

    ui_update_sender
        .send(UIUpdates::AudioDeviceIndex(
            output_device.device_index as i32,
        ))
        .expect(
            "restart_main_audio_loop_with_new_device(): Could not send audio device index \
                    update to the UI. Exiting.",
        );

    ui_update_sender
        .send(UIUpdates::AudioDeviceChannelCount(output_device.channel_count.load(Relaxed)))
        .expect(
            "restart_main_audio_loop_with_new_device(): Could not send audio device channel count update to the UI. \
            Exiting.",
        );

    ui_update_sender
        .send(UIUpdates::AudioDeviceChannelIndexes {
            left: current_output_channels.left.load(Relaxed),
            right: current_output_channels.right.load(Relaxed),
        })
        .expect(
            "restart_main_audio_loop_with_new_device(): Could not send audio device channels \
                    update to the UI. Exiting ",
        );

    start_main_audio_output_loop(output_device, current_output_channels, sample_consumer).ok()
}

fn create_control_listener(
    ui_update_sender: Sender<UIUpdates>,
    ui_update_receiver: Receiver<AudioDeviceEvent>,
    sample_producer_sender: Sender<Producer<f32>>,
    current_output_channels: Arc<OutputChannels>,
) {
    thread::spawn(move || {
        let mut audio_output_stream: Option<Stream> = None;

        let ui_update_sender = ui_update_sender.clone();
        let sample_producer_sender = sample_producer_sender.clone();

        log::debug!("create_control_listener(): Audio Device Event listener thread running");
        while let Ok(update) = ui_update_receiver.recv() {
            match update {
                AudioDeviceEvent::UIOutputDeviceUpdate(name) => {
                    if let Some(stream) = audio_output_stream {
                        let _ = stream.pause();
                        drop(stream);
                    }

                    let host = default_host();
                    let mut new_output_device = new_output_device_from_name(&host, &name);

                    if new_output_device.is_none() {
                        log::error!(
                            "create_control_listener(): Could not find audio output device: {name}. Using the default device"
                        );
                        new_output_device = default_audio_output_device();
                    }

                    let new_channel_indexes = if let Some(device) = new_output_device.as_ref() {
                        default_channels_from_device(device).expect("create_control_listener(): Could not get default audio channels for the new output device. Exiting.")
                    } else {
                        (OUTPUT_CHANNEL_DISABLED_VALUE, OUTPUT_CHANNEL_DISABLED_VALUE)
                    };

                    current_output_channels
                        .left
                        .store(new_channel_indexes.0, Relaxed);
                    current_output_channels
                        .right
                        .store(new_channel_indexes.1, Relaxed);

                    audio_output_stream = start_new_output_device(
                        new_output_device,
                        &current_output_channels,
                        &ui_update_sender,
                        &sample_producer_sender,
                    );
                }
                AudioDeviceEvent::UIOutputDeviceLeftChannelUpdate(channel) => {
                    current_output_channels
                        .left
                        .store(channel - USER_CHANNEL_TO_CHANNEL_INDEX_OFFSET, Relaxed);

                    ui_update_sender
                        .send(UIUpdates::AudioDeviceChannelIndexes {
                            left: channel - USER_CHANNEL_TO_CHANNEL_INDEX_OFFSET,
                            right: current_output_channels.right.load(Relaxed),
                        }).expect
                    ("create_device_monitor(): Could not send audio device channel update to the UI. Exiting.");
                }
                AudioDeviceEvent::UIOutputDeviceRightChannelUpdate(channel) => {
                    current_output_channels
                        .right
                        .store(channel - USER_CHANNEL_TO_CHANNEL_INDEX_OFFSET, Relaxed);

                    ui_update_sender
                        .send(UIUpdates::AudioDeviceChannelIndexes {
                            left: current_output_channels.left.load(Relaxed),
                            right: channel - USER_CHANNEL_TO_CHANNEL_INDEX_OFFSET,
                        }).expect
                    ("create_device_monitor(): Could not send audio device channel update to the UI. Exiting.");
                }
                AudioDeviceEvent::OutputDeviceUpdate(mut new_output_device) => {
                    if let Some(stream) = audio_output_stream {
                        let _ = stream.pause();
                        drop(stream);
                    }

                    if new_output_device.is_none() {
                        log::error!(
                            "create_control_listener(): Could not find audio output device Using the default device",
                        );
                        new_output_device = default_audio_output_device();
                    }

                    let new_channel_indexes = if let Some(device) = new_output_device.as_ref() {
                        default_channels_from_device(device).expect("create_control_listener(): Could not get default audio channels for the new output device. Exiting.")
                    } else {
                        (OUTPUT_CHANNEL_DISABLED_VALUE, OUTPUT_CHANNEL_DISABLED_VALUE)
                    };

                    current_output_channels
                        .left
                        .store(new_channel_indexes.0, Relaxed);
                    current_output_channels
                        .right
                        .store(new_channel_indexes.1, Relaxed);

                    audio_output_stream = start_new_output_device(
                        new_output_device,
                        &current_output_channels,
                        &ui_update_sender,
                        &sample_producer_sender,
                    );
                }
            }
        }
    });
}

fn start_new_output_device(
    new_output_device: Option<OutputDevice>,
    current_output_channels: &Arc<OutputChannels>,
    ui_update_sender: &Sender<UIUpdates>,
    sample_producer_sender: &Sender<Producer<f32>>,
) -> Option<Stream> {
    if let Some(device) = new_output_device {
        ui_update_sender
            .send(UIUpdates::AudioDeviceIndex(device.device_index as i32)).expect
        ("create_device_monitor(): Could not send audio device list update to the UI. Exiting.");

        ui_update_sender
            .send(UIUpdates::AudioDeviceChannelCount(device.channel_count.load(Relaxed))).expect
        ("create_device_monitor(): Could not send audio device list update to the UI. Exiting.");

        ui_update_sender
            .send(UIUpdates::AudioDeviceChannelIndexes {
                left: current_output_channels.left.load(Relaxed),
                right: current_output_channels.right.load(Relaxed),
            }).expect
        ("create_device_monitor(): Could not send audio device channel update to the UI. Exiting.");

        restart_main_audio_loop_with_new_device(
            ui_update_sender,
            sample_producer_sender,
            device,
            current_output_channels.clone(),
        )
    } else {
        log::error!(
            "create_control_listener(): Could not find audio output device. Proceeding without audio output."
        );
        None
    }
}

fn start_main_audio_output_loop(
    output_device: OutputDevice,
    output_channels: Arc<OutputChannels>,
    mut sample_buffer: Consumer<f32>,
) -> Result<Stream> {
    let default_device_stream_config = output_device.device.default_output_config()?.config();
    let channel_count = output_device.channel_count;

    log::info!(
        "Starting audio output loop with the device: {}",
        output_device.name
    );
    let stream = output_device.device.build_output_stream(
        &default_device_stream_config,
        move |buffer: &mut [f32], _: &cpal::OutputCallbackInfo| {
            let number_of_channels = channel_count.load(Relaxed) as usize;
            let number_of_frames = buffer.len() / number_of_channels;
            let left_channel_index = output_channels.left.load(Relaxed);
            let right_channel_index = output_channels.right.load(Relaxed);
            let mut samples = if let Ok(samples) = sample_buffer.read_chunk(number_of_frames * 2) {
                samples.into_iter()
            } else {
                log::debug!("Audio output buffer dropout");
                return;
            };

            for frame in buffer.chunks_mut(number_of_channels) {
                frame[left_channel_index as usize] = samples.next().unwrap_or_default();
                let right_sample = samples.next().unwrap_or_default();
                let right_channel = output_channels.right.load(Relaxed);
                if right_channel != OUTPUT_CHANNEL_DISABLED_VALUE {
                    frame[right_channel_index as usize] = right_sample;
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

fn default_audio_output_device() -> Option<OutputDevice> {
    let audio_host = default_host();
    if let Some(device) = audio_host.default_output_device() {
        let device_name = device.name().unwrap_or("Unknown".to_string());
        log::debug!(
            "default_audio_output_device(): Using default audio output device: {device_name}"
        );

        new_output_device_from_name(&audio_host, &device_name)
    } else {
        log::error!("default_audio_output_device(): No default audio output device found.");
        None
    }
}

fn new_output_device_from_name(host: &Host, name: &str) -> Option<OutputDevice> {
    let output_devices = host.output_devices().ok()?;

    output_devices
        .enumerate()
        .find_map(|(device_index, device)| {
            if device.name().ok()? == name {
                let channel_count = device.default_output_config().ok()?.channels();

                if channel_count == 0 {
                    return None;
                }

                Some(OutputDevice {
                    name: name.to_string(),
                    channel_count: AtomicU16::new(channel_count),
                    device,
                    device_index,
                })
            } else {
                None
            }
        })
}

fn default_channels_from_device(output_device: &OutputDevice) -> Result<(i32, i32)> {
    let channel_count = output_device.device.default_output_config()?.channels();

    if channel_count == 0 {
        return Err(anyhow!(AudioError::NoAudioOutputChannels));
    }

    let left_index = DEFAULT_LEFT_CHANNEL_INDEX;

    let right_index = if channel_count > MONO_CHANNEL_COUNT {
        DEFAULT_RIGHT_CHANNEL_INDEX
    } else {
        OUTPUT_CHANNEL_DISABLED_VALUE
    };

    Ok((left_index, right_index))
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

fn update_current_output_device(
    host: &Host,
    current_device_list: &[String],
    current_output_device: &mut Option<OutputDevice>,
) {
    if current_device_list.is_empty() {
        *current_output_device = None;
        return;
    }

    if let Some(output_device) = current_output_device {
        if let Some(index) = current_device_list
            .iter()
            .position(|device| *device == output_device.name)
        {
            output_device.device_index = index;
            return;
        }
    }

    let default_device = current_device_list[DEFAULT_AUDIO_DEVICE_INDEX].clone();
    *current_output_device = new_output_device_from_name(host, &default_device);
    log::info!(
        "Audio Device List Changed. Current device no long available. Using Default Device: {default_device}."
    );
}

fn output_audio_device_name_list(host: &Host) -> Vec<String> {
    let mut device_name_list: Vec<String> = Vec::new();

    if let Ok(device_list) = host.output_devices() {
        for device in device_list {
            if let Ok(name) = device.name() {
                device_name_list.push(name);
            }
        }
    }

    device_name_list
}
