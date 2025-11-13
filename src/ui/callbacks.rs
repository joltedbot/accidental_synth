use crate::AccidentalSynth;
use crate::midi::MidiDeviceEvent;
use anyhow::Result;
use crossbeam_channel::Sender;
use slint::Weak;

pub fn register_callbacks(
    ui_weak: Weak<AccidentalSynth>,
    midi_update_sender: Sender<MidiDeviceEvent>,
) -> Result<()> {
    callback_midi_input_channel_changed(&ui_weak, midi_update_sender.clone());
    callback_midi_input_port_changed(&ui_weak, midi_update_sender);
    callback_audio_output_device_changed(&ui_weak);
    callback_audio_output_left_channel_changed(&ui_weak);
    callback_audio_output_right_channel_changed(&ui_weak);
    callback_audio_sample_rate_changed(&ui_weak);

    Ok(())
}

fn callback_midi_input_channel_changed(
    ui_weak: &Weak<AccidentalSynth>,
    midi_update_sender: Sender<MidiDeviceEvent>,
) {
    if let Some(ui) = ui_weak.upgrade() {
        ui.on_midi_input_channel_changed(move |channel| {
            midi_update_sender.send(MidiDeviceEvent::UIMidiInputChannelIndexUpdated(channel.to_string())).expect(
                "callback_midi_input_port_changed(): Could not send new midi port name update to the midi module. \
                Exiting. ",
            );
        });
    }
}

fn callback_midi_input_port_changed(
    ui_weak: &Weak<AccidentalSynth>,
    midi_update_sender: Sender<MidiDeviceEvent>,
) {
    if let Some(ui) = ui_weak.upgrade() {
        ui.on_midi_input_port_changed(move |port|{
            midi_update_sender.send(MidiDeviceEvent::UIMidiInputPortUpdated(port.to_string())).expect(
                "callback_midi_input_port_changed(): Could not send new midi port name update to the midi module. \
                Exiting. ",
            );
        });
    }
}

fn callback_audio_output_device_changed(ui_weak: &Weak<AccidentalSynth>) {
    if let Some(ui) = ui_weak.upgrade() {
        ui.on_audio_output_device_changed(|device| {
            println!("Audio output device changed. {}", device);
        });
    }
}

fn callback_audio_output_left_channel_changed(ui_weak: &Weak<AccidentalSynth>) {
    if let Some(ui) = ui_weak.upgrade() {
        ui.on_audio_output_left_channel_changed(|channel| {
            println!("Audio output left channel changed. {}", channel);
        });
    }
}

fn callback_audio_output_right_channel_changed(ui_weak: &Weak<AccidentalSynth>) {
    if let Some(ui) = ui_weak.upgrade() {
        ui.on_audio_output_right_channel_changed(|channel| {
            println!("Audio output right channel changed. {}", channel);
        });
    }
}

fn callback_audio_sample_rate_changed(ui_weak: &Weak<AccidentalSynth>) {
    if let Some(ui) = ui_weak.upgrade() {
        ui.on_audio_sample_rate_changed(|channel| {
            println!("Audio sample rate changed. {}", channel);
        });
    }
}
