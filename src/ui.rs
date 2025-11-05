mod constants;
mod callbacks;

use std::default::Default;
use super::AccidentalSynth;
use crossbeam_channel::{Sender, Receiver, bounded};
use slint::{SharedString, Weak};
use std::thread;
use anyhow::Result;
use crate::ui::constants::MIDI_CHANNEL_LIST;
use crate::ui::callbacks::register_callbacks;

const UI_UPDATE_CHANNEL_CAPACITY: usize = 10;

#[derive(Clone, Default)]
struct MidiPort {
    input_ports: Vec<String>,
    input_port_index: i32,
    channels: [&'static str; 17],
    channel_index: i32,
}

#[derive(Debug, Clone, PartialEq)]
pub enum UIUpdates {

}

pub struct UI {
    ui_update_sender: Sender<UIUpdates>,
    ui_update_receiver: Receiver<UIUpdates>,
}

impl UI {
    pub fn new() -> Self {
        log::info!("Constructing UI Module");

        let (ui_update_sender, ui_update_receiver) = bounded(UI_UPDATE_CHANNEL_CAPACITY);

        let me = MidiPort {
          channels: MIDI_CHANNEL_LIST,
            ..Default::default()
        };

        Self {
            ui_update_sender,
            ui_update_receiver,
        }
    }

    pub fn get_ui_update_sender(&self) -> Sender<UIUpdates> {
        self.ui_update_sender.clone()
    }

    pub fn run(&mut self, ui_weak: Weak<AccidentalSynth>) -> Result<()> {
        let ui_updates = self.ui_update_receiver.clone();

        self.set_ui_default_values(ui_weak.clone())?;
        register_callbacks(ui_weak.clone())?;

        Ok(())
    }

    fn set_ui_default_values(&self, ui_weak: Weak<AccidentalSynth>) -> Result<()> {

        ui_weak.upgrade_in_event_loop(move |ui| {
            ui.set_version(SharedString::from(env!("CARGO_PKG_VERSION")));
        })?;

        Ok(())
    }

}

#[cfg(test)]
mod tests {
    use crossbeam_channel::internal::SelectHandle;
    use super::*;

    #[test]
    fn new_returns_correct_object_contents() {
        let ui = UI::new();
        let ui_update_sender = ui.get_ui_update_sender();
        assert!(ui_update_sender.is_ready());
    }
}