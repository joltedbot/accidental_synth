mod constants;
mod device_monitor;

use crate::audio::constants::{
    BUFFER_DROP_OUT_LOGGER, COREAUDIO_DEVICE_LIST_UPDATE_REST_PERIOD_IN_MS,
};
use crate::audio::device_monitor::{DeviceMonitor, get_audio_device_list};
use crate::defaults::Defaults;
use crate::ui::UIUpdates;
use anyhow::{Result, anyhow};
use constants::{
    AUDIO_MESSAGE_SENDER_CAPACITY, SAMPLE_BUFFER_CAPACITY, SAMPLE_BUFFER_SENDER_CAPACITY,
    USER_CHANNEL_TO_CHANNEL_INDEX_OFFSET,
};
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use cpal::{BufferSize, Device, Host, SampleRate, Stream, StreamConfig, default_host};
use crossbeam_channel::{Receiver, Sender, bounded};
use rtrb::{Consumer, Producer, RingBuffer};
use std::sync::Arc;
use std::sync::atomic::Ordering::Relaxed;
use std::sync::atomic::{AtomicI32, AtomicU16, AtomicU32};
use std::thread;
use std::time::Duration;
use thiserror::Error;

#[derive(Clone)]
pub struct OutputDevice {
    pub name: String,
    pub device: Device,
    pub device_index: usize,
}

pub struct OutputChannels {
    left: AtomicI32,
    right: AtomicI32,
}

#[derive(Clone)]
pub struct OutputStreamParameters {
    pub sample_rate: Arc<AtomicU32>,
    pub buffer_size: Arc<AtomicU32>,
    pub channel_count: Arc<AtomicU16>,
}

impl OutputStreamParameters {
    pub fn steam_config(&self) -> StreamConfig {
        let buffer_size_samples = self.buffer_size.load(Relaxed);
        let buffer_size = BufferSize::Fixed(buffer_size_samples);

        StreamConfig {
            sample_rate: SampleRate(self.sample_rate.load(Relaxed)),
            buffer_size,
            channels: self.channel_count.load(Relaxed),
        }
    }

    pub fn sample_rate_index(&self) -> usize {
        Defaults::SUPPORTED_SAMPLE_RATES
            .iter()
            .position(|&x| x == self.sample_rate.load(Relaxed))
            .unwrap_or(Defaults::SAMPLE_RATE_INDEX)
    }

    pub fn buffer_size_index(&self) -> usize {
        Defaults::SUPPORTED_BUFFER_SIZES
            .iter()
            .position(|&x| x == self.buffer_size.load(Relaxed))
            .unwrap_or(Defaults::BUFFER_SIZE_INDEX)
    }
}

#[derive(Debug, Clone, Error)]
pub enum AudioError {
    #[error("No Audio Output Devices Found")]
    NoAudioOutputDevices,

    #[error("No Audio Output Channels Found")]
    NoAudioOutputChannels,
}

#[derive(Debug)]
pub enum AudioDeviceUpdateEvents {
    UIOutputDevice(String),
    UIOutputDeviceLeftChannel(i32),
    UIOutputDeviceRightChannel(i32),
    OutputDeviceListChanged,
    SampleRateChanged(String),
    BufferSizeChanged(String),
}

pub struct Audio {
    output_stream_parameters: OutputStreamParameters,
    sample_buffer_sender: Sender<Producer<f32>>,
    sample_buffer_receiver: Receiver<Producer<f32>>,
    device_update_receiver: Receiver<AudioDeviceUpdateEvents>,
    device_update_sender: Sender<AudioDeviceUpdateEvents>,
    current_output_channels: Arc<OutputChannels>,
    device_monitor: Option<DeviceMonitor>,
    buffer_dropout_counter: Arc<AtomicU32>,
}

impl Audio {
    pub fn new() -> Result<Self> {
        log::debug!(target: "audio", "Constructing Audio Module");
        let (output_device_sender, output_device_receiver) = bounded(SAMPLE_BUFFER_SENDER_CAPACITY);
        let (device_update_sender, device_update_receiver) = bounded(AUDIO_MESSAGE_SENDER_CAPACITY);

        let default_output_device =
            default_audio_output_device().ok_or(AudioError::NoAudioOutputDevices)?;

        let sample_rate = default_output_device
            .device
            .default_output_config()?
            .sample_rate()
            .0;

        let channel_count = default_output_device
            .device
            .default_output_config()?
            .channels();

        let output_stream_parameters = OutputStreamParameters {
            sample_rate: Arc::new(AtomicU32::new(sample_rate)),
            buffer_size: Arc::new(AtomicU32::new(
                Defaults::SUPPORTED_BUFFER_SIZES[Defaults::BUFFER_SIZE_INDEX],
            )),
            channel_count: Arc::new(AtomicU16::new(channel_count)),
        };

        Ok(Self {
            output_stream_parameters,
            sample_buffer_sender: output_device_sender,
            sample_buffer_receiver: output_device_receiver,
            device_update_receiver,
            device_update_sender,
            current_output_channels: Arc::new(OutputChannels {
                left: AtomicI32::new(Defaults::LEFT_CHANNEL_INDEX),
                right: AtomicI32::new(Defaults::RIGHT_CHANNEL_INDEX),
            }),
            device_monitor: None,
            buffer_dropout_counter: Arc::new(AtomicU32::new(0)),
        })
    }

    pub fn get_output_stream_parameters(&self) -> OutputStreamParameters {
        self.output_stream_parameters.clone()
    }

    pub fn get_sample_buffer_receiver(&self) -> Receiver<Producer<f32>> {
        self.sample_buffer_receiver.clone()
    }

    pub fn get_device_update_sender(&self) -> Sender<AudioDeviceUpdateEvents> {
        self.device_update_sender.clone()
    }

    pub fn run(&mut self, ui_update_sender: Sender<UIUpdates>) -> Result<()> {
        log::debug!(target: "audio", "Starting Audio Module");

        log::debug!(target: "audio", "Creating the Audio Device Monitor");

        self.create_core_audio_device_monitor()?;

        log::debug!(target: "audio", "Starting the Audio Output Device Listener");
        start_audio_buffer_dropout_logger(self.buffer_dropout_counter.clone());

        log::debug!(target: "audio", "Starting the Audio Output Device Listener");
        start_control_listener(
            self.output_stream_parameters.clone(),
            ui_update_sender,
            self.device_update_receiver.clone(),
            self.sample_buffer_sender.clone(),
            self.current_output_channels.clone(),
            self.buffer_dropout_counter.clone(),
        );

        Ok(())
    }

    fn create_core_audio_device_monitor(&mut self) -> Result<()> {
        let mut core_audio_device_monitor = DeviceMonitor::new(self.device_update_sender.clone());
        core_audio_device_monitor.run().map_err(|err| {
            anyhow!("run(): macOS Specific CoreAudio Device Monitor failed to start: {err:?}.")
        })?;
        self.device_monitor = Some(core_audio_device_monitor);
        Ok(())
    }
}

fn restart_main_audio_loop_with_new_device(
    output_steam_parameters: OutputStreamParameters,
    ui_update_sender: &Sender<UIUpdates>,
    sample_producer_sender: &Sender<Producer<f32>>,
    output_device: &OutputDevice,
    current_output_channels: &Arc<OutputChannels>,
    buffer_dropout_counter: &Arc<AtomicU32>,
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
        .send(UIUpdates::AudioDeviceChannelCount(output_steam_parameters.channel_count.load(Relaxed)))
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

    ui_update_sender
        .send(UIUpdates::AudioDeviceSampleRateIndex(output_steam_parameters.sample_rate_index() as i32))
        .expect(
            "restart_main_audio_loop_with_new_device(): Could not send audio device sample rate index \
                    update to the UI. Exiting ",
        );

    ui_update_sender
        .send(UIUpdates::AudioDeviceBufferSizeIndex(output_steam_parameters.buffer_size_index() as i32))
        .expect(
            "restart_main_audio_loop_with_new_device(): Could not send audio device buffer size index \
                    update to the UI. Exiting ",
        );

    start_main_audio_output_loop(
        output_steam_parameters,
        output_device,
        current_output_channels,
        sample_consumer,
        buffer_dropout_counter.clone(),
    )
    .ok()
}

#[allow(clippy::too_many_lines)]
fn start_control_listener(
    mut output_stream_parameters: OutputStreamParameters,
    ui_update_sender: Sender<UIUpdates>,
    device_update_receiver: Receiver<AudioDeviceUpdateEvents>,
    sample_producer_sender: Sender<Producer<f32>>,
    current_output_channels: Arc<OutputChannels>,
    buffer_dropout_counter: Arc<AtomicU32>,
) {
    log::debug!(target: "audio::control_listener",  "Starting the Audio Device Event listener thread");
    thread::spawn(move || {
        let mut audio_output_stream: Option<Stream> = None;
        let mut current_output_device_name = String::new();
        let host = default_host();

        let ui_update_sender = ui_update_sender.clone();
        let sample_producer_sender = sample_producer_sender.clone();

        log::debug!(target: "audio::control_listener", "Audio Device Event listener thread running");

        while let Ok(update) = device_update_receiver.recv() {
            log::trace!(target: "audio::control_listener",
                update:? = update;
                "Received UI update");

            match update {
                AudioDeviceUpdateEvents::OutputDeviceListChanged => {
                    log::debug!(
                        "AudioDeviceEvent::OutputDeviceListChanged: The audio device list changed. Updating"
                    );

                    // Sleeping to give USB audio devices time to setting before requesting the new device list to
                    // prevent HAL issues or audio glitches.
                    thread::sleep(Duration::from_millis(
                        COREAUDIO_DEVICE_LIST_UPDATE_REST_PERIOD_IN_MS,
                    ));

                    let new_output_device_list = get_audio_device_list().expect("create_control_listener(): Could not get audio devices from CoreAudio. Exiting.");

                    ui_update_sender
                        .send(UIUpdates::AudioDeviceList(
                            new_output_device_list.clone(),
                        ))
                        .expect("create_device_monitor(): Could not send audio device list update to the UI. Exiting.");

                    if new_output_device_list.contains(&current_output_device_name) {
                        update_ui_with_new_device_index(
                            &mut current_output_device_name,
                            &ui_update_sender,
                            &new_output_device_list,
                        );
                        continue;
                    }

                    log::warn!(
                        "create_control_listener(): The current audio device {current_output_device_name} is not in the new list. {new_output_device_list:?} \
                    Getting the default device."
                    );

                    let new_output_device = default_audio_output_device();

                    update_device_properties(
                        &current_output_channels,
                        &mut current_output_device_name,
                        Option::from(&new_output_device),
                        &mut output_stream_parameters,
                    );

                    if let Some(stream) = audio_output_stream {
                        let _ = stream.pause();
                    }

                    audio_output_stream = start_new_output_device(
                        output_stream_parameters.clone(),
                        new_output_device,
                        &current_output_channels,
                        &ui_update_sender,
                        &sample_producer_sender,
                        &buffer_dropout_counter,
                    );
                }
                AudioDeviceUpdateEvents::UIOutputDevice(name) => {
                    log::debug!(
                        "AudioDeviceEvent::UIOutputDeviceUpdate: Received UI update for audio output device: {name}"
                    );

                    let mut new_output_device = new_output_device_from_name(&host, &name);

                    if new_output_device.is_none() {
                        log::error!(
                            "create_control_listener(): Could not find audio output device: {name}. Using the default device"
                        );
                        new_output_device = default_audio_output_device();
                    }

                    update_device_properties(
                        &current_output_channels,
                        &mut current_output_device_name,
                        Option::from(&new_output_device),
                        &mut output_stream_parameters,
                    );

                    if let Some(stream) = audio_output_stream {
                        let _ = stream.pause();
                    }

                    audio_output_stream = start_new_output_device(
                        output_stream_parameters.clone(),
                        new_output_device,
                        &current_output_channels,
                        &ui_update_sender,
                        &sample_producer_sender,
                        &buffer_dropout_counter,
                    );
                }
                AudioDeviceUpdateEvents::UIOutputDeviceLeftChannel(channel) => {
                    log::debug!(
                        "AudioDeviceEvent::UIOutputDeviceLeftChannelUpdate: Received UI update for audio output device left channel: {channel}"
                    );

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
                AudioDeviceUpdateEvents::UIOutputDeviceRightChannel(channel) => {
                    log::debug!(
                        "AudioDeviceEvent::UIOutputDeviceRightChannelUpdate: Received UI update for audio output device right channel: {channel}"
                    );

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
                AudioDeviceUpdateEvents::SampleRateChanged(sample_rate) => {
                    log::debug!(
                        "AudioDeviceEvent::SampleRateChanged: Received UI update for audio output device sample rate: {sample_rate}"
                    );
                    let numeric_sample_rate = sample_rate
                        .parse::<u32>()
                        .unwrap_or(Defaults::SUPPORTED_SAMPLE_RATES[Defaults::SAMPLE_RATE_INDEX]);
                    output_stream_parameters
                        .sample_rate
                        .store(numeric_sample_rate, Relaxed);

                    let new_output_device =
                        new_output_device_from_name(&host, &current_output_device_name);

                    if let Some(stream) = audio_output_stream {
                        let _ = stream.pause();
                    }

                    audio_output_stream = start_new_output_device(
                        output_stream_parameters.clone(),
                        new_output_device,
                        &current_output_channels,
                        &ui_update_sender,
                        &sample_producer_sender,
                        &buffer_dropout_counter,
                    );
                }
                AudioDeviceUpdateEvents::BufferSizeChanged(buffer_size) => {
                    log::debug!(
                        "AudioDeviceEvent::BufferSizeChanged: Received UI update for audio output device buffer size: {buffer_size}"
                    );

                    let numeric_buffer_size = buffer_size
                        .parse::<u32>()
                        .unwrap_or(Defaults::SUPPORTED_BUFFER_SIZES[Defaults::BUFFER_SIZE_INDEX]);
                    output_stream_parameters
                        .buffer_size
                        .store(numeric_buffer_size, Relaxed);

                    let new_output_device =
                        new_output_device_from_name(&host, &current_output_device_name);

                    if let Some(stream) = audio_output_stream {
                        let _ = stream.pause();
                    }

                    audio_output_stream = start_new_output_device(
                        output_stream_parameters.clone(),
                        new_output_device,
                        &current_output_channels,
                        &ui_update_sender,
                        &sample_producer_sender,
                        &buffer_dropout_counter,
                    );
                }
            }
        }
    });
}

fn start_audio_buffer_dropout_logger(buffer_dropout_counter: Arc<AtomicU32>) {
    log::debug!(target: "audio", "Starting the audio buffer dropout logger thread");
    thread::spawn(move || {
        loop {
            let buffer_dropout_count = buffer_dropout_counter.load(Relaxed);
            if buffer_dropout_count > 0 {
                log::warn!(
                    "The main audio buffer dropped {buffer_dropout_count} times since the last check."
                );
                buffer_dropout_counter.store(0, Relaxed);
            }

            thread::sleep(Duration::from_secs(BUFFER_DROP_OUT_LOGGER));
        }
    });
}

fn update_ui_with_new_device_index(
    current_output_device_name: &mut String,
    ui_update_sender: &Sender<UIUpdates>,
    new_output_device_list: &[String],
) {
    let new_device_index = new_output_device_list
        .iter()
        .position(|device_name| device_name == current_output_device_name)
        .unwrap_or(Defaults::AUDIO_DEVICE_INDEX);

    ui_update_sender
        .send(UIUpdates::AudioDeviceIndex(new_device_index as i32))
        .expect(
            "create_device_monitor(): Could not send audio device index update to the UI. Exiting.",
        );

    log::debug!(
        "AudioDeviceEvent::OutputDeviceListUpdate: The current device is still in the \
                        list. Updating UI with the new index: {new_device_index} and continuing"
    );
}

fn update_device_properties(
    current_output_channels: &Arc<OutputChannels>,
    current_output_device_name: &mut String,
    new_output_device: Option<&OutputDevice>,
    output_stream_parameters: &mut OutputStreamParameters,
) {
    *current_output_device_name = if let Some(device) = new_output_device {
        device.name.clone()
    } else {
        String::new()
    };

    update_current_channels(current_output_channels, new_output_device);

    if let Some(default_config) = new_output_device
        .as_ref()
        .map(|device| device.device.default_output_config())
        .and_then(|config| config.ok())
    {
        output_stream_parameters
            .channel_count
            .store(default_config.channels(), Relaxed);
    }
}

fn update_current_channels(
    current_output_channels: &Arc<OutputChannels>,
    new_output_device: Option<&OutputDevice>,
) {
    let new_channel_indexes = if let Some(device) = new_output_device {
        default_channels_from_device(device).expect("create_control_listener(): Could not get default audio channels for the new output device. Exiting.")
    } else {
        (
            Defaults::OUTPUT_CHANNEL_DISABLED_VALUE,
            Defaults::OUTPUT_CHANNEL_DISABLED_VALUE,
        )
    };

    current_output_channels
        .left
        .store(new_channel_indexes.0, Relaxed);
    current_output_channels
        .right
        .store(new_channel_indexes.1, Relaxed);
}

fn start_new_output_device(
    output_steam_parameters: OutputStreamParameters,
    new_output_device: Option<OutputDevice>,
    current_output_channels: &Arc<OutputChannels>,
    ui_update_sender: &Sender<UIUpdates>,
    sample_producer_sender: &Sender<Producer<f32>>,
    buffer_dropout_counter: &Arc<AtomicU32>,
) -> Option<Stream> {
    if let Some(device) = new_output_device {
        ui_update_sender
            .send(UIUpdates::AudioDeviceIndex(device.device_index as i32)).expect
        ("create_device_monitor(): Could not send audio device list update to the UI. Exiting.");

        ui_update_sender
            .send(UIUpdates::AudioDeviceChannelCount(output_steam_parameters.channel_count.load(Relaxed))).expect
        ("create_device_monitor(): Could not send audio device list update to the UI. Exiting.");

        ui_update_sender
            .send(UIUpdates::AudioDeviceChannelIndexes {
                left: current_output_channels.left.load(Relaxed),
                right: current_output_channels.right.load(Relaxed),
            }).expect
        ("create_device_monitor(): Could not send audio device channel update to the UI. Exiting.");

        log::debug!(
            "start_new_output_device(): Starting audio output loop with the device: {:?}",
            device.name
        );

        restart_main_audio_loop_with_new_device(
            output_steam_parameters,
            ui_update_sender,
            sample_producer_sender,
            &device,
            current_output_channels,
            buffer_dropout_counter,
        )
    } else {
        log::error!(
            "create_control_listener(): Could not find audio output device. Proceeding without audio output."
        );
        None
    }
}

fn start_main_audio_output_loop(
    output_stream_parameters: OutputStreamParameters,
    output_device: &OutputDevice,
    output_channels: &Arc<OutputChannels>,
    mut sample_buffer: Consumer<f32>,
    buffer_dropout_counter: Arc<AtomicU32>,
) -> Result<Stream> {
    let device_stream_config = output_stream_parameters.steam_config();

    log::info!(
        "Starting audio output loop with the device: {}, sample rate: {} Hz, buffer size: {} samples, channels: {}",
        output_device.name,
        output_stream_parameters.sample_rate.load(Relaxed),
        output_stream_parameters.buffer_size.load(Relaxed),
        output_stream_parameters.channel_count.load(Relaxed)
    );

    let number_of_channels = output_stream_parameters.channel_count.load(Relaxed) as usize;
    let output_channels_thread = output_channels.clone();

    let stream = output_device.device.build_output_stream(
        &device_stream_config,
        move |buffer: &mut [f32], _: &cpal::OutputCallbackInfo| {
            let left_channel_index = output_channels_thread.left.load(Relaxed);
            let right_channel_index = output_channels_thread.right.load(Relaxed);
            let number_of_frames = buffer.len() / number_of_channels;
            let mut samples = if let Ok(samples) = sample_buffer.read_chunk(number_of_frames * 2) {
                samples.into_iter()
            } else {
                let _ = buffer_dropout_counter.fetch_add(1, Relaxed);
                return;
            };

            for frame in buffer.chunks_mut(number_of_channels) {
                frame[left_channel_index as usize] = samples.next().unwrap_or_default();
                let right_sample = samples.next().unwrap_or_default();
                if right_channel_index != Defaults::OUTPUT_CHANNEL_DISABLED_VALUE {
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

    let left_index = Defaults::LEFT_CHANNEL_INDEX;

    let right_index = if channel_count > Defaults::MONO_CHANNEL_COUNT {
        Defaults::RIGHT_CHANNEL_INDEX
    } else {
        Defaults::OUTPUT_CHANNEL_DISABLED_VALUE
    };

    Ok((left_index, right_index))
}
