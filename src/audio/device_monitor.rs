use crate::audio::constants::OSSSTATUS_NO_ERROR;

#[cfg(not(target_os = "macos"))]
use {
    crate::audio::constants::DEVICE_LIST_POLLING_INTERVAL_IN_MS,
    crate::audio::update_current_output_device_list_if_changed, cpal::default_host,
    crossbeam_channel::Sender, std::ffi::c_void, std::thread, std::thread::sleep,
    std::time::Duration,
};

use crate::audio::AudioDeviceUpdateEvents;
use anyhow::{Result, anyhow};
use core_foundation::base::TCFType;
use core_foundation::string::{CFString, CFStringRef};
use coreaudio_sys::{
    AudioDeviceID, AudioObjectAddPropertyListener, AudioObjectGetPropertyData,
    AudioObjectGetPropertyDataSize, AudioObjectID, AudioObjectPropertyAddress,
    AudioObjectRemovePropertyListener, OSStatus, UInt32, kAudioDevicePropertyDeviceNameCFString,
    kAudioDevicePropertyStreamConfiguration, kAudioHardwarePropertyDevices,
    kAudioObjectPropertyElementMain, kAudioObjectPropertyElementMaster,
    kAudioObjectPropertyScopeGlobal, kAudioObjectPropertyScopeOutput, kAudioObjectSystemObject,
};

use crossbeam_channel::Sender;
use std::ffi::c_void;
use std::ptr;

#[cfg(target_os = "macos")]
pub struct DeviceMonitor {
    device_update_sender: Sender<AudioDeviceUpdateEvents>,
    property_address: AudioObjectPropertyAddress,
    client_data_ptr: Option<*mut c_void>,
}

impl DeviceMonitor {
    pub fn new(device_update_sender: Sender<AudioDeviceUpdateEvents>) -> Self {
        log::info!("Constructing CoreAudio device monitor");
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
        log::debug!("run(): Starting CoreAudio device monitor");
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

pub fn get_audio_device_list() -> Result<Vec<String>, String> {
    log::debug!("Getting new Coreaudio device list.");
    unsafe {
        let property_address = AudioObjectPropertyAddress {
            mSelector: kAudioHardwarePropertyDevices,
            mScope: kAudioObjectPropertyScopeGlobal,
            mElement: kAudioObjectPropertyElementMain,
        };

        let mut property_size: UInt32 = 0;
        let status = AudioObjectGetPropertyDataSize(
            kAudioObjectSystemObject,
            &raw const property_address,
            0,
            ptr::null(),
            &raw mut property_size,
        );

        if status != OSSSTATUS_NO_ERROR {
            return Err(format!("Failed to get the device list size: {status}"));
        }

        // Get the device IDs
        let device_count = property_size as usize / size_of::<AudioDeviceID>();
        let mut devices = vec![0u32; device_count];

        let status = AudioObjectGetPropertyData(
            kAudioObjectSystemObject,
            &raw const property_address,
            0,
            ptr::null(),
            &raw mut property_size,
            devices.as_mut_ptr().cast(),
        );

        if status != OSSSTATUS_NO_ERROR {
            return Err(format!("Failed to get the device list: {status}"));
        }

        let mut result = Vec::new();
        for &device_id in &devices {
            if is_output_device(device_id) {
                match get_device_name(device_id) {
                    Ok(name) => {
                        result.push(name);
                    }
                    Err(err) => {
                        log::warn!(
                            "Couldn't get the name for the audio output device {device_id} from CoreAudio. Error: \
                            {err}",
                        );
                    }
                }
            }
        }

        Ok(result)
    }
}

unsafe fn is_output_device(device_id: AudioDeviceID) -> bool {
    unsafe {
        let property_address = AudioObjectPropertyAddress {
            mSelector: kAudioDevicePropertyStreamConfiguration,
            mScope: kAudioObjectPropertyScopeOutput,
            mElement: kAudioObjectPropertyElementMaster,
        };

        let mut property_size: UInt32 = 0;
        let status = AudioObjectGetPropertyDataSize(
            device_id,
            &raw const property_address,
            0,
            ptr::null(),
            &raw mut property_size,
        );

        if status != OSSSTATUS_NO_ERROR || property_size < size_of::<UInt32>() as UInt32 {
            log::debug!(
                "is_output_device(): Failed to get the device stream configuration size for the device {device_id}. Status: {status}, Size: {property_size}"
            );
            return false;
        }

        let mut buffer = vec![0u8; property_size as usize];
        let mut property_data_size = property_size;

        let status = AudioObjectGetPropertyData(
            device_id,
            &raw const property_address,
            0,
            ptr::null(),
            &raw mut property_data_size,
            buffer.as_mut_ptr().cast::<c_void>(),
        );

        if status != OSSSTATUS_NO_ERROR {
            log::debug!(
                "is_output_device(): Failed to get the device stream configuration for the device {device_id}. Status: {status}"
            );
            return false;
        }

        let num_buffers = if buffer.len() >= 4 {
            u32::from_ne_bytes([buffer[0], buffer[1], buffer[2], buffer[3]])
        } else {
            log::warn!(
                "is_output_device(): Returned buffer size is less than 4 bytes. Returning 0 and continuing."
            );
            0
        };

        num_buffers > 0
    }
}

unsafe fn get_device_name(device_id: AudioDeviceID) -> Result<String, String> {
    let property_address = AudioObjectPropertyAddress {
        mSelector: kAudioDevicePropertyDeviceNameCFString,
        mScope: kAudioObjectPropertyScopeGlobal,
        mElement: kAudioObjectPropertyElementMain,
    };

    let mut property_size = size_of::<CFStringRef>() as u32;
    let mut cf_string: CFStringRef = ptr::null();

    let status = unsafe {
        AudioObjectGetPropertyData(
            device_id,
            &raw const property_address,
            0,
            ptr::null(),
            &raw mut property_size,
            (&raw mut cf_string).cast(),
        )
    };

    if status != OSSSTATUS_NO_ERROR {
        return Err(format!("Failed to get device name: {status}"));
    }

    if cf_string.is_null() {
        return Err("Device name is null".to_string());
    }

    let cf_string_wrapper = unsafe { CFString::wrap_under_create_rule(cf_string) };
    Ok(cf_string_wrapper.to_string())
}

extern "C" fn device_listener_callback(
    _in_object_id: AudioObjectID,
    _in_number_addresses: u32,
    _in_addresses: *const AudioObjectPropertyAddress,
    in_client_data: *mut c_void,
) -> OSStatus {
    unsafe {
        if in_client_data.is_null() {
            log::debug!("device_listener_callback(): Received a null client data pointer.");
            return 0;
        }

        let device_update_sender = &*(in_client_data as *const Sender<AudioDeviceUpdateEvents>);

        device_update_sender.send(AudioDeviceUpdateEvents::OutputDeviceListChanged).expect("device_listener_callback(): Failed to send audio device update to the UI. Exiting.");
    }

    OSSSTATUS_NO_ERROR
}

#[cfg(not(target_os = "macos"))]
pub fn create_device_monitor(device_update_sender: Sender<AudioDeviceUpdateEvents>) {
    let host = default_host();
    let mut current_output_device_list = Vec::new();

    thread::spawn(move || {
        log::debug!("run(): Audio device monitor thread running");
        loop {
            let is_changed = update_current_output_device_list_if_changed(
                &host,
                &mut current_output_device_list,
            );

            if is_changed {
                log::debug!("create_device_monitor(): Output device list changed");
                device_update_sender
                    .send(AudioDeviceUpdateEvents::OutputDeviceList(current_output_device_list.clone()))
                    .expect(
                        "create_device_monitor(): Could not send audio device update to the UI. Exiting.",
                    );
            }

            sleep(Duration::from_millis(DEVICE_LIST_POLLING_INTERVAL_IN_MS));
        }
    });
}
