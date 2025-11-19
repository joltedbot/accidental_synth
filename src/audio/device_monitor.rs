use crate::audio::{
    AudioDeviceUpdateEvents, DEVICE_LIST_POLLING_INTERVAL_IN_MS,
    update_current_output_device_list_if_changed,
};
use cpal::default_host;
use crossbeam_channel::Sender;
use std::thread;
use std::thread::sleep;
use std::time::Duration;

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
