use crate::AccidentalSynth;
use crate::audio::AudioDeviceUpdateEvents;
use crate::midi::MidiDeviceUpdateEvents;
use crate::ui::constants::AUDIO_DEVICE_CHANNEL_NULL_VALUE;
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

pub fn callback_audio_sample_rate_changed(ui_weak: &Weak<AccidentalSynth>) {
    if let Some(ui) = ui_weak.upgrade() {
        ui.on_audio_sample_rate_changed(|channel| {
            println!("Audio sample rate changed. {channel}");
        });
    }
}
