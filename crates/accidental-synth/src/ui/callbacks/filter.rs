use crate::AccidentalSynth;
use accsyn_types::synth_events::SynthesizerUpdateEvents;
use crossbeam_channel::Sender;
use slint::Weak;

pub fn callback_filter_cutoff_changed(
    ui_weak: &Weak<AccidentalSynth>,
    synthesizer_update_sender: Sender<SynthesizerUpdateEvents>,
) {
    if let Some(ui) = ui_weak.upgrade() {
        ui.on_filter_cutoff_changed(move |cutoff| {
            synthesizer_update_sender
                .send(SynthesizerUpdateEvents::FilterCutoffFrequency(cutoff))
                .expect(
                    "callback_filter_cutoff_changed(): Could not send new \
            synthesizer filter cutoff frequency to the synthesizer module.Exiting.",
                );
        });
    }
}

pub fn callback_filter_resonance_changed(
    ui_weak: &Weak<AccidentalSynth>,
    synthesizer_update_sender: Sender<SynthesizerUpdateEvents>,
) {
    if let Some(ui) = ui_weak.upgrade() {
        ui.on_filter_resonance_changed(move |value| {
            synthesizer_update_sender
                .send(SynthesizerUpdateEvents::FilterResonance(value))
                .expect(
                    "callback_filter_resonance_changed(): Could not send new \
            synthesizer filter resonance to the synthesizer module.Exiting.",
                );
        });
    }
}

pub fn callback_filter_poles_changed(
    ui_weak: &Weak<AccidentalSynth>,
    synthesizer_update_sender: Sender<SynthesizerUpdateEvents>,
) {
    if let Some(ui) = ui_weak.upgrade() {
        ui.on_filter_poles_changed(move |value| {
            synthesizer_update_sender
                .send(SynthesizerUpdateEvents::FilterPoleCount(value))
                .expect(
                    "callback_filter_poles_changed(): Could not send new \
            synthesizer filter pole count to the synthesizer module.Exiting.",
                );
        });
    }
}

pub fn callback_filter_key_tracking_changed(
    ui_weak: &Weak<AccidentalSynth>,
    synthesizer_update_sender: Sender<SynthesizerUpdateEvents>,
) {
    if let Some(ui) = ui_weak.upgrade() {
        ui.on_filter_key_tracking_changed(move |value| {
            synthesizer_update_sender
                .send(SynthesizerUpdateEvents::FilterKeyTrackingAmount(value))
                .expect(
                    "callback_filter_key_tracking_changed(): Could not send new \
            synthesizer filter key tracking amount to the synthesizer module.Exiting.",
                );
        });
    }
}

pub fn callback_filter_envelope_amount_changed(
    ui_weak: &Weak<AccidentalSynth>,
    synthesizer_update_sender: Sender<SynthesizerUpdateEvents>,
) {
    if let Some(ui) = ui_weak.upgrade() {
        ui.on_filter_envelope_amount_changed(move |value| {
            synthesizer_update_sender
                .send(SynthesizerUpdateEvents::FilterEnvelopeAmount(value))
                .expect(
                    "callback_filter_envelope_amount_changed(): Could not send new \
            synthesizer filter envelope amount to the synthesizer module.Exiting.",
                );
        });
    }
}

pub fn callback_filter_lfo_amount_changed(
    ui_weak: &Weak<AccidentalSynth>,
    synthesizer_update_sender: Sender<SynthesizerUpdateEvents>,
) {
    if let Some(ui) = ui_weak.upgrade() {
        ui.on_filter_lfo_amount_changed(move |value| {
            synthesizer_update_sender
                .send(SynthesizerUpdateEvents::FilterLfoAmount(value))
                .expect(
                    "callback_filter_lfo_amount_changed(): Could not send new \
            synthesizer filter lfo amount to the synthesizer module.Exiting.",
                );
        });
    }
}
