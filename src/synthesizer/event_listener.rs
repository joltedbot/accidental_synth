use crate::modules::effects::EffectIndex;
use crate::modules::lfo::DEFAULT_LFO_PHASE;
use crate::synthesizer::constants::{
    ENVELOPE_INDEX_AMP, ENVELOPE_INDEX_FILTER, LFO_INDEX_FILTER, LFO_INDEX_MOD_WHEEL,
};
use crate::synthesizer::set_parameters::{
    set_effect_is_enabled, set_effect_parameter, set_envelope_amount, set_envelope_attack_time,
    set_envelope_decay_time, set_envelope_inverted, set_envelope_release_time,
    set_envelope_sustain_level, set_filter_cutoff, set_filter_poles, set_filter_resonance,
    set_key_tracking_amount, set_lfo_frequency, set_lfo_phase, set_lfo_phase_reset, set_lfo_range,
    set_oscillator_balance, set_oscillator_clip_boost, set_oscillator_course_tune,
    set_oscillator_fine_tune, set_oscillator_hard_sync, set_oscillator_key_sync,
    set_oscillator_level, set_oscillator_mute, set_oscillator_shape_parameter1,
    set_oscillator_shape_parameter2, set_output_balance, set_output_level, set_output_mute,
    set_pitch_bend_range, set_portamento_enabled, set_portamento_time, set_velocity_curve,
};
use crate::synthesizer::{
    CurrentNote, EnvelopeIndex, LFOIndex, ModuleParameters, OscillatorIndex,
    SynthesizerUpdateEvents,
};
use crate::ui::UIUpdates;
use crossbeam_channel::{Receiver, Sender};
use std::sync::Arc;
use std::sync::atomic::Ordering::Relaxed;
use std::thread;

#[allow(clippy::too_many_lines)]
pub fn start_update_event_listener(
    ui_update_receiver: Receiver<SynthesizerUpdateEvents>,
    module_parameters: Arc<ModuleParameters>,
    mut current_note: Arc<CurrentNote>,
    ui_update_sender: Sender<UIUpdates>,
) {
    thread::spawn(move || {
        log::debug!("start_update_event_listener(): spawned thread to receive UI events");

        while let Ok(event) = ui_update_receiver.recv() {
            match event {
                SynthesizerUpdateEvents::WaveShapeIndex(oscillator_index, wave_shape_index) => {
                    if oscillator_index >= 0
                        && oscillator_index < module_parameters.oscillators.len() as i32
                    {
                        module_parameters.oscillators[oscillator_index as usize]
                            .wave_shape_index
                            .store(wave_shape_index as u8, Relaxed);
                    } else {
                        log::warn!(
                            "start_update_event_listener(): Invalid oscillator index: {oscillator_index}"
                        );
                    }
                }
                SynthesizerUpdateEvents::CourseTune(oscillator_index, course_tune) => {
                    if oscillator_index >= 0
                        && oscillator_index < module_parameters.oscillators.len() as i32
                    {
                        set_oscillator_course_tune(
                            &module_parameters.oscillators[oscillator_index as usize],
                            course_tune,
                        );
                    } else {
                        log::warn!(
                            "start_update_event_listener(): Invalid oscillator index: {oscillator_index}"
                        );
                    }
                }
                SynthesizerUpdateEvents::FineTune(oscillator_index, fine_tune) => {
                    if oscillator_index >= 0
                        && oscillator_index < module_parameters.oscillators.len() as i32
                    {
                        let cents = set_oscillator_fine_tune(
                            &module_parameters.oscillators[oscillator_index as usize],
                            fine_tune,
                        );

                        ui_update_sender.send(UIUpdates::OscillatorFineTuneCents(oscillator_index, i32::from(cents))).expect
                        ("start_update_event_listener(): Failed to send oscillator fine-tune display value to the UI.");
                    } else {
                        log::warn!(
                            "start_update_event_listener(): Invalid oscillator index: {oscillator_index}"
                        );
                    }
                }
                SynthesizerUpdateEvents::ClipperBoost(oscillator_index, boost) => {
                    if oscillator_index >= 0
                        && oscillator_index < module_parameters.oscillators.len() as i32
                    {
                        set_oscillator_clip_boost(
                            &module_parameters.oscillators[oscillator_index as usize],
                            boost,
                        );
                    } else {
                        log::warn!(
                            "start_update_event_listener(): Invalid oscillator index: {oscillator_index}"
                        );
                    }
                }
                SynthesizerUpdateEvents::Parameter1(oscillator_index, boost) => {
                    if oscillator_index >= 0
                        && oscillator_index < module_parameters.oscillators.len() as i32
                    {
                        set_oscillator_shape_parameter1(
                            &module_parameters.oscillators[oscillator_index as usize],
                            boost,
                        );
                    } else {
                        log::warn!(
                            "start_update_event_listener(): Invalid oscillator index: {oscillator_index}"
                        );
                    }
                }
                SynthesizerUpdateEvents::Parameter2(oscillator_index, boost) => {
                    if oscillator_index >= 0
                        && oscillator_index < module_parameters.oscillators.len() as i32
                    {
                        set_oscillator_shape_parameter2(
                            &module_parameters.oscillators[oscillator_index as usize],
                            boost,
                        );
                    } else {
                        log::warn!(
                            "start_update_event_listener(): Invalid oscillator index: {oscillator_index}"
                        );
                    }
                }
                SynthesizerUpdateEvents::FilterCutoffFrequency(frequency) => {
                    set_filter_cutoff(&module_parameters.filter, frequency);
                }
                SynthesizerUpdateEvents::FilterResonance(resonance) => {
                    set_filter_resonance(&module_parameters.filter, resonance);
                }
                SynthesizerUpdateEvents::FilterPoleCount(poles) => {
                    set_filter_poles(&module_parameters.filter, poles);
                }
                SynthesizerUpdateEvents::FilterKeyTrackingAmount(amount) => {
                    set_key_tracking_amount(&module_parameters.filter, amount);
                }
                SynthesizerUpdateEvents::FilterEnvelopeAmount(amount) => {
                    set_envelope_amount(
                        &module_parameters.envelopes[EnvelopeIndex::Filter as usize],
                        amount,
                    );
                }
                SynthesizerUpdateEvents::FilterLfoAmount(amount) => {
                    set_lfo_range(&module_parameters.lfos[LFOIndex::Filter as usize], amount);
                }
                SynthesizerUpdateEvents::FilterEnvelopeAttack(envelope_index, milliseconds) => {
                    match envelope_index {
                        ENVELOPE_INDEX_AMP => {
                            set_envelope_attack_time(
                                &module_parameters.envelopes[EnvelopeIndex::Amp as usize],
                                milliseconds,
                            );
                        }
                        ENVELOPE_INDEX_FILTER => set_envelope_attack_time(
                            &module_parameters.envelopes[EnvelopeIndex::Filter as usize],
                            milliseconds,
                        ),
                        _ => {
                            log::warn!(
                                "start_ui_event_listener():SynthesizerUpdateEvents::FilterEnvelopeAttack: Invalid \
                                Envelope index: {envelope_index}"
                            );
                        }
                    }
                }
                SynthesizerUpdateEvents::FilterEnvelopeDecay(envelope_index, milliseconds) => {
                    match envelope_index {
                        ENVELOPE_INDEX_AMP => {
                            set_envelope_decay_time(
                                &module_parameters.envelopes[EnvelopeIndex::Amp as usize],
                                milliseconds,
                            );
                        }
                        ENVELOPE_INDEX_FILTER => set_envelope_decay_time(
                            &module_parameters.envelopes[EnvelopeIndex::Filter as usize],
                            milliseconds,
                        ),
                        _ => {
                            log::warn!(
                                "start_ui_event_listener():SynthesizerUpdateEvents::FilterEnvelopeDecay: Invalid \
                                Envelope index: {envelope_index}"
                            );
                        }
                    }
                }
                SynthesizerUpdateEvents::FilterEnvelopeSustain(envelope_index, level) => {
                    match envelope_index {
                        ENVELOPE_INDEX_AMP => {
                            set_envelope_sustain_level(
                                &module_parameters.envelopes[EnvelopeIndex::Amp as usize],
                                level,
                            );
                        }
                        ENVELOPE_INDEX_FILTER => {
                            set_envelope_sustain_level(
                                &module_parameters.envelopes[EnvelopeIndex::Filter as usize],
                                level,
                            );
                        }
                        _ => {
                            log::warn!(
                                "start_ui_event_listener():SynthesizerUpdateEvents::FilterEnvelopeSustain: Invalid \
                                Envelope index: {envelope_index}"
                            );
                        }
                    }
                }
                SynthesizerUpdateEvents::FilterEnvelopeRelease(envelope_index, milliseconds) => {
                    match envelope_index {
                        ENVELOPE_INDEX_AMP => {
                            set_envelope_release_time(
                                &module_parameters.envelopes[EnvelopeIndex::Amp as usize],
                                milliseconds,
                            );
                        }
                        ENVELOPE_INDEX_FILTER => set_envelope_release_time(
                            &module_parameters.envelopes[EnvelopeIndex::Filter as usize],
                            milliseconds,
                        ),
                        _ => {
                            log::warn!(
                                "start_ui_event_listener():SynthesizerUpdateEvents::FilterEnvelopeRelease: Invalid \
                                Envelope index: {envelope_index}"
                            );
                        }
                    }
                }
                SynthesizerUpdateEvents::FilterEnvelopeInvert(envelope_index, is_inverted) => {
                    match envelope_index {
                        ENVELOPE_INDEX_AMP => set_envelope_inverted(
                            &module_parameters.envelopes[EnvelopeIndex::Amp as usize],
                            f32::from(is_inverted),
                        ),
                        ENVELOPE_INDEX_FILTER => set_envelope_inverted(
                            &module_parameters.envelopes[EnvelopeIndex::Filter as usize],
                            f32::from(is_inverted),
                        ),
                        _ => {
                            log::warn!(
                                "start_ui_event_listener():SynthesizerUpdateEvents::FilterEnvelopeInvert: Invalid \
                                Envelope index: {envelope_index}"
                            );
                        }
                    }
                }
                SynthesizerUpdateEvents::LfoFrequency(lfo_index, frequency) => {
                    let frequency = match lfo_index {
                        LFO_INDEX_MOD_WHEEL => set_lfo_frequency(
                            &module_parameters.lfos[LFOIndex::ModWheel as usize],
                            frequency,
                        ),
                        LFO_INDEX_FILTER => set_lfo_frequency(
                            &module_parameters.lfos[LFOIndex::Filter as usize],
                            frequency,
                        ),
                        _ => {
                            log::warn!(
                                "start_ui_event_listener():SynthesizerUpdateEvents::LfoFrequency: Invalid LFO index: {lfo_index}"
                            );
                            return;
                        }
                    };
                    ui_update_sender.send(UIUpdates::LFOFrequencyDisplay(lfo_index, frequency)).expect
                    ("start_update_event_listener(): Failed to send oscillator fine-tune display value to the UI.");
                }
                SynthesizerUpdateEvents::LfoShapeIndex(lfo_index, wave_shape_index) => {
                    match lfo_index {
                        LFO_INDEX_MOD_WHEEL => {
                            module_parameters.lfos[LFOIndex::ModWheel as usize]
                                .wave_shape
                                .store(wave_shape_index as u8, Relaxed);
                        }
                        LFO_INDEX_FILTER => {
                            module_parameters.lfos[LFOIndex::Filter as usize]
                                .wave_shape
                                .store(wave_shape_index as u8, Relaxed);
                        }
                        _ => {
                            log::warn!(
                                "start_ui_event_listener():SynthesizerUpdateEvents::LfoShapeIndex: Invalid\
                                 LFO index: {lfo_index}"
                            );
                        }
                    }
                }
                SynthesizerUpdateEvents::LfoPhase(lfo_index, phase) => {
                    match lfo_index {
                        LFO_INDEX_MOD_WHEEL => {
                            set_lfo_phase(
                                &module_parameters.lfos[LFOIndex::ModWheel as usize],
                                phase,
                            );
                        }
                        LFO_INDEX_FILTER => {
                            set_lfo_phase(
                                &module_parameters.lfos[LFOIndex::Filter as usize],
                                phase,
                            );
                        }
                        _ => {
                            log::warn!(
                                "start_ui_event_listener():SynthesizerUpdateEvents::LfoPhase: Invalid LFO index: {lfo_index}"
                            );
                        }
                    }
                    ui_update_sender
                        .send(UIUpdates::LFOPhase(lfo_index, phase))
                        .expect(
                            "start_update_event_listener(): \
                    Failed to send the lfo reset to the UI.",
                        );
                }
                SynthesizerUpdateEvents::LfoPhaseReset(lfo_index) => {
                    match lfo_index {
                        LFO_INDEX_MOD_WHEEL => {
                            set_lfo_phase_reset(
                                &module_parameters.lfos[LFOIndex::ModWheel as usize],
                            );
                        }
                        LFO_INDEX_FILTER => {
                            set_lfo_phase_reset(&module_parameters.lfos[LFOIndex::Filter as usize]);
                        }
                        _ => {
                            log::warn!(
                                "start_ui_event_listener():SynthesizerUpdateEvents::LfoPhaseReset: Invalid LFO index: {lfo_index}"
                            );
                        }
                    }
                    ui_update_sender
                        .send(UIUpdates::LFOPhase(lfo_index, DEFAULT_LFO_PHASE))
                        .expect(
                            "start_update_event_listener(): \
                    Failed to send the lfo reset to the UI.",
                        );
                }
                SynthesizerUpdateEvents::PortamentoEnabled(is_enabled) => {
                    set_portamento_enabled(&module_parameters.oscillators, f32::from(is_enabled));
                }
                SynthesizerUpdateEvents::PortamentoTime(milliseconds) => {
                    set_portamento_time(&module_parameters.oscillators, milliseconds);
                }
                SynthesizerUpdateEvents::PitchBendRange(range) => {
                    set_pitch_bend_range(&module_parameters.keyboard, range);
                }
                SynthesizerUpdateEvents::VelocityCurve(curve) => {
                    set_velocity_curve(&mut current_note, curve);
                }
                SynthesizerUpdateEvents::HardSyncEnabled(is_enabled) => {
                    set_oscillator_hard_sync(&module_parameters.oscillators, f32::from(is_enabled));
                }
                SynthesizerUpdateEvents::KeySyncEnabled(is_enabled) => {
                    set_oscillator_key_sync(&module_parameters.oscillators, f32::from(is_enabled));
                }
                SynthesizerUpdateEvents::OutputBalance(balance) => {
                    set_output_balance(&module_parameters.mixer, balance);
                }
                SynthesizerUpdateEvents::OutputLevel(level) => {
                    set_output_level(&module_parameters.mixer, level);
                }
                SynthesizerUpdateEvents::OutputMute(is_muted) => {
                    set_output_mute(&module_parameters.mixer, f32::from(is_muted));
                }
                SynthesizerUpdateEvents::OscillatorMixerBalance(oscillator_index, balance) => {
                    if let Some(oscillator) = OscillatorIndex::from_i32(oscillator_index) {
                        set_oscillator_balance(&module_parameters.mixer, oscillator, balance);
                    } else {
                        log::warn!(
                            "start_ui_event_listener():SynthesizerUpdateEvents::OscillatorMixerBalance: Invalid oscillator index: {oscillator_index}"
                        );
                    }
                }
                SynthesizerUpdateEvents::OscillatorMixerLevel(oscillator_index, level) => {
                    if let Some(oscillator) = OscillatorIndex::from_i32(oscillator_index) {
                        set_oscillator_level(&module_parameters.mixer, oscillator, level);
                    } else {
                        log::warn!(
                            "start_ui_event_listener():SynthesizerUpdateEvents::OscillatorMixerLevel: Invalid oscillator index: {oscillator_index}"
                        );
                    }
                }
                SynthesizerUpdateEvents::OscillatorMixerMute(oscillator_index, is_muted) => {
                    if let Some(oscillator) = OscillatorIndex::from_i32(oscillator_index) {
                        set_oscillator_mute(
                            &module_parameters.mixer,
                            oscillator,
                            f32::from(is_muted),
                        );
                    } else {
                        log::warn!(
                            "start_ui_event_listener():SynthesizerUpdateEvents::OscillatorMixerMute: Invalid oscillator index: {oscillator_index}"
                        );
                    }
                }
                SynthesizerUpdateEvents::EffectEnabled(effect_index, is_enabled) => {
                    if let Some(effect) = EffectIndex::from_i32(effect_index) {
                        set_effect_is_enabled(&module_parameters.effects, effect, is_enabled);
                    } else {
                        log::warn!(
                            "start_ui_event_listener():SynthesizerUpdateEvents::EffectEnabled: Invalid effect index: \
                            {effect_index}"
                        );
                    }
                }
                SynthesizerUpdateEvents::EffectParameters(effect_index, parameter_index, value) => {
                    if let Some(effect) = EffectIndex::from_i32(effect_index) {
                        set_effect_parameter(
                            &module_parameters.effects,
                            effect,
                            parameter_index,
                            value,
                        );
                    } else {
                        log::warn!(
                            "start_ui_event_listener():SynthesizerUpdateEvents::EffectEnabled: Invalid effect index: \
                            {effect_index}"
                        );
                    }
                }
            }
        }
    });
}
