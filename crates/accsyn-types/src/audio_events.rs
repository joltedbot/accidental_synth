use crate::defaults::Defaults;
use std::sync::Arc;
use std::sync::atomic::Ordering::Relaxed;
use std::sync::atomic::{AtomicU16, AtomicU32};

#[derive(Clone)]
pub struct OutputStreamParameters {
    pub sample_rate: Arc<AtomicU32>,
    pub buffer_size: Arc<AtomicU32>,
    pub channel_count: Arc<AtomicU16>,
}

impl OutputStreamParameters {
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

#[derive(Debug)]
pub enum AudioDeviceUpdateEvents {
    UIOutputDevice(String),
    UIOutputDeviceLeftChannel(i32),
    UIOutputDeviceRightChannel(i32),
    OutputDeviceListChanged,
    SampleRateChanged(String),
    BufferSizeChanged(String),
}
