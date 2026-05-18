use crate::constants::{
    COREAUDIO_DEVICE_LIST_UPDATE_REST_PERIOD_IN_MS, USER_CHANNEL_TO_CHANNEL_INDEX_OFFSET,
};
use crate::device_monitor::get_audio_device_list;
use crate::{OutputChannels, start_main_audio_loop_with_new_device};
use accsyn_core::audio_events::{AudioDeviceUpdateEvents, OutputStreamParameters};
use accsyn_core::defaults::Defaults;
use accsyn_core::ui_events::UIUpdates;
use coreaudio::audio_unit::AudioUnit;
use crossbeam_channel::{Receiver, Sender};
use rtrb::Producer;
use std::sync::Arc;
use std::sync::atomic::AtomicU32;
use std::sync::atomic::Ordering::Relaxed;
use std::thread;
use std::time::Duration;

#[allow(clippy::too_many_lines)]
pub fn start_control_listener(
    mut output_stream_parameters: OutputStreamParameters,
    ui_update_sender: Sender<UIUpdates>,
    device_update_receiver: Receiver<AudioDeviceUpdateEvents>,
    sample_producer_sender: Sender<Producer<f32>>,
    current_output_channels: Arc<OutputChannels>,
    buffer_dropout_counter: Arc<AtomicU32>,
) {
    log::debug!(target: "audio::control_listener",  "Starting the Audio Device Event listener thread");
    thread::spawn(move || {
        let mut audio_output_stream: Option<AudioUnit> = None;
        let mut current_output_device_name = String::new();

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
                        target: "audio::control",
                        "AudioDeviceEvent::OutputDeviceListChanged: The audio device list changed. Updating"
                    );

                    // Sleeping to give USB audio devices time to setting before requesting the new device list to
                    // prevent HAL issues or audio glitches.
                    thread::sleep(Duration::from_millis(
                        COREAUDIO_DEVICE_LIST_UPDATE_REST_PERIOD_IN_MS,
                    ));

                    let new_output_device_list = match get_audio_device_list() {
                        Ok(list) => list,
                        Err(e) => {
                            log::error!(target: "audio::control", "Failed to get audio device list from CoreAudio: {e}");
                            continue;
                        }
                    };

                    crate::send_audio_ui_update(
                        &ui_update_sender,
                        UIUpdates::AudioDeviceList(new_output_device_list.clone()),
                    );

                    if new_output_device_list.contains(&current_output_device_name) {
                        crate::update_ui_with_new_device_index(
                            &mut current_output_device_name,
                            &ui_update_sender,
                            &new_output_device_list,
                        );
                        continue;
                    }

                    log::warn!(
                        target: "audio::control",
                        "create_control_listener(): The current audio device {current_output_device_name} is not in the new list. {new_output_device_list:?} \
                    Getting the default device."
                    );

                    let new_output_device = crate::default_audio_output_device();

                    crate::update_device_properties(
                        &current_output_channels,
                        &mut current_output_device_name,
                        Option::from(&new_output_device),
                        &mut output_stream_parameters,
                    );

                    if let Some(mut audio_unit) = audio_output_stream {
                        let _ = audio_unit.stop();
                    }

                    audio_output_stream = start_main_audio_loop_with_new_device(
                        &output_stream_parameters.clone(),
                        new_output_device,
                        &current_output_channels,
                        &ui_update_sender,
                        &sample_producer_sender,
                        &buffer_dropout_counter,
                    );
                }
                AudioDeviceUpdateEvents::UIOutputDevice(name) => {
                    log::debug!(
                        target: "audio::control",
                        "AudioDeviceEvent::UIOutputDeviceUpdate: Received UI update for audio output device: {name}"
                    );

                    let mut new_output_device = crate::new_output_device_from_name(&name);

                    if new_output_device.is_none() {
                        log::error!(
                            target: "audio::control",
                            "create_control_listener(): Could not find audio output device: {name}. Using the default device"
                        );
                        new_output_device = crate::default_audio_output_device();
                    }

                    crate::update_device_properties(
                        &current_output_channels,
                        &mut current_output_device_name,
                        Option::from(&new_output_device),
                        &mut output_stream_parameters,
                    );

                    if let Some(mut audio_unit) = audio_output_stream {
                        let _ = audio_unit.stop();
                    }

                    audio_output_stream = start_main_audio_loop_with_new_device(
                        &output_stream_parameters.clone(),
                        new_output_device,
                        &current_output_channels,
                        &ui_update_sender,
                        &sample_producer_sender,
                        &buffer_dropout_counter,
                    );
                }
                AudioDeviceUpdateEvents::UIOutputDeviceLeftChannel(channel) => {
                    log::debug!(
                        target: "audio::control",
                        "AudioDeviceEvent::UIOutputDeviceLeftChannelUpdate: Received UI update for audio output device left channel: {channel}"
                    );

                    current_output_channels
                        .left
                        .store(channel - USER_CHANNEL_TO_CHANNEL_INDEX_OFFSET, Relaxed);

                    crate::send_audio_ui_update(
                        &ui_update_sender,
                        UIUpdates::AudioDeviceChannelIndexes {
                            left: channel - USER_CHANNEL_TO_CHANNEL_INDEX_OFFSET,
                            right: current_output_channels.right.load(Relaxed),
                        },
                    );
                }
                AudioDeviceUpdateEvents::UIOutputDeviceRightChannel(channel) => {
                    log::debug!(
                        target: "audio::control",
                        "AudioDeviceEvent::UIOutputDeviceRightChannelUpdate: Received UI update for audio output device right channel: {channel}"
                    );

                    current_output_channels
                        .right
                        .store(channel - USER_CHANNEL_TO_CHANNEL_INDEX_OFFSET, Relaxed);

                    crate::send_audio_ui_update(
                        &ui_update_sender,
                        UIUpdates::AudioDeviceChannelIndexes {
                            left: current_output_channels.left.load(Relaxed),
                            right: channel - USER_CHANNEL_TO_CHANNEL_INDEX_OFFSET,
                        },
                    );
                }
                AudioDeviceUpdateEvents::SampleRateChanged(sample_rate) => {
                    log::debug!(
                        target: "audio::control",
                        "AudioDeviceEvent::SampleRateChanged: Received UI update for audio output device sample rate: {sample_rate}"
                    );
                    let numeric_sample_rate = sample_rate
                        .parse::<u32>()
                        .unwrap_or(Defaults::SUPPORTED_SAMPLE_RATES[Defaults::SAMPLE_RATE_INDEX]);
                    output_stream_parameters
                        .sample_rate
                        .store(numeric_sample_rate, Relaxed);

                    let new_output_device =
                        crate::new_output_device_from_name(&current_output_device_name);

                    if let Some(mut audio_unit) = audio_output_stream {
                        let _ = audio_unit.stop();
                    }

                    audio_output_stream = start_main_audio_loop_with_new_device(
                        &output_stream_parameters.clone(),
                        new_output_device,
                        &current_output_channels,
                        &ui_update_sender,
                        &sample_producer_sender,
                        &buffer_dropout_counter,
                    );
                }
                AudioDeviceUpdateEvents::BufferSizeChanged(buffer_size) => {
                    log::debug!(
                        target: "audio::control",
                        "AudioDeviceEvent::BufferSizeChanged: Received UI update for audio output device buffer size: {buffer_size}"
                    );

                    let numeric_buffer_size = buffer_size
                        .parse::<u32>()
                        .unwrap_or(Defaults::SUPPORTED_BUFFER_SIZES[Defaults::BUFFER_SIZE_INDEX]);
                    output_stream_parameters
                        .buffer_size
                        .store(numeric_buffer_size, Relaxed);

                    let new_output_device =
                        crate::new_output_device_from_name(&current_output_device_name);

                    if let Some(mut audio_unit) = audio_output_stream {
                        let _ = audio_unit.stop();
                    }

                    audio_output_stream = start_main_audio_loop_with_new_device(
                        &output_stream_parameters.clone(),
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
