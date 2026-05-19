use crate::sample_convert::SampleConvert;
use crate::{AudioError, OutputChannels, OutputDevice, sample_format_from_supported_bit_depth};
use accsyn_core::audio_events::OutputStreamParameters;
use accsyn_core::defaults::Defaults;
use anyhow::anyhow;
use coreaudio::audio_unit::audio_format::LinearPcmFlags;
use coreaudio::audio_unit::macos_helpers::{
    audio_unit_from_device_id_uninitialized, get_supported_physical_stream_formats,
    set_device_physical_stream_format, set_device_sample_rate,
};
use coreaudio::audio_unit::render_callback::data;
use coreaudio::audio_unit::{
    AudioUnit, Element, SampleFormat, Scope, StreamFormat, render_callback,
};
use coreaudio_sys::kAudioDevicePropertyBufferFrameSize;
use rtrb::Consumer;
use std::sync::Arc;
use std::sync::atomic::AtomicU32;
use std::sync::atomic::Ordering::Relaxed;

fn make_render_callback<T>(
    output_channels: Arc<OutputChannels>,
    mut sample_buffer: Consumer<f32>,
    number_of_channels: u32,
    buffer_dropout_counter: Arc<AtomicU32>,
) -> impl FnMut(render_callback::Args<data::Interleaved<T>>) -> Result<(), ()> + 'static
where
    T: SampleConvert,
    data::Interleaved<T>: data::Data,
{
    move |args: render_callback::Args<data::Interleaved<T>>| {
        let left_channel_index = output_channels.left.load(Relaxed);
        let right_channel_index = output_channels.right.load(Relaxed);

        let mut samples = if let Ok(samples) = sample_buffer.read_chunk(args.num_frames * 2) {
            samples.into_iter()
        } else {
            let _ = buffer_dropout_counter.fetch_add(1, Relaxed);
            return Ok(());
        };

        // Allow usize cast sign loss: channel indices are small positive values (0-7 typical, 0-31 max)
        // Right channel check guards against -1 (disabled), the left channel is never negative
        #[allow(clippy::cast_sign_loss)]
        for frame in args.data.buffer.chunks_mut(number_of_channels as usize) {
            frame[left_channel_index as usize] =
                T::from_f32_sample(samples.next().unwrap_or_default());
            let right_sample = samples.next().unwrap_or_default();
            if right_channel_index != Defaults::OUTPUT_CHANNEL_DISABLED_VALUE {
                frame[right_channel_index as usize] = T::from_f32_sample(right_sample);
            }
        }

        Ok(())
    }
}

pub fn start_main_audio_output_loop(
    output_stream_parameters: &OutputStreamParameters,
    output_device: &OutputDevice,
    output_channels: &Arc<OutputChannels>,
    sample_buffer: Consumer<f32>,
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

    let number_of_channels = output_stream_parameters.channel_count.load(Relaxed);
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

    let supported_formats = get_supported_physical_stream_formats(output_device.id)?;

    log::debug!(target: "audio::loop", "Device reports {} supported physical stream formats", supported_formats.len());

    let physical_format = supported_formats
        .iter()
        .find(|desc| {
            sample_rate >= desc.mSampleRateRange.mMinimum
                && sample_rate <= desc.mSampleRateRange.mMaximum
                && desc.mFormat.mChannelsPerFrame == u32::from(number_of_channels)
        })
        .map(|desc| desc.mFormat)
        .ok_or_else(|| anyhow!(AudioError::NoMatchingAudioStreamFormat))?;

    let supported_bit_depth = physical_format.mBitsPerChannel;
    let supported_flags = physical_format.mFormatFlags;
    let supported_channels = physical_format.mChannelsPerFrame;

    log::debug!(target: "audio::loop", "Setting physical device side stream format: {physical_format:?}");
    set_device_physical_stream_format(output_device.id, physical_format)?;

    let Ok(sample_format) = sample_format_from_supported_bit_depth(supported_bit_depth) else {
        return Err(anyhow!(AudioError::BitDepthNotSupported(
            supported_bit_depth
        )));
    };

    let client_format = StreamFormat {
        sample_rate,
        sample_format,
        flags: LinearPcmFlags::from_bits_truncate(supported_flags),
        channels: supported_channels,
    };

    log::debug!(target: "audio::loop", "Setting audio unit client stream format to: {client_format:?}");
    output_audio_unit.set_stream_format(client_format, Scope::Input, Element::Output)?;

    log::info!(target: "audio::loop", "Starting Render (Output) Callback for Audio Unit");

    match sample_format {
        SampleFormat::F32 => output_audio_unit.set_render_callback(make_render_callback::<f32>(
            output_channels_thread,
            sample_buffer,
            number_of_channels.into(),
            buffer_dropout_counter,
        ))?,
        SampleFormat::I32 => output_audio_unit.set_render_callback(make_render_callback::<i32>(
            output_channels_thread,
            sample_buffer,
            number_of_channels.into(),
            buffer_dropout_counter,
        ))?,
        SampleFormat::I16 => output_audio_unit.set_render_callback(make_render_callback::<i16>(
            output_channels_thread,
            sample_buffer,
            number_of_channels.into(),
            buffer_dropout_counter,
        ))?,
        SampleFormat::I8 => output_audio_unit.set_render_callback(make_render_callback::<i8>(
            output_channels_thread,
            sample_buffer,
            number_of_channels.into(),
            buffer_dropout_counter,
        ))?,
        SampleFormat::I24 => {
            // 24bit integers are actually packed into high-aligned 32bit integers, so i24 will throw an error with
            // the render callback complaining of an invalid sample format.
            return Err(anyhow!(AudioError::BitDepthNotSupported(
                supported_bit_depth
            )));
        }
    }

    log::debug!(target: "audio::loop", "start_main_audio_output_loop(): Initializing the audio unit.");
    output_audio_unit.initialize()?;

    log::debug!(target: "audio::loop", "start_main_audio_output_loop(): Starting the audio unit");
    output_audio_unit.start()?;

    log::info!(target: "audio::loop", "start_main_audio_output_loop(): Main audio loop initialized and started.");

    Ok(output_audio_unit)
}
