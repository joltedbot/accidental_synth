use anyhow::Result;
use slint::Weak;
use crate::AccidentalSynth;

pub fn register_callbacks(ui_weak: Weak<AccidentalSynth>) -> Result<()> {

    callback_midi_input_channel_changed(&ui_weak);
    callback_midi_input_port_changed(&ui_weak);
    callback_audio_output_device_changed(&ui_weak);
    callback_audio_output_left_channel_changed(&ui_weak);
    callback_audio_output_right_channel_changed(&ui_weak);
    callback_audio_sample_rate_changed(&ui_weak);
    
    Ok(())
}


fn callback_midi_input_channel_changed(ui_weak: &Weak<AccidentalSynth>) {
    if let Some(ui) = ui_weak.upgrade() {
        ui.on_midi_input_channel_changed(|channel|{
            println!("Midi input channel changed. {}", channel);
        });
    }
}

fn callback_midi_input_port_changed(ui_weak: &Weak<AccidentalSynth>) {
    if let Some(ui) = ui_weak.upgrade() {
        ui.on_midi_input_port_changed(|port|{
            println!("Midi input port changed. {}", port);
        });
    }
}

fn callback_audio_output_device_changed(ui_weak: &Weak<AccidentalSynth>) {
    if let Some(ui) = ui_weak.upgrade() {
        ui.on_audio_output_device_changed(|device|{
            println!("Audio output device changed. {}", device);
        });
    }
}

fn callback_audio_output_left_channel_changed(ui_weak: &Weak<AccidentalSynth>) {
    if let Some(ui) = ui_weak.upgrade() {
        ui.on_audio_output_left_channel_changed(|channel|{
            println!("Audio output left channel changed. {}", channel);
        });
    }
}

fn callback_audio_output_right_channel_changed(ui_weak: &Weak<AccidentalSynth>) {
    if let Some(ui) = ui_weak.upgrade() {
        ui.on_audio_output_right_channel_changed(|channel|{
            println!("Audio output right channel changed. {}", channel);
        });
    }
}

fn callback_audio_sample_rate_changed(ui_weak: &Weak<AccidentalSynth>) {
    if let Some(ui) = ui_weak.upgrade() {
        ui.on_audio_sample_rate_changed(|channel|{
            println!("Audio sample rate changed. {}", channel);
        });
    }
}
