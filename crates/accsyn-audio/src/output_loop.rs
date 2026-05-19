use crate::{AudioError, OutputChannels, OutputDevice, f32_to_int24_aligned_high};
use accsyn_core::audio_events::OutputStreamParameters;
use accsyn_core::defaults::Defaults;
use anyhow::anyhow;
use coreaudio::audio_unit::audio_format::LinearPcmFlags;
use coreaudio::audio_unit::macos_helpers::{audio_unit_from_device_id_uninitialized, get_supported_physical_stream_formats, set_device_physical_stream_format, set_device_sample_rate};
use coreaudio::audio_unit::{render_callback, AudioUnit, Element, SampleFormat, Scope, StreamFormat};
use coreaudio_sys::kAudioDevicePropertyBufferFrameSize;
use rtrb::Consumer;
use std::sync::Arc;
use std::sync::atomic::AtomicU32;
use std::sync::atomic::Ordering::Relaxed;
use coreaudio::audio_unit::render_callback::data;

/// Audio device update events that can be sent to the audio subsystem.
type Args = render_callback::Args<data::Interleaved<i32>>;

pub fn start_main_audio_output_loop(
    output_stream_parameters: &OutputStreamParameters,
    output_device: &OutputDevice,
    output_channels: &Arc<OutputChannels>,
    mut sample_buffer: Consumer<f32>,
    buffer_dropout_counter: Arc<AtomicU32>,
) -> anyhow::Result<AudioUnit> {
    log::info!(
        target: "audio::loop",
        "Setting up audio output loop with the device: {}, sample rate: {} Hz, buffer size: {} samples, channels: {}",
        output_device.name,
        output_stream_parameters.sample_rate.load(Relaxed),
        output_stream_parameters.buffer_size.load(Relaxed),
        output_stream_parameters.channel_count.load(Relaxed)
    );

    let number_of_channels = output_stream_parameters.channel_count.load(Relaxed) as usize;
    let output_channels_thread = output_channels.clone();
    let sample_rate = f64::from(output_stream_parameters.sample_rate.load(Relaxed));
    let buffer_size = output_stream_parameters.buffer_size.load(Relaxed);

    log::debug!(target: "audio::loop", "Setting sample rate to: {sample_rate}");
    set_device_sample_rate(output_device.id, sample_rate)?;

    log::debug!(target: "audio::loop", "Create uninitialized audio unit from device id: {}", output_device.id);
    let mut output_audio_unit = audio_unit_from_device_id_uninitialized(output_device.id, false)?;

    log::debug!(target: "audio::loop", "Setting buffer frame size: {buffer_size}");
    output_audio_unit.set_property(
        kAudioDevicePropertyBufferFrameSize,
        Scope::Output,
        Element::Output,
        Some(&buffer_size),
    )?;

    // I24 high-aligned: signed integer in upper 24 bits of a 32-bit container.
    // SampleFormat::I32 sets mBitsPerChannel=32 which won't match; query device formats directly.
    let required_flags = (LinearPcmFlags::IS_SIGNED_INTEGER | LinearPcmFlags::IS_ALIGNED_HIGH).bits();
    let supported_formats = get_supported_physical_stream_formats(output_device.id)
        .map_err(|e| anyhow!("Failed to query physical stream formats: {e}"))?;

    log::debug!(target: "audio::loop", "Device reports {} supported physical stream formats", supported_formats.len());

    let physical_format = supported_formats
        .iter()
        .find(|desc| {
            (desc.mFormat.mFormatFlags & required_flags) == required_flags
                && sample_rate >= desc.mSampleRateRange.mMinimum
                && sample_rate <= desc.mSampleRateRange.mMaximum
                && desc.mFormat.mChannelsPerFrame == number_of_channels as u32
        })
        .map(|desc| desc.mFormat)
        .ok_or_else(|| anyhow!(AudioError::NoMatchingAudioStreamFormat))?;

    log::debug!(target: "audio::loop", "Setting physical device side stream format: {physical_format:?}");
    set_device_physical_stream_format(output_device.id, physical_format)?;

    // Client format: 32-bit I32 to satisfy data::Interleaved<i32>; AUHAL converts to I24 physical.
    // f32_to_int24_aligned_high packs the 24-bit value into the top 24 bits of i32, so the
    // AUHAL's 8-bit right-shift to I24_HIGH produces the correct sample value.
    let client_format = StreamFormat {
        sample_rate,
        sample_format: SampleFormat::I32,
        flags: LinearPcmFlags::IS_SIGNED_INTEGER | LinearPcmFlags::IS_ALIGNED_HIGH,
        channels: physical_format.mChannelsPerFrame,
    };
    log::debug!(target: "audio::loop", "Setting audio unit client stream format to: {client_format:?}");
    output_audio_unit.set_stream_format(client_format, Scope::Input, Element::Output)?;

    log::info!(target: "audio::loop", "Starting Render (Output) Callback for Audio Unit");
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
            frame[left_channel_index as usize] = f32_to_int24_aligned_high(samples.next().unwrap_or_default());
            let right_sample = samples.next().unwrap_or_default();
            if right_channel_index != Defaults::OUTPUT_CHANNEL_DISABLED_VALUE {
                frame[right_channel_index as usize] = f32_to_int24_aligned_high(right_sample);
            }
        }

        Ok(())
    })?;

    log::debug!(target: "audio::loop", "start_main_audio_output_loop(): Initializing the audio unit.");
    output_audio_unit.initialize()?;

    log::debug!(target: "audio::loop", "start_main_audio_output_loop(): Starting the audio unit");
    output_audio_unit.start()?;

    log::info!(target: "audio::loop", "start_main_audio_output_loop(): Main audio loop initialized and started.");

    Ok(output_audio_unit)
}
