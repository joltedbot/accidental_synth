use crate::defaults::Defaults;
use std::sync::Arc;
use std::sync::atomic::Ordering::Relaxed;
use std::sync::atomic::{AtomicU16, AtomicU32};

/// Shared audio output stream configuration parameters.
#[derive(Clone)]
pub struct OutputStreamParameters {
    /// Current sample rate in Hz.
    pub sample_rate: Arc<AtomicU32>,
    /// Current audio buffer size in samples.
    pub buffer_size: Arc<AtomicU32>,
    /// Number of output channels on the current device.
    pub channel_count: Arc<AtomicU16>,
}

impl OutputStreamParameters {
    /// Returns the index of the current sample rate in [`Defaults::SUPPORTED_SAMPLE_RATES`].
    pub fn sample_rate_index(&self) -> usize {
        Defaults::SUPPORTED_SAMPLE_RATES
            .iter()
            .position(|&x| x == self.sample_rate.load(Relaxed))
            .unwrap_or(Defaults::SAMPLE_RATE_INDEX)
    }

    /// Returns the index of the current buffer size in [`Defaults::SUPPORTED_BUFFER_SIZES`].
    pub fn buffer_size_index(&self) -> usize {
        Defaults::SUPPORTED_BUFFER_SIZES
            .iter()
            .position(|&x| x == self.buffer_size.load(Relaxed))
            .unwrap_or(Defaults::BUFFER_SIZE_INDEX)
    }
}

/// Events for updating audio device configuration.
#[derive(Debug)]
pub enum AudioDeviceUpdateEvents {
    /// User selected a new output device by name.
    UIOutputDevice(String),
    /// User changed the left output channel index.
    UIOutputDeviceLeftChannel(i32),
    /// User changed the right output channel index.
    UIOutputDeviceRightChannel(i32),
    /// The system audio device list changed (device connected/disconnected).
    OutputDeviceListChanged,
    /// User changed the sample rate (value as string for UI parsing).
    SampleRateChanged(String),
    /// User changed the buffer size (value as string for UI parsing).
    BufferSizeChanged(String),
}
