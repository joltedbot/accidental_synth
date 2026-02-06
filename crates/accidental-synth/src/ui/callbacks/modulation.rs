use crate::AccidentalSynth;
use accsyn_types::synth_events::SynthesizerUpdateEvents;
use crossbeam_channel::Sender;
use slint::Weak;

pub fn callback_envelope_attack_changed(
    ui_weak: &Weak<AccidentalSynth>,
    synthesizer_update_sender: Sender<SynthesizerUpdateEvents>,
) {
    if let Some(ui) = ui_weak.upgrade() {
        ui.on_envelope_attack_changed(move |envelope_index, milliseconds| {
            synthesizer_update_sender
                .send(SynthesizerUpdateEvents::FilterEnvelopeAttack(
                    envelope_index,
                    milliseconds,
                ))
                .expect(
                    "callback_filter_lfo_amount_changed(): Could not send new \
            envelope attack to the synthesizer module.Exiting.",
                );
        });
    }
}

pub fn callback_envelope_decay_changed(
    ui_weak: &Weak<AccidentalSynth>,
    synthesizer_update_sender: Sender<SynthesizerUpdateEvents>,
) {
    if let Some(ui) = ui_weak.upgrade() {
        ui.on_envelope_decay_changed(move |envelope_index, milliseconds| {
            synthesizer_update_sender
                .send(SynthesizerUpdateEvents::FilterEnvelopeDecay(
                    envelope_index,
                    milliseconds,
                ))
                .expect(
                    "callback_filter_lfo_amount_changed(): Could not send new \
            envelope decay to the synthesizer module.Exiting.",
                );
        });
    }
}

pub fn callback_envelope_sustain_changed(
    ui_weak: &Weak<AccidentalSynth>,
    synthesizer_update_sender: Sender<SynthesizerUpdateEvents>,
) {
    if let Some(ui) = ui_weak.upgrade() {
        ui.on_envelope_sustain_changed(move |envelope_index, level| {
            synthesizer_update_sender
                .send(SynthesizerUpdateEvents::FilterEnvelopeSustain(
                    envelope_index,
                    level,
                ))
                .expect(
                    "callback_filter_lfo_amount_changed(): Could not send new \
            envelope sustain to the synthesizer module.Exiting.",
                );
        });
    }
}

pub fn callback_envelope_release_changed(
    ui_weak: &Weak<AccidentalSynth>,
    synthesizer_update_sender: Sender<SynthesizerUpdateEvents>,
) {
    if let Some(ui) = ui_weak.upgrade() {
        ui.on_envelope_release_changed(move |envelope_index, milliseconds| {
            synthesizer_update_sender
                .send(SynthesizerUpdateEvents::FilterEnvelopeRelease(
                    envelope_index,
                    milliseconds,
                ))
                .expect(
                    "callback_filter_lfo_amount_changed(): Could not send new \
            envelope release to the synthesizer module.Exiting.",
                );
        });
    }
}

pub fn callback_envelope_invert_changed(
    ui_weak: &Weak<AccidentalSynth>,
    synthesizer_update_sender: Sender<SynthesizerUpdateEvents>,
) {
    if let Some(ui) = ui_weak.upgrade() {
        ui.on_envelope_invert_changed(move |filter_index, is_inverted| {
            synthesizer_update_sender
                .send(SynthesizerUpdateEvents::FilterEnvelopeInvert(
                    filter_index,
                    is_inverted,
                ))
                .expect(
                    "callback_filter_lfo_amount_changed(): Could not send new \
            envelope inverted state to the synthesizer module.Exiting.",
                );
        });
    }
}

pub fn callback_lfo_frequency_changed(
    ui_weak: &Weak<AccidentalSynth>,
    synthesizer_update_sender: Sender<SynthesizerUpdateEvents>,
) {
    if let Some(ui) = ui_weak.upgrade() {
        ui.on_lfo_frequency_changed(move |lfo_index, frequency| {
            synthesizer_update_sender
                .send(SynthesizerUpdateEvents::LfoFrequency(lfo_index, frequency))
                .expect(
                    "callback_filter_lfo_amount_changed(): Could not send new \
            LFO Frequency to the synthesizer module.Exiting.",
                );
        });
    }
}

pub fn callback_lfo_shape_changed(
    ui_weak: &Weak<AccidentalSynth>,
    synthesizer_update_sender: Sender<SynthesizerUpdateEvents>,
) {
    if let Some(ui) = ui_weak.upgrade() {
        ui.on_lfo_shape_changed(move |lfo_index, shape_index| {
            synthesizer_update_sender
                .send(SynthesizerUpdateEvents::LfoShapeIndex(
                    lfo_index,
                    shape_index,
                ))
                .expect(
                    "callback_filter_lfo_amount_changed(): Could not send new \
            LFO shape index to the synthesizer module.Exiting.",
                );
        });
    }
}

pub fn callback_lfo_phase_changed(
    ui_weak: &Weak<AccidentalSynth>,
    synthesizer_update_sender: Sender<SynthesizerUpdateEvents>,
) {
    if let Some(ui) = ui_weak.upgrade() {
        ui.on_lfo_phase_changed(move |lfo_index, phase| {
            synthesizer_update_sender
                .send(SynthesizerUpdateEvents::LfoPhase(lfo_index, phase))
                .expect(
                    "callback_filter_lfo_amount_changed(): Could not send new \
            LFO phase to the synthesizer module.Exiting.",
                );
        });
    }
}

pub fn callback_lfo_phase_reset(
    ui_weak: &Weak<AccidentalSynth>,
    synthesizer_update_sender: Sender<SynthesizerUpdateEvents>,
) {
    if let Some(ui) = ui_weak.upgrade() {
        ui.on_lfo_phase_reset(move |lfo_index| {
            synthesizer_update_sender
                .send(SynthesizerUpdateEvents::LfoPhaseReset(lfo_index))
                .expect(
                    "callback_filter_lfo_amount_changed(): Could not send  \
            LFO phase reset command to the synthesizer module.Exiting.",
                );
        });
    }
}
