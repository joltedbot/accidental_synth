use crate::{Args, AudioError, OutputChannels, OutputDevice};
use accsyn_core::audio_events::OutputStreamParameters;
use accsyn_core::defaults::Defaults;
use anyhow::anyhow;
use coreaudio::audio_unit::audio_format::LinearPcmFlags;
use coreaudio::audio_unit::macos_helpers::{
    audio_unit_from_device_id_uninitialized, find_matching_physical_format,
    set_device_physical_stream_format, set_device_sample_rate,
};
use coreaudio::audio_unit::{AudioUnit, Element, SampleFormat, Scope, StreamFormat};
use coreaudio_sys::kAudioDevicePropertyBufferFrameSize;
use rtrb::Consumer;
use std::sync::Arc;
use std::sync::atomic::AtomicU32;
use std::sync::atomic::Ordering::Relaxed;

pub fn start_main_audio_output_loop(
    output_stream_parameters: &OutputStreamParameters,
    output_device: &OutputDevice,
    output_channels: &Arc<OutputChannels>,
    mut sample_buffer: Consumer<f32>,
    buffer_dropout_counter: Arc<AtomicU32>,
) -> anyhow::Result<AudioUnit> {
    log::info!(
        target: "audio::control",
        "Setting up audio output loop with the device: {}, sample rate: {} Hz, buffer size: {} samples, channels: {}",
        output_device.name,
        output_stream_parameters.sample_rate.load(Relaxed),
        output_stream_parameters.buffer_size.load(Relaxed),
        output_stream_parameters.channel_count.load(Relaxed)
    );

    let device_stream_format = crate::stream_format_from_device_id(output_device.device)?;
    let number_of_channels = output_stream_parameters.channel_count.load(Relaxed) as usize;
    let output_channels_thread = output_channels.clone();

    set_device_sample_rate(
        output_device.device,
        f64::from(output_stream_parameters.sample_rate.load(Relaxed)),
    )?;

    let mut output_audio_unit =
        audio_unit_from_device_id_uninitialized(output_device.device, false)?;

    output_audio_unit.set_property(
        kAudioDevicePropertyBufferFrameSize,
        Scope::Output,
        Element::Output,
        Some(&output_stream_parameters.buffer_size.load(Relaxed)),
    )?;

    let output_stream_format = StreamFormat {
        sample_rate: device_stream_format.sample_rate,
        sample_format: SampleFormat::F32,
        flags: LinearPcmFlags::IS_FLOAT | LinearPcmFlags::IS_PACKED,
        channels: device_stream_format.channels,
    };

    let new_stream_format =
        find_matching_physical_format(output_device.device, output_stream_format)
            .ok_or(anyhow!(AudioError::NoMatchingAudioStreamFormat))?;
    set_device_physical_stream_format(output_device.device, new_stream_format)?;

    log::debug!(
        target: "audio::control",
        "Using render (output) stream format: {output_stream_format:?}",
    );
    output_audio_unit.set_stream_format(output_stream_format, Scope::Input, Element::Output)?;

    log::info!(target: "audio::control", "Starting Render (Output) Callback for Audio Unit");
    output_audio_unit.set_render_callback(move |args: Args| {
        let left_channel_index = output_channels_thread.left.load(Relaxed);
        let right_channel_index = output_channels_thread.right.load(Relaxed);

        let mut samples = if let Ok(samples) = sample_buffer.read_chunk(args.num_frames * 2) {
            samples.into_iter()
        } else {
            let _ = buffer_dropout_counter.fetch_add(1, Relaxed);
            return Ok(());
        };

        // Allow usize cast sign loss: channel indices are small positive values (0-7 typical, 0-31 max)
        // Right channel check guards against -1 (disabled), the left channel is never negative
        #[allow(clippy::cast_sign_loss)]
        for frame in args.data.buffer.chunks_mut(number_of_channels) {
            frame[left_channel_index as usize] = samples.next().unwrap_or_default();
            let right_sample = samples.next().unwrap_or_default();
            if right_channel_index != Defaults::OUTPUT_CHANNEL_DISABLED_VALUE {
                frame[right_channel_index as usize] = right_sample;
            }
        }

        Ok(())
    })?;

    log::debug!(target: "audio::control", "start_main_audio_output_loop(): Initializing the audio unit.");
    output_audio_unit.initialize()?;

    log::debug!(target: "audio::control", "start_main_audio_output_loop(): Starting the audio unit");
    output_audio_unit.start()?;

    log::info!(target: "audio::control", "start_main_audio_output_loop(): Main audio loop initialized and started.");

    Ok(output_audio_unit)
}
