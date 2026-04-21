use crate::AccidentalSynth;
use crate::ui::constants::AUDIO_DEVICE_CHANNEL_NULL_VALUE;
use accsyn_midi::MidiDeviceUpdateEvents;
use accsyn_types::audio_events::AudioDeviceUpdateEvents;
use accsyn_types::synth_events::SynthesizerUpdateEvents;
use accsyn_types::ui_events::UIUpdates;
use crossbeam_channel::Sender;
use slint::Weak;

pub fn callback_midi_input_channel_changed(
    ui_weak: &Weak<AccidentalSynth>,
    midi_update_sender: Sender<MidiDeviceUpdateEvents>,
) {
    if let Some(ui) = ui_weak.upgrade() {
        ui.on_midi_input_channel_changed(move |channel| {
            midi_update_sender.send(MidiDeviceUpdateEvents::UIMidiInputChannelIndex(channel.to_string())).expect(
                "callback_midi_input_port_changed(): Could not send new midi port name update to the midi module. \
                Exiting. ",
            );
        });
    }
}

pub fn callback_midi_input_port_changed(
    ui_weak: &Weak<AccidentalSynth>,
    midi_update_sender: Sender<MidiDeviceUpdateEvents>,
) {
    if let Some(ui) = ui_weak.upgrade() {
        ui.on_midi_input_port_changed(move |port|{
            midi_update_sender.send(MidiDeviceUpdateEvents::UIMidiInputPort(port.to_string())).expect(
                "callback_midi_input_port_changed(): Could not send new midi port name update to the midi module. \
                Exiting. ",
            );
        });
    }
}

pub fn callback_audio_output_device_changed(
    ui_weak: &Weak<AccidentalSynth>,
    audio_output_device_sender: Sender<AudioDeviceUpdateEvents>,
) {
    if let Some(ui) = ui_weak.upgrade() {
        ui.on_audio_output_device_changed(move |device| {
            audio_output_device_sender.send(AudioDeviceUpdateEvents::UIOutputDevice(device.to_string())).expect(
                "callback_audio_output_device_changed(): Could not send new audio output device update to the audio module.Exiting.",
            );
        });
    }
}

pub fn callback_audio_output_left_channel_changed(
    ui_weak: &Weak<AccidentalSynth>,
    audio_output_device_sender: Sender<AudioDeviceUpdateEvents>,
) {
    if let Some(ui) = ui_weak.upgrade() {
        ui.on_audio_output_left_channel_changed(move |channel| {
            let channel_number = channel.parse::<i32>().unwrap_or(AUDIO_DEVICE_CHANNEL_NULL_VALUE);
            audio_output_device_sender.send(AudioDeviceUpdateEvents::UIOutputDeviceLeftChannel(channel_number)).expect(
                "callback_audio_output_device_changed(): Could not send new audio output device update to the audio module.Exiting.",
            );
        });
    }
}

pub fn callback_audio_output_right_channel_changed(
    ui_weak: &Weak<AccidentalSynth>,
    audio_output_device_sender: Sender<AudioDeviceUpdateEvents>,
) {
    if let Some(ui) = ui_weak.upgrade() {
        ui.on_audio_output_right_channel_changed(move |channel| {
            let channel_number = channel.parse::<i32>().unwrap_or(AUDIO_DEVICE_CHANNEL_NULL_VALUE);
            audio_output_device_sender.send(AudioDeviceUpdateEvents::UIOutputDeviceRightChannel(channel_number))
                .expect(
                "callback_audio_output_device_changed(): Could not send new audio output device update to the audio module.Exiting.",
            );
        });
    }
}

pub fn callback_audio_sample_rate_changed(
    ui_weak: &Weak<AccidentalSynth>,
    audio_output_device_sender: Sender<AudioDeviceUpdateEvents>,
) {
    if let Some(ui) = ui_weak.upgrade() {
        ui.on_audio_sample_rate_changed(move |rate| {
            audio_output_device_sender
                .send(AudioDeviceUpdateEvents::SampleRateChanged(String::from(rate))).expect("callback_audio_output_device_changed(): Could not send new audio sample  rate update to the audio module.Exiting.");
        });
    }
}

pub fn callback_audio_buffer_size_changed(
    ui_weak: &Weak<AccidentalSynth>,
    audio_output_device_sender: Sender<AudioDeviceUpdateEvents>,
) {
    if let Some(ui) = ui_weak.upgrade() {
        ui.on_audio_buffer_size_changed(move |size| {
            audio_output_device_sender
                .send(AudioDeviceUpdateEvents::BufferSizeChanged(String::from(size))).expect("callback_audio_output_device_changed(): Could not send new audio buffer size update to the audio module.Exiting.");
        });
    }
}

pub fn callback_patch_changed(
    ui_weak: &Weak<AccidentalSynth>,
    synthesizer_update_sender: Sender<SynthesizerUpdateEvents>,
    ui_update_sender: Sender<UIUpdates>,
) {
    if let Some(ui) = ui_weak.upgrade() {
        ui.on_patch_changed(move |patch_index| {
            synthesizer_update_sender.send(SynthesizerUpdateEvents::PatchChanged(patch_index)).expect(
                "callback_preset_changed(): Could not send new preset update to the audio module. Exiting.",
            );
            ui_update_sender.send(UIUpdates::Patches(patch_index)).expect(
                "callback_preset_changed(): Could not send preset UI update. Exiting.",
            );
        });
    }
}

pub fn callback_patch_saved(
    ui_weak: &Weak<AccidentalSynth>,
    synthesizer_update_sender: Sender<SynthesizerUpdateEvents>,
) {
    if let Some(ui) = ui_weak.upgrade() {
        ui.on_patch_saved(move |patch_name| {
            log::trace!("callback_patch_saved(): Sending SynthesizerUpdateEvents::PatchSaved : {}", patch_name);
            synthesizer_update_sender.send(SynthesizerUpdateEvents::PatchSaved(patch_name.trim().to_string())).expect(
                "callback_preset_changed(): Could not send saved preset update to the synthesizer module. Exiting.",
            );
        });
    }
}
pub fn callback_patch_deleted(
    ui_weak: &Weak<AccidentalSynth>,
    synthesizer_update_sender: Sender<SynthesizerUpdateEvents>,
) {
    if let Some(ui) = ui_weak.upgrade() {
        ui.on_patch_deleted(move |patch_index| {
            log::trace!("callback_patch_deleted(): Sending SynthesizerUpdateEvents::PatchDeleted : {}", patch_index);
            synthesizer_update_sender.send(SynthesizerUpdateEvents::PatchDeleted(patch_index)).expect(
                "callback_preset_changed(): Could not send deleted preset update to the synthesizer module. Exiting.",
            );
        });
    }
}
