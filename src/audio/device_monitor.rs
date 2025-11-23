use crate::audio::constants::DEVICE_LIST_POLLING_INTERVAL_IN_MS;
use crate::audio::{AudioDeviceUpdateEvents, update_current_output_device_list_if_changed};
use anyhow::{Result, anyhow};
use core_foundation::base::TCFType;
use core_foundation::string::{CFString, CFStringRef};
use coreaudio_sys::{AudioDeviceID, AudioObjectAddPropertyListener, AudioObjectGetPropertyData, AudioObjectGetPropertyDataSize, AudioObjectID, AudioObjectPropertyAddress, AudioObjectRemovePropertyListener, OSStatus, UInt32, kAudioDevicePropertyDeviceNameCFString, kAudioHardwarePropertyDevices, kAudioObjectPropertyElementMain, kAudioObjectPropertyScopeGlobal, kAudioObjectSystemObject, kAudioDevicePropertyStreamConfiguration, kAudioObjectPropertyScopeOutput, kAudioObjectPropertyElementMaster};
use cpal::default_host;
use crossbeam_channel::Sender;
use std::ffi::c_void;
use std::thread::sleep;
use std::time::Duration;
use std::{ptr, thread};

const AUDIO_OBJECT_SYSTEM_OBJECT: AudioObjectID = 1;

#[cfg(target_os = "macos")]
pub struct DeviceMonitor {
    device_update_sender: Sender<AudioDeviceUpdateEvents>,
    property_address: AudioObjectPropertyAddress,
    user_data_ptr: Option<*mut c_void>,
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
            user_data_ptr: None,
        }
    }
    pub fn run(&mut self) -> Result<()> {
        log::debug!("run(): Starting CoreAudio device monitor");
        let _ = self
            .device_update_sender
            .send(AudioDeviceUpdateEvents::OutputDeviceListChanged);

        unsafe {
            let boxed_device_update_sender = Box::new(self.device_update_sender.clone());
            let user_data = Box::into_raw(boxed_device_update_sender) as *mut c_void;

            let status = AudioObjectAddPropertyListener(
                kAudioObjectSystemObject,
                &self.property_address as *const _,
                Some(device_listener_callback),
                user_data,
            );

            if status != 0 {
                // Cleanup on failure
                let _ = Box::from_raw(user_data as *mut Sender<AudioDeviceUpdateEvents>);
                return Err(anyhow!("Failed to add property listener: {status}"));
            }

            // Store pointer for cleanup later
            self.user_data_ptr = Some(user_data);
        }

        Ok(())
    }

    pub fn stop(&mut self) -> Result<(), String> {
        unsafe {
            let status = AudioObjectRemovePropertyListener(
                kAudioObjectSystemObject,
                &self.property_address as *const _,
                Some(device_listener_callback),
                self.user_data_ptr.unwrap_or(ptr::null_mut()),
            );

            if let Some(user_data) = self.user_data_ptr.take() {
                let _ = Box::from_raw(user_data as *mut Sender<AudioDeviceUpdateEvents>);
            }

            if status != 0 {
                return Err(format!("Failed to remove property listener: {}", status));
            }
        }

        Ok(())
    }
}

pub fn get_audio_devices() -> Result<Vec<String>, String> {
    unsafe {
        let property_address = AudioObjectPropertyAddress {
            mSelector: kAudioHardwarePropertyDevices,
            mScope: kAudioObjectPropertyScopeGlobal,
            mElement: kAudioObjectPropertyElementMain,
        };

        let mut property_size: UInt32 = 0;
        let status = AudioObjectGetPropertyDataSize(
            kAudioObjectSystemObject,
            &property_address,
            0,
            ptr::null(),
            &mut property_size,
        );

        if status != 0 {
            return Err(format!("Failed to get the device list size: {}", status));
        }

        // Get the device IDs
        let device_count = property_size as usize / size_of::<AudioDeviceID>();
        let mut devices = vec![0u32; device_count];

        let status = AudioObjectGetPropertyData(
            AUDIO_OBJECT_SYSTEM_OBJECT,
            &property_address as *const _,
            0,
            ptr::null(),
            &mut property_size as *mut _,
            devices.as_mut_ptr() as *mut _,
        );

        if status != 0 {
            return Err(format!("Failed to get the device list: {status}"));
        }

        // Get names for each device
        let mut result = Vec::new();
        for &device_id in &devices {
            if is_output_device(device_id) {
                match get_device_name(device_id) {
                    Ok(name) => {
                        println!("Output Device {}", name);
                        result.push(name);
                    }
                    Err(e) => {
                        println!("Warning: couldn't get the name for the device {}",  e);
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
            &property_address,
            0,
            ptr::null(),
            &mut property_size,
        );

        if status != 0 {
            return false;
        }

        if property_size == 0 {
            return false;
        }

        let mut buffer = vec![0u8; property_size as usize];
        let mut actual_size = property_size;

        let status = AudioObjectGetPropertyData(
            device_id,
            &property_address,
            0,
            ptr::null(),
            &mut actual_size,
            buffer.as_mut_ptr() as *mut c_void,
        );

        if status != 0 {
            return false;
        }
        
        let num_buffers = *(buffer.as_ptr() as *const UInt32);

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
            &property_address as *const _,
            0,
            ptr::null(),
            &mut property_size as *mut _,
            &mut cf_string as *mut _ as *mut _,
        )
    };

    if status != 0 {
        return Err(format!("Failed to get device name: {}", status));
    }

    if cf_string.is_null() {
        return Err("Device name is null".to_string());
    }

    let cf_string_wrapper = unsafe { CFString::wrap_under_create_rule(cf_string) };
    Ok(cf_string_wrapper.to_string())
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
            return 0;
        }

        let device_update_sender = &*(in_client_data as *const Sender<AudioDeviceUpdateEvents>);

        let _ = device_update_sender.send(AudioDeviceUpdateEvents::OutputDeviceListChanged);
    }

    0 // noErr
}

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
