use crossbeam_channel::Sender;
use slint::Weak;
use crate::AccidentalSynth;
use crate::synthesizer::SynthesizerUpdateEvents;

pub fn callback_effect_enable(
    ui_weak: &Weak<AccidentalSynth>,
    synthesizer_update_sender: Sender<SynthesizerUpdateEvents>,
) {
    if let Some(ui) = ui_weak.upgrade() {
        ui.on_effect_enabled(move |effect_index, is_enabled| {
            synthesizer_update_sender
                .send(SynthesizerUpdateEvents::EffectEnabled(effect_index, is_enabled))
                .expect(
                    "callback_effect_enable(): Could not send new \
            effect state to the synthesizer module. Exiting.",
                );
        });
    }
}

pub fn callback_effect_parameter_changed(
    ui_weak: &Weak<AccidentalSynth>,
    synthesizer_update_sender: Sender<SynthesizerUpdateEvents>,
) {
    if let Some(ui) = ui_weak.upgrade() {
        ui.on_effect_parameter_changed(move |effect_index, parameter_index, value| {
            synthesizer_update_sender
                .send(SynthesizerUpdateEvents::EffectParameters(effect_index, parameter_index, value))
                .expect(
                    "callback_effect_enable(): Could not send new \
            effect parameter value to the synthesizer module. Exiting.",
                );
        });
    }
}