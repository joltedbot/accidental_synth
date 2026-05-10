use crate::constants::OSSSTATUS_NO_ERROR;
use accsyn_core::audio_events::AudioDeviceUpdateEvents;
use anyhow::{Result, anyhow};
use coreaudio_sys::{
    AudioObjectAddPropertyListener, AudioObjectID, AudioObjectPropertyAddress,
    AudioObjectRemovePropertyListener, OSStatus, kAudioHardwarePropertyDevices,
    kAudioObjectPropertyElementMain, kAudioObjectPropertyScopeGlobal, kAudioObjectSystemObject,
};

use coreaudio::audio_unit::Scope;
use coreaudio::audio_unit::macos_helpers::{
    get_audio_device_ids, get_audio_device_supports_scope, get_device_name,
};
use crossbeam_channel::Sender;
use std::ffi::c_void;
use std::ptr;

pub struct DeviceMonitor {
    device_update_sender: Sender<AudioDeviceUpdateEvents>,
    property_address: AudioObjectPropertyAddress,
    client_data_ptr: Option<*mut c_void>,
}

impl DeviceMonitor {
    pub fn new(device_update_sender: Sender<AudioDeviceUpdateEvents>) -> Self {
        log::info!(target: "audio::device", "Constructing CoreAudio device monitor");
        Self {
            device_update_sender,
            property_address: AudioObjectPropertyAddress {
                mSelector: kAudioHardwarePropertyDevices,
                mScope: kAudioObjectPropertyScopeGlobal,
                mElement: kAudioObjectPropertyElementMain,
            },
            client_data_ptr: None,
        }
    }

    pub fn run(&mut self) -> Result<()> {
        log::info!(target: "audio::device", "Starting CoreAudio device monitor");
        let _ = self
            .device_update_sender
            .send(AudioDeviceUpdateEvents::OutputDeviceListChanged);

        unsafe {
            let boxed_device_update_sender = Box::new(self.device_update_sender.clone());
            let client_data = Box::into_raw(boxed_device_update_sender).cast::<c_void>();

            let status = AudioObjectAddPropertyListener(
                kAudioObjectSystemObject,
                &raw const self.property_address,
                Some(device_listener_callback),
                client_data,
            );

            if status != OSSSTATUS_NO_ERROR {
                let _ = Box::from_raw(client_data.cast::<Sender<AudioDeviceUpdateEvents>>());
                return Err(anyhow!("Failed to add property listener: {status}"));
            }

            self.client_data_ptr = Some(client_data);
        }

        Ok(())
    }

    pub fn stop(&mut self) -> Result<(), String> {
        log::info!(target: "audio::device", "Stoping CoreAudio device monitor");
        unsafe {
            let status = AudioObjectRemovePropertyListener(
                kAudioObjectSystemObject,
                &raw const self.property_address,
                Some(device_listener_callback),
                self.client_data_ptr.unwrap_or(ptr::null_mut()),
            );

            if let Some(client_data) = self.client_data_ptr.take() {
                let _ = Box::from_raw(client_data.cast::<Sender<AudioDeviceUpdateEvents>>());
            }

            if status != OSSSTATUS_NO_ERROR {
                return Err(format!("Failed to remove property listener: {status}"));
            }
        }

        Ok(())
    }
}

impl Drop for DeviceMonitor {
    fn drop(&mut self) {
        let _ = self.stop();
    }
}

extern "C" fn device_listener_callback(
    _in_object_id: AudioObjectID,
    _in_number_addresses: u32,
    _in_addresses: *const AudioObjectPropertyAddress,
    in_client_data: *mut c_void,
) -> OSStatus {
    unsafe {
        if in_client_data.is_null() {
            log::trace!(target: "audio::device", "Callback received null client data pointer");
            return 0;
        }

        let device_update_sender = &*(in_client_data as *const Sender<AudioDeviceUpdateEvents>);

        let _ = device_update_sender.send(AudioDeviceUpdateEvents::OutputDeviceListChanged);
    }

    OSSSTATUS_NO_ERROR
}

pub fn get_audio_device_list() -> Result<Vec<String>> {
    log::trace!(target: "audio::device", "Enumerating CoreAudio devices");
    let device_ids = get_audio_device_ids()?;
    Ok(device_ids
        .iter()
        .filter(|&device_id| {
            get_audio_device_supports_scope(*device_id, Scope::Output).unwrap_or(false)
        })
        .filter_map(|device_id| get_device_name(*device_id).ok())
        .collect::<Vec<String>>())
}
