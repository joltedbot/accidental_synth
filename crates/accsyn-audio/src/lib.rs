//! CPAL-based audio output with hot-swappable device support for the `AccSyn` synthesizer.
//!
//! Manages audio output streams, device monitoring via `CoreAudio`, and
//! sample delivery from the synthesis engine to the audio hardware.

#![warn(missing_docs)]

mod constants;
mod control_listener;
mod device_monitor;
mod output_loop;

use accsyn_core::audio_events::{AudioDeviceUpdateEvents, OutputStreamParameters};
use accsyn_core::casting::f64_to_u32_clamped;
use accsyn_core::defaults::Defaults;
use accsyn_core::ui_events::UIUpdates;

use crate::constants::{
    AUDIO_MESSAGE_SENDER_CAPACITY, BUFFER_DROP_OUT_LOGGER, SAMPLE_BUFFER_SENDER_CAPACITY,
    STEREO_CHANNEL_COUNT, STEREO_SAMPLE_BUFFER_COUNT,
};
use crate::device_monitor::{DeviceMonitor, get_audio_device_list};
use anyhow::{Result, anyhow};
use coreaudio::audio_unit::macos_helpers::{
    audio_unit_from_device_id_uninitialized, get_default_device_id, get_device_id_from_name,
    get_device_name,
};
use coreaudio::audio_unit::{
    AudioUnit, Element, Scope, StreamFormat, render_callback, render_callback::data,
};
use coreaudio_sys::AudioDeviceID;
use crossbeam_channel::{Receiver, Sender, bounded};
use rtrb::{Producer, RingBuffer};
use std::sync::Arc;
use std::sync::atomic::{AtomicI32, AtomicU16, AtomicU32, Ordering::Relaxed};
use std::thread;
use std::time::Duration;
use thiserror::Error;

/// A named audio output device with its CPAL handle and list index.
#[derive(Clone)]
pub struct OutputDevice {
    /// Human-readable device name.
    pub name: String,
    /// `CoreAudio` device handle for audio output.
    pub device: AudioDeviceID,
    /// Index of this device in the host's output device list.
    pub device_index: i32,
}

/// Atomic left/right channel index pair for the current output device.
pub struct OutputChannels {
    left: AtomicI32,
    right: AtomicI32,
}

/// Errors that can occur during audio device initialization.
#[derive(Debug, Clone, Error)]
pub enum AudioError {
    /// No audio output devices were found on the system.
    #[error("No Audio Output Devices Found")]
    NoAudioOutputDevices,

    /// The selected audio device has no output channels.
    #[error("No Audio Output Channels Found")]
    NoAudioOutputChannels,

    /// No matching physical audio stream format was found for this device
    #[error("No Matching Audio Stream Format Found")]
    NoMatchingAudioStreamFormat,
}

/// Audio device update events that can be sent to the audio subsystem.
type Args = render_callback::Args<data::Interleaved<f32>>;

/// Main audio output manager handling device selection, stream configuration, and sample delivery.
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
    /// Creates a new `Audio` instance with the default output device and configuration.
    ///
    /// # Errors
    ///
    /// Returns an error if it cannot find or get the configuration for the default audio output device
    pub fn new() -> Result<Self> {
        log::debug!(target: "audio", "Constructing Audio Module");
        let (output_device_sender, output_device_receiver) = bounded(SAMPLE_BUFFER_SENDER_CAPACITY);
        let (device_update_sender, device_update_receiver) = bounded(AUDIO_MESSAGE_SENDER_CAPACITY);

        let default_output_device_id =
            get_default_device_id(false).ok_or(AudioError::NoAudioOutputDevices)?;
        let stream_format = stream_format_from_device_id(default_output_device_id)?;
        let channel_count = stream_format.channels;
        let sample_rate = stream_format.sample_rate;

        let output_stream_parameters = OutputStreamParameters {
            sample_rate: Arc::new(AtomicU32::new(f64_to_u32_clamped(sample_rate))),
            buffer_size: Arc::new(AtomicU32::new(
                Defaults::SUPPORTED_BUFFER_SIZES[Defaults::BUFFER_SIZE_INDEX],
            )),
            // channel_count is a CoreAudio u32 that never exceeds u16::MAX in practice
            #[allow(clippy::cast_possible_truncation)]
            channel_count: Arc::new(AtomicU16::new(channel_count as u16)),
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

    /// Returns a clone of the shared output stream parameters.
    #[must_use]
    pub fn get_output_stream_parameters(&self) -> OutputStreamParameters {
        self.output_stream_parameters.clone()
    }

    /// Returns a clone of the sample buffer receiver for the synthesis engine.
    #[must_use]
    pub fn get_sample_buffer_receiver(&self) -> Receiver<Producer<f32>> {
        self.sample_buffer_receiver.clone()
    }

    /// Returns a clone of the device update sender for sending device change events.
    #[must_use]
    pub fn get_device_update_sender(&self) -> Sender<AudioDeviceUpdateEvents> {
        self.device_update_sender.clone()
    }

    /// Starts the audio subsystem: device monitor, dropout logger, and control listener.
    ///
    /// # Errors
    ///
    /// Returns an error if it cannot start the `CoreAudio` device monitor process
    pub fn run(&mut self, ui_update_sender: Sender<UIUpdates>) -> Result<()> {
        log::debug!(target: "audio", "Starting Audio Module");

        log::debug!(target: "audio", "Creating the Audio Device Monitor");
        self.create_core_audio_device_monitor()?;

        log::debug!(target: "audio", "Starting the audio buffer dropout logger");
        start_audio_buffer_dropout_logger(self.buffer_dropout_counter.clone());

        log::debug!(target: "audio", "Starting the Audio Output Device Listener");
        control_listener::start_control_listener(
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

fn send_audio_ui_update(sender: &Sender<UIUpdates>, update: UIUpdates) {
    if let Err(e) = sender.send(update) {
        log::error!(target: "audio::control", "Failed to send UI update: {e}");
    }
}

fn start_audio_buffer_dropout_logger(buffer_dropout_counter: Arc<AtomicU32>) {
    log::debug!(target: "audio", "Starting the audio buffer dropout logger thread");
    thread::spawn(move || {
        loop {
            let buffer_dropout_count = buffer_dropout_counter.load(Relaxed);
            if buffer_dropout_count > 0 {
                log::warn!(
                    target: "audio::control",
                    "The main audio buffer dropped {buffer_dropout_count} times since the last check."
                );
                buffer_dropout_counter.store(0, Relaxed);
            }

            thread::sleep(Duration::from_secs(BUFFER_DROP_OUT_LOGGER));
        }
    });
}

fn default_audio_output_device() -> Option<OutputDevice> {
    if let Some(default_output_device_id) = get_default_device_id(false) {
        let name = get_device_name(default_output_device_id).ok()?;

        log::debug!(
        target: "audio::control",
        "default_audio_output_device(): Using default audio output device: {name}");

        output_device_from_name_and_id(&name, default_output_device_id)
    } else {
        log::error!(target: "audio::control", "default_audio_output_device(): No default audio output device found.");
        None
    }
}

fn new_output_device_from_name(name: &str) -> Option<OutputDevice> {
    log::debug!(target: "audio::control", "new_output_device_from_name(): Looking up device: {name}");
    let is_input_device = false;
    let output_device_id = get_device_id_from_name(name, is_input_device)?;
    output_device_from_name_and_id(name, output_device_id)
}

fn output_device_from_name_and_id(
    name: &str,
    output_device_id: AudioDeviceID,
) -> Option<OutputDevice> {
    log::trace!(target: "audio::control", "output_device_from_name_and_id(): Resolving device '{name}' (id={output_device_id})");
    let output_devices = get_audio_device_list().ok()?;

    output_devices
        .iter()
        .enumerate()
        .find_map(|(index, device_name)| {
            // The size of this value is limited by the number of audio devices that can be connected to the system.
            // This can never exceed i32::MAX
            #[allow(clippy::cast_possible_truncation, clippy::cast_possible_wrap)]
            let device_index = index as i32;

            if device_name == name {
                Some(OutputDevice {
                    name: name.to_string(),
                    device: output_device_id,
                    device_index,
                })
            } else {
                None
            }
        })
}

fn default_channels_from_device(output_device: &OutputDevice) -> Result<(i32, i32)> {
    log::debug!(target: "audio::control", "default_channels_from_device(): Querying channels for '{}'", output_device.name);
    let stream_format = stream_format_from_device_id(output_device.device)?;
    let channel_count = stream_format.channels;

    if channel_count == 0 {
        log::error!(target: "audio::control", "default_channels_from_device(): Device '{}' has no output channels", output_device.name);
        return Err(anyhow!(AudioError::NoAudioOutputChannels));
    }

    let left_index = Defaults::LEFT_CHANNEL_INDEX;

    let right_index = if channel_count > Defaults::MONO_CHANNEL_COUNT {
        Defaults::RIGHT_CHANNEL_INDEX
    } else {
        Defaults::OUTPUT_CHANNEL_DISABLED_VALUE
    };

    log::debug!(target: "audio::control", "default_channels_from_device(): channel_count={channel_count}, left={left_index}, right={right_index}");
    Ok((left_index, right_index))
}

fn stream_format_from_device_id(output_device_id: AudioDeviceID) -> Result<StreamFormat> {
    log::trace!(target: "audio::control", "stream_format_from_device_id(): Querying stream format for device id={output_device_id}");
    let audio_unit = audio_unit_from_device_id_uninitialized(output_device_id, false)?;
    let stream_format = audio_unit.stream_format(Scope::Global, Element::Output)?;
    log::trace!(target: "audio::control", "stream_format_from_device_id(): sample_rate={}, channels={}", stream_format.sample_rate, stream_format.channels);
    Ok(stream_format)
}

fn update_ui_with_new_device_index(
    current_output_device_name: &mut String,
    ui_update_sender: &Sender<UIUpdates>,
    new_output_device_list: &[String],
) {
    let new_device_index = new_output_device_list
        .iter()
        .position(|device_name| device_name == current_output_device_name)
        .unwrap_or(Defaults::AUDIO_DEVICE_INDEX as usize);

    // The size of this value is limited by the number of audio devices that can be connected to the system.
    // This can never exceed i32::MAX
    #[allow(clippy::cast_possible_truncation, clippy::cast_possible_wrap)]
    send_audio_ui_update(
        ui_update_sender,
        UIUpdates::AudioDeviceIndex(new_device_index as i32),
    );

    log::debug!(
        target: "audio::control",
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
    log::debug!(target: "audio::control", "update_device_properties(): device={:?}", new_output_device.map(|d| &d.name));
    update_current_channels(current_output_channels, new_output_device);

    let output_device_id = if let Some(output_device) = new_output_device {
        let output_device_name = output_device.name.clone();
        *current_output_device_name = output_device_name;
        output_device.device
    } else {
        *current_output_device_name = String::new();
        return;
    };

    if let Ok(stream_format) = stream_format_from_device_id(output_device_id) {
        // The size of this value is limited by the number of audio devices that can be connected to the system.
        // This can never exceed u16::MAX
        #[allow(clippy::cast_possible_truncation, clippy::cast_possible_wrap)]
        output_stream_parameters
            .channel_count
            .store(stream_format.channels as u16, Relaxed);
    }
}

fn update_current_channels(
    current_output_channels: &Arc<OutputChannels>,
    new_output_device: Option<&OutputDevice>,
) {
    log::debug!(target: "audio::control", "update_current_channels(): device={:?}", new_output_device.map(|d| &d.name));
    let new_channel_indexes = if let Some(device) = new_output_device {
        default_channels_from_device(device).unwrap_or((
            Defaults::OUTPUT_CHANNEL_DISABLED_VALUE,
            Defaults::OUTPUT_CHANNEL_DISABLED_VALUE,
        ))
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

fn start_main_audio_loop_with_new_device(
    output_steam_parameters: &OutputStreamParameters,
    new_output_device: Option<OutputDevice>,
    current_output_channels: &Arc<OutputChannels>,
    ui_update_sender: &Sender<UIUpdates>,
    sample_producer_sender: &Sender<Producer<f32>>,
    buffer_dropout_counter: &Arc<AtomicU32>,
) -> Option<AudioUnit> {
    if let Some(device) = new_output_device {
        send_audio_ui_update(
            ui_update_sender,
            UIUpdates::AudioDeviceIndex(device.device_index),
        );

        send_audio_ui_update(
            ui_update_sender,
            UIUpdates::AudioDeviceChannelCount(output_steam_parameters.channel_count.load(Relaxed)),
        );

        send_audio_ui_update(
            ui_update_sender,
            UIUpdates::AudioDeviceChannelIndexes {
                left: current_output_channels.left.load(Relaxed),
                right: current_output_channels.right.load(Relaxed),
            },
        );

        log::debug!(
            target: "audio::control",
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
            target: "audio::control",
            "create_control_listener(): Could not find audio output device. Proceeding without audio output."
        );
        None
    }
}

fn restart_main_audio_loop_with_new_device(
    output_steam_parameters: &OutputStreamParameters,
    ui_update_sender: &Sender<UIUpdates>,
    sample_producer_sender: &Sender<Producer<f32>>,
    output_device: &OutputDevice,
    current_output_channels: &Arc<OutputChannels>,
    buffer_dropout_counter: &Arc<AtomicU32>,
) -> Option<AudioUnit> {
    log::debug!(
        target: "audio::control",
        "restart_main_audio_loop_with_new_device(): New Audio Output Device: {:?}",
        output_device.name
    );

    let ring_buffer_capacity = output_steam_parameters.buffer_size.load(Relaxed)
        * STEREO_CHANNEL_COUNT
        * STEREO_SAMPLE_BUFFER_COUNT;
    let (sample_producer, sample_consumer) = RingBuffer::<f32>::new(ring_buffer_capacity as usize);
    if let Err(e) = sample_producer_sender.send(sample_producer) {
        log::error!(target: "audio::control", "Failed to send sample producer: {e}");
        return None;
    }

    send_audio_ui_update(
        ui_update_sender,
        UIUpdates::AudioDeviceIndex(output_device.device_index),
    );

    send_audio_ui_update(
        ui_update_sender,
        UIUpdates::AudioDeviceChannelCount(output_steam_parameters.channel_count.load(Relaxed)),
    );

    send_audio_ui_update(
        ui_update_sender,
        UIUpdates::AudioDeviceChannelIndexes {
            left: current_output_channels.left.load(Relaxed),
            right: current_output_channels.right.load(Relaxed),
        },
    );

    // Indexes are in a fixed range manually configured in a constant array the array will remain a single digit length
    // as this value is determined by what the hardware supports
    #[allow(clippy::cast_possible_truncation, clippy::cast_possible_wrap)]
    send_audio_ui_update(
        ui_update_sender,
        UIUpdates::AudioDeviceSampleRateIndex(output_steam_parameters.sample_rate_index() as i32),
    );

    // Indexes are in a fixed range manually configured in a constant array the array will remain a single digit length
    // as this value is determined by what the hardware supports
    #[allow(clippy::cast_possible_truncation, clippy::cast_possible_wrap)]
    send_audio_ui_update(
        ui_update_sender,
        UIUpdates::AudioDeviceBufferSizeIndex(output_steam_parameters.buffer_size_index() as i32),
    );

    match output_loop::start_main_audio_output_loop(
        output_steam_parameters,
        output_device,
        current_output_channels,
        sample_consumer,
        buffer_dropout_counter.clone(),
    ) {
        Ok(audio_unit) => Some(audio_unit),
        Err(error) => {
            log::error!(target: "audio::control", "Failed to start the main audio output loop. {error:?}");
            None
        }
    }
}
