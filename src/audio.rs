use anyhow::{Result, anyhow};
use cpal::traits::{DeviceTrait, HostTrait};
use cpal::{Device, default_host};
use thiserror::Error;

#[derive(Debug, Clone, Error, PartialEq, Eq)]
pub enum AudioError {
    #[error("No Audio Output Devices Found")]
    NoAudioOutputDevices,
}

pub struct Audio {
    sample_rate: u32,
    default_output_device: Device,
}

impl Audio {
    pub fn new() -> Result<Self> {
        log::info!("Constructing Audio Module");
        let default_output_device = default_audio_output_device()?;
        let sample_rate = default_output_device
            .default_output_config()?
            .sample_rate()
            .0;

        Ok(Self {
            sample_rate,
            default_output_device,
        })
    }

    pub fn default_output_device(&self) -> Device {
        self.default_output_device.clone()
    }

    pub fn sample_rate(&self) -> u32 {
        self.sample_rate
    }
}

fn default_audio_output_device() -> Result<Device> {
    let default_output_device = default_host().default_output_device();
    if let Some(device) = default_output_device {
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_returns_audio_with_correct_default_values() {
        let audio = Audio::new().unwrap();

        assert_ne!(audio.sample_rate, 0);
        assert_ne!(
            audio
                .default_output_device
                .name()
                .unwrap_or("Unknown".to_string()),
            "Unknown"
        );
    }

    #[test]
    fn default_output_device_returns_default_audio_output_device() {
        if let Err(err) = default_audio_output_device() {
            panic!("default_output_device(): {err}");
        }
    }
}
