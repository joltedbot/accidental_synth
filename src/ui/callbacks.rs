use anyhow::Result;
use slint::Weak;
use crate::AccidentalSynth;

pub fn register_callbacks(ui_weak: Weak<AccidentalSynth>) -> Result<()> {

    callback_midi_input_channel_changed(ui_weak);

    Ok(())
}


fn callback_midi_input_channel_changed(ui_weak: Weak<AccidentalSynth>) {

    if let Some(ui) = ui_weak.upgrade() {
        ui.on_midi_input_channel_changed(|channel|{
            println!("Midi input channel changed. {}", channel);
        });
    }
}

