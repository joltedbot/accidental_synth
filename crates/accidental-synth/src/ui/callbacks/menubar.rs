use crate::AccidentalSynth;
use crate::ui::constants::{GIT_REPO_URL, MANUAL_URL, MIDI_IMPLEMENTATION_CHART_URL};
use slint::Weak;
use std::path::PathBuf;

pub fn callback_open_manual(ui_weak: &Weak<AccidentalSynth>) {
    if let Some(ui) = ui_weak.upgrade() {
        ui.on_open_manual(|| {
            if let Err(error) = open::that(MANUAL_URL) {
                log::warn!("Failed to open manual page: {error}");
            }
        });
    }
}

pub fn callback_open_midi_chart(ui_weak: &Weak<AccidentalSynth>) {
    if let Some(ui) = ui_weak.upgrade() {
        ui.on_open_midi_chart(|| {
            if let Err(error) = open::that(MIDI_IMPLEMENTATION_CHART_URL) {
                log::warn!("Failed to open midi implementation chart page: {error}");
            }
        });
    }
}

pub fn callback_open_patch_folder(ui_weak: &Weak<AccidentalSynth>, patch_directory: PathBuf) {
    if let Some(ui) = ui_weak.upgrade() {
        ui.on_open_patch_folder(move || {
            if let Err(error) = open::that(patch_directory.as_os_str()) {
                log::warn!("Failed to open patch folder page: {error}");
            }
        });
    }
}

pub fn callback_open_git_repo(ui_weak: &Weak<AccidentalSynth>) {
    if let Some(ui) = ui_weak.upgrade() {
        ui.on_open_git_repo(|| {
            if let Err(error) = open::that(GIT_REPO_URL) {
                log::warn!("Failed to open git repo page: {error}");
            }
        });
    }
}
