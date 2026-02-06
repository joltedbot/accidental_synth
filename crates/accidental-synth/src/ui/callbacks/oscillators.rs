use crate::AccidentalSynth;
use accsyn_types::synth_events::SynthesizerUpdateEvents;
use crossbeam_channel::Sender;
use slint::Weak;

pub fn callback_osc_oscillator_shape_changed(
    ui_weak: &Weak<AccidentalSynth>,
    synthesizer_update_sender: Sender<SynthesizerUpdateEvents>,
) {
    if let Some(ui) = ui_weak.upgrade() {
        ui.on_osc_wave_shape_changed(move |oscillator_index, shape_index| {
            synthesizer_update_sender
                .send(SynthesizerUpdateEvents::WaveShapeIndex(
                    oscillator_index,
                    shape_index,
                ))
                .expect(
                    "callback_osc_oscillator_shape_changed(): Could not send new \
            synthesizer oscillator shape update to the synthesizer module.Exiting.",
                );
        });
    }
}

pub fn callback_osc_course_tune_changed(
    ui_weak: &Weak<AccidentalSynth>,
    synthesizer_update_sender: Sender<SynthesizerUpdateEvents>,
) {
    if let Some(ui) = ui_weak.upgrade() {
        ui.on_osc_course_tune_changed(move |oscillator_index, course_tune| {
            synthesizer_update_sender
                .send(SynthesizerUpdateEvents::CourseTune(
                    oscillator_index,
                    course_tune,
                ))
                .expect(
                    "callback_osc_course_tune_changed(): Could not send new \
            synthesizer oscillator course tune to the synthesizer module.Exiting.",
                );
        });
    }
}

pub fn callback_osc_fine_tune_changed(
    ui_weak: &Weak<AccidentalSynth>,
    synthesizer_update_sender: Sender<SynthesizerUpdateEvents>,
) {
    if let Some(ui) = ui_weak.upgrade() {
        ui.on_osc_fine_tune_changed(move |oscillator_index, fine_tune| {
            synthesizer_update_sender
                .send(SynthesizerUpdateEvents::FineTune(
                    oscillator_index,
                    fine_tune,
                ))
                .expect(
                    "callback_osc_fine_tune_changed(): Could not send new \
            synthesizer oscillator fine tune to the synthesizer module.Exiting.",
                );
        });
    }
}

pub fn callback_osc_clipper_boost_changed(
    ui_weak: &Weak<AccidentalSynth>,
    synthesizer_update_sender: Sender<SynthesizerUpdateEvents>,
) {
    if let Some(ui) = ui_weak.upgrade() {
        ui.on_osc_clipper_boost_changed(move |oscillator_index, boost| {
            synthesizer_update_sender
                .send(SynthesizerUpdateEvents::ClipperBoost(
                    oscillator_index,
                    boost,
                ))
                .expect(
                    "callback_osc_clipper_boost_changed(): Could not send new \
            synthesizer oscillator clipper boost to the synthesizer module.Exiting.",
                );
        });
    }
}

pub fn callback_osc_parameter1_changed(
    ui_weak: &Weak<AccidentalSynth>,
    synthesizer_update_sender: Sender<SynthesizerUpdateEvents>,
) {
    if let Some(ui) = ui_weak.upgrade() {
        ui.on_osc_parameter1_changed(move |oscillator_index, value| {
            synthesizer_update_sender
                .send(SynthesizerUpdateEvents::Parameter1(oscillator_index, value))
                .expect(
                    "callback_osc_parameter1_changed(): Could not send new \
            synthesizer oscillator parameter1 to the synthesizer module.Exiting.",
                );
        });
    }
}

pub fn callback_osc_parameter2_changed(
    ui_weak: &Weak<AccidentalSynth>,
    synthesizer_update_sender: Sender<SynthesizerUpdateEvents>,
) {
    if let Some(ui) = ui_weak.upgrade() {
        ui.on_osc_parameter2_changed(move |oscillator_index, value| {
            synthesizer_update_sender
                .send(SynthesizerUpdateEvents::Parameter2(oscillator_index, value))
                .expect(
                    "callback_osc_parameter2_changed(): Could not send new \
            synthesizer oscillator parameter2 to the synthesizer module.Exiting.",
                );
        });
    }
}
