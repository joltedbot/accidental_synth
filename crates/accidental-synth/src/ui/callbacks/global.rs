use crate::AccidentalSynth;
use accsyn_core::synth_events::SynthesizerUpdateEvents;
use crossbeam_channel::Sender;
use slint::Weak;

pub fn callback_portamento_enabled(
    ui_weak: &Weak<AccidentalSynth>,
    synthesizer_update_sender: Sender<SynthesizerUpdateEvents>,
) {
    if let Some(ui) = ui_weak.upgrade() {
        ui.on_portamento_enabled(move |is_enabled| {
            log::trace!(target: "ui::global", "callback_portamento_enabled(): Sending SynthesizerUpdateEvents::PortamentoEnabled : {is_enabled}");
            synthesizer_update_sender
                .send(SynthesizerUpdateEvents::PortamentoEnabled(is_enabled))
                .expect(
                    "callback_portamento_enabled(): Could not send new \
            portamento state to the synthesizer module.Exiting.",
                );
        });
    }
}

pub fn callback_portamento_time_changed(
    ui_weak: &Weak<AccidentalSynth>,
    synthesizer_update_sender: Sender<SynthesizerUpdateEvents>,
) {
    if let Some(ui) = ui_weak.upgrade() {
        ui.on_portamento_time_changed(move |milliseconds| {
            log::trace!(target: "ui::global", "callback_portamento_time_changed(): Sending SynthesizerUpdateEvents::PortamentoTime : {milliseconds}");
            synthesizer_update_sender
                .send(SynthesizerUpdateEvents::PortamentoTime(milliseconds))
                .expect(
                    "callback_portamento_time_changed(): Could not send new \
            portamento time to the synthesizer module.Exiting.",
                );
        });
    }
}

pub fn callback_pitch_bend_range_changed(
    ui_weak: &Weak<AccidentalSynth>,
    synthesizer_update_sender: Sender<SynthesizerUpdateEvents>,
) {
    if let Some(ui) = ui_weak.upgrade() {
        ui.on_pitch_bend_range_changed(move |range| {
            log::trace!(target: "ui::global", "callback_pitch_bend_range_changed(): Sending SynthesizerUpdateEvents::PitchBendRange : {range}");
            synthesizer_update_sender
                .send(SynthesizerUpdateEvents::PitchBendRange(range))
                .expect(
                    "callback_pitch_bend_range_changed(): Could not send new \
            pitch bend range to the synthesizer module.Exiting.",
                );
        });
    }
}

pub fn callback_velocity_curve_changed(
    ui_weak: &Weak<AccidentalSynth>,
    synthesizer_update_sender: Sender<SynthesizerUpdateEvents>,
) {
    if let Some(ui) = ui_weak.upgrade() {
        ui.on_velocity_curve_changed(move |curve| {
            log::trace!(target: "ui::global", "callback_velocity_curve_changed(): Sending SynthesizerUpdateEvents::VelocityCurve : {curve}");
            synthesizer_update_sender
                .send(SynthesizerUpdateEvents::VelocityCurve(curve))
                .expect(
                    "callback_velocity_curve_changed(): Could not send new \
            velocity curve to the synthesizer module.Exiting.",
                );
        });
    }
}
pub fn callback_hard_sync_enabled(
    ui_weak: &Weak<AccidentalSynth>,
    synthesizer_update_sender: Sender<SynthesizerUpdateEvents>,
) {
    if let Some(ui) = ui_weak.upgrade() {
        ui.on_hard_sync_enabled(move |is_enabled| {
            log::trace!(target: "ui::global", "callback_hard_sync_enabled(): Sending SynthesizerUpdateEvents::HardSyncEnabled : {is_enabled}");
            synthesizer_update_sender
                .send(SynthesizerUpdateEvents::HardSyncEnabled(is_enabled))
                .expect(
                    "callback_hard_sync_enabled(): Could not send new \
            hard sync state to the synthesizer module.Exiting.",
                );
        });
    }
}
pub fn callback_key_sync_enabled(
    ui_weak: &Weak<AccidentalSynth>,
    synthesizer_update_sender: Sender<SynthesizerUpdateEvents>,
) {
    if let Some(ui) = ui_weak.upgrade() {
        ui.on_key_sync_enabled(move |is_enabled| {
            log::trace!(target: "ui::global", "callback_key_sync_enabled(): Sending SynthesizerUpdateEvents::KeySyncEnabled : {is_enabled}");
            synthesizer_update_sender
                .send(SynthesizerUpdateEvents::KeySyncEnabled(is_enabled))
                .expect(
                    "callback_key_sync_enabled(): Could not send new \
            key sync state to the synthesizer module.Exiting.",
                );
        });
    }
}

pub fn callback_polarity_flipped(
    ui_weak: &Weak<AccidentalSynth>,
    synthesizer_update_sender: Sender<SynthesizerUpdateEvents>,
) {
    if let Some(ui) = ui_weak.upgrade() {
        ui.on_polarity_flipped(move |is_flipped| {
            log::trace!(target: "ui::global", "callback_polarity_flipped(): Sending SynthesizerUpdateEvents::PolarityFlipped : {is_flipped}");
            synthesizer_update_sender
                .send(SynthesizerUpdateEvents::PolarityFlipped(is_flipped))
                .expect(
                    "callback_polarity_flipped(): Could not send new \
            polarity state to the synthesizer module.Exiting.",
                );
        });
    }
}

pub fn callback_sustain_enabled(
    ui_weak: &Weak<AccidentalSynth>,
    synthesizer_update_sender: Sender<SynthesizerUpdateEvents>,
) {
    if let Some(ui) = ui_weak.upgrade() {
        ui.on_sustain_enabled(move |is_enabled| {
            log::trace!(target: "ui::global", "callback_sustain_enabled(): Sending SynthesizerUpdateEvents::SustainPedal : {is_enabled}");
            synthesizer_update_sender
                .send(SynthesizerUpdateEvents::SustainPedal(is_enabled))
                .expect(
                    "callback_sustain_enabled(): Could not send new \
            sustain state to the synthesizer module.Exiting.",
                );
        });
    }
}

pub fn callback_output_balance_update(
    ui_weak: &Weak<AccidentalSynth>,
    synthesizer_update_sender: Sender<SynthesizerUpdateEvents>,
) {
    if let Some(ui) = ui_weak.upgrade() {
        ui.on_output_balance_update(move |balance| {
            log::trace!(target: "ui::global", "callback_output_balance_update(): Sending SynthesizerUpdateEvents::OutputBalance : {balance}");
            synthesizer_update_sender
                .send(SynthesizerUpdateEvents::OutputBalance(balance))
                .expect(
                    "callback_output_balance_update(): Could not send new \
            output balance to the synthesizer module.Exiting.",
                );
        });
    }
}
pub fn callback_output_level_update(
    ui_weak: &Weak<AccidentalSynth>,
    synthesizer_update_sender: Sender<SynthesizerUpdateEvents>,
) {
    if let Some(ui) = ui_weak.upgrade() {
        ui.on_output_level_update(move |level| {
            log::trace!(target: "ui::global", "callback_output_level_update(): Sending SynthesizerUpdateEvents::OutputLevel : {level}");
            synthesizer_update_sender
                .send(SynthesizerUpdateEvents::OutputLevel(level))
                .expect(
                    "callback_output_level_update(): Could not send new \
            output level to the synthesizer module.Exiting.",
                );
        });
    }
}
pub fn callback_output_mute_update(
    ui_weak: &Weak<AccidentalSynth>,
    synthesizer_update_sender: Sender<SynthesizerUpdateEvents>,
) {
    if let Some(ui) = ui_weak.upgrade() {
        ui.on_output_mute_update(move |is_muted| {
            log::trace!(target: "ui::global", "callback_output_mute_update(): Sending SynthesizerUpdateEvents::OutputMute : {is_muted}");
            synthesizer_update_sender
                .send(SynthesizerUpdateEvents::OutputMute(is_muted))
                .expect(
                    "callback_output_mute_update(): Could not send new \
            output mute state to the synthesizer module.Exiting.",
                );
        });
    }
}
pub fn callback_osc_mixer_balance_update(
    ui_weak: &Weak<AccidentalSynth>,
    synthesizer_update_sender: Sender<SynthesizerUpdateEvents>,
) {
    if let Some(ui) = ui_weak.upgrade() {
        ui.on_osc_mixer_balance_update(move |oscillator_index, balance| {
            log::trace!(target: "ui::global", "callback_osc_mixer_balance_update(): Sending SynthesizerUpdateEvents::OscillatorMixerBalance : ({oscillator_index}, {balance})");
            synthesizer_update_sender
                .send(SynthesizerUpdateEvents::OscillatorMixerBalance(
                    oscillator_index,
                    balance,
                ))
                .expect(
                    "callback_osc_mixer_balance_update(): Could not send new \
                    oscillator {oscillator_index} balance to the synthesizer module.Exiting.",
                );
        });
    }
}
pub fn callback_osc_mixer_level_update(
    ui_weak: &Weak<AccidentalSynth>,
    synthesizer_update_sender: Sender<SynthesizerUpdateEvents>,
) {
    if let Some(ui) = ui_weak.upgrade() {
        ui.on_osc_mixer_level_update(move |oscillator_index, balance| {
            log::trace!(target: "ui::global", "callback_osc_mixer_level_update(): Sending SynthesizerUpdateEvents::OscillatorMixerLevel : ({oscillator_index}, {balance})");
            synthesizer_update_sender
                .send(SynthesizerUpdateEvents::OscillatorMixerLevel(
                    oscillator_index,
                    balance,
                ))
                .expect(
                    "callback_osc_mixer_level_update(): Could not send new \
            oscillator {oscillator_index} level to the synthesizer module.Exiting.",
                );
        });
    }
}
pub fn callback_osc_mixer_mute_update(
    ui_weak: &Weak<AccidentalSynth>,
    synthesizer_update_sender: Sender<SynthesizerUpdateEvents>,
) {
    if let Some(ui) = ui_weak.upgrade() {
        ui.on_osc_mixer_mute_update(move |oscillator_index, is_muted| {
            log::trace!(target: "ui::global", "callback_osc_mixer_mute_update(): Sending SynthesizerUpdateEvents::OscillatorMixerMute : ({oscillator_index}, {is_muted})");
            synthesizer_update_sender
                .send(SynthesizerUpdateEvents::OscillatorMixerMute(
                    oscillator_index,
                    is_muted,
                ))
                .expect(
                    "callback_osc_mixer_mute_update(): Could not send new \
            oscillator {oscillator_index} mute state to the synthesizer module.Exiting.",
                );
        });
    }
}
