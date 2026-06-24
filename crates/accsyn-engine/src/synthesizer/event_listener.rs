use crate::modules::effects::EffectIndex;
use crate::modules::lfo::DEFAULT_LFO_PHASE;
use crate::modules::oscillator::constants::OSCILLATOR_WAVESHAPE_PARAMETER_DEFAULTS;
use crate::synthesizer::ModuleParameters;
use crate::synthesizer::clock::bpm_from_thirty_second_note_duration;
use crate::synthesizer::constants::{
    ENVELOPE_INDEX_AMP, ENVELOPE_INDEX_FILTER, ENVELOPE_INDEX_PITCH, LFO_INDEX_FILTER,
    LFO_INDEX_MOD_WHEEL, PATCH_DELETE_FAILURE, PATCH_DELETE_FILE_DOES_NOT_EXIST,
    PATCH_DELETE_SUCCESS, PATCH_SAVE_ALREADY_EXISTS, PATCH_SAVE_FAILURE, PATCH_SAVE_SUCCESS,
};
use crate::synthesizer::midi_value_converters::bool_to_normal_value;
use crate::synthesizer::patches::{Patches, PatchesError, get_module_parameters_from_patch_index};
use crate::synthesizer::set_parameters::{
    set_effect_is_enabled, set_effect_parameter, set_envelope_amount, set_envelope_attack_time,
    set_envelope_decay_time, set_envelope_inverted, set_envelope_release_time,
    set_envelope_sustain_level, set_envelope_sustain_pedal, set_filter_cutoff, set_filter_poles,
    set_filter_resonance, set_key_tracking_amount, set_lfo_clock_sync, set_lfo_frequency,
    set_lfo_key_sync, set_lfo_phase, set_lfo_phase_reset, set_lfo_range,
    set_module_parameters_from_preset, set_oscillator_balance, set_oscillator_clip_boost,
    set_oscillator_course_tune, set_oscillator_fine_tune, set_oscillator_hard_sync,
    set_oscillator_key_sync, set_oscillator_level, set_oscillator_mute,
    set_oscillator_pitch_envelope_amount, set_oscillator_polarity, set_oscillator_shape_parameter1,
    set_oscillator_shape_parameter2, set_output_balance, set_output_level, set_output_mute,
    set_pitch_bend_range, set_portamento_enabled, set_portamento_time, set_velocity_curve,
};
use accsyn_core::casting::i32_to_u8_clamped;
use accsyn_core::synth_events::{
    EnvelopeIndex, LFO_SYNC_INTERVAL_NAMES, LFOIndex, LfoSyncInterval, OscillatorIndex,
    SynthesizerUpdateEvents,
};
use accsyn_core::ui_events::UIUpdates;
use crossbeam_channel::{Receiver, Sender};
use std::sync::atomic::Ordering::Relaxed;
use std::sync::{Arc, Mutex, PoisonError};
use std::thread;
use std::time::Instant;

#[allow(clippy::too_many_lines)]
pub fn start_update_event_listener(
    ui_update_receiver: Receiver<SynthesizerUpdateEvents>,
    module_parameters: Arc<ModuleParameters>,
    patches: Arc<Mutex<Patches>>,
    ui_update_sender: Sender<UIUpdates>,
) {
    let mut last_thirty_second_note_time_now: Option<Instant> = None;

    thread::spawn(move || {
        log::debug!(target: "synthesizer::events", "start_update_event_listener(): spawned thread to receive UI events");

        while let Ok(event) = ui_update_receiver.recv() {
            match event {
                SynthesizerUpdateEvents::WaveShapeIndex(oscillator_index, wave_shape_index) => {
                    match usize::try_from(oscillator_index) {
                        Ok(idx) if idx < module_parameters.oscillators.len() => {
                            module_parameters.oscillators[idx]
                                .wave_shape_index
                                .store(i32_to_u8_clamped(wave_shape_index), Relaxed);

                            if let Err(e) = ui_update_sender.send(UIUpdates::OscillatorWaveShape(
                                oscillator_index,
                                wave_shape_index,
                            )) {
                                log::error!(target: "synthesizer::event_listener", "Failed to send oscillator \
                                Wave Shape Update to the UI: {e}");
                            }
                            #[allow(clippy::cast_sign_loss)]
                            // Oscillator_index is always non-negative
                            let shape_parameter1 = OSCILLATOR_WAVESHAPE_PARAMETER_DEFAULTS
                                [wave_shape_index as usize]
                                .0;
                            module_parameters.oscillators[idx]
                                .shape_parameter1
                                .store(shape_parameter1);

                            if let Err(e) = ui_update_sender.send(UIUpdates::OscillatorParameter1(
                                oscillator_index,
                                shape_parameter1,
                            )) {
                                log::error!(target: "synthesizer::event_listener", "Failed to send oscillator \
                                Shape Parameter 1 Update to the UI: {e}");
                            }

                            #[allow(clippy::cast_sign_loss)]
                            // Oscillator_index is always non-negative
                            let shape_parameter2 = OSCILLATOR_WAVESHAPE_PARAMETER_DEFAULTS
                                [wave_shape_index as usize]
                                .1;
                            module_parameters.oscillators[idx]
                                .shape_parameter2
                                .store(shape_parameter2);

                            if let Err(e) = ui_update_sender.send(UIUpdates::OscillatorParameter2(
                                oscillator_index,
                                shape_parameter2,
                            )) {
                                log::error!(target: "synthesizer::event_listener", "Failed to send oscillator \
                                Shape Parameter 2 Update to the UI: {e}");
                            }
                        }
                        _ => {
                            log::warn!(
                                target: "synthesizer::events",
                                "start_update_event_listener(): Invalid oscillator index: {oscillator_index}"
                            );
                        }
                    }
                }
                SynthesizerUpdateEvents::CourseTune(oscillator_index, course_tune) => {
                    match usize::try_from(oscillator_index) {
                        Ok(idx) if idx < module_parameters.oscillators.len() => {
                            set_oscillator_course_tune(
                                &module_parameters.oscillators[idx],
                                course_tune,
                            );
                        }
                        _ => {
                            log::warn!(
                                target: "synthesizer::events",
                                "start_update_event_listener(): Invalid oscillator index: {oscillator_index}"
                            );
                        }
                    }
                }
                SynthesizerUpdateEvents::FineTune(oscillator_index, fine_tune) => {
                    match usize::try_from(oscillator_index) {
                        Ok(idx) if idx < module_parameters.oscillators.len() => {
                            let cents = set_oscillator_fine_tune(
                                &module_parameters.oscillators[idx],
                                fine_tune,
                            );

                            if let Err(e) =
                                ui_update_sender.send(UIUpdates::OscillatorFineTuneCents(
                                    oscillator_index,
                                    i32::from(cents),
                                ))
                            {
                                log::error!(target: "synthesizer::event_listener", "Failed to send oscillator fine-tune display value to the UI: {e}");
                            }
                        }
                        _ => {
                            log::warn!(
                                target: "synthesizer::events",
                                "start_update_event_listener(): Invalid oscillator index: {oscillator_index}"
                            );
                        }
                    }
                }
                SynthesizerUpdateEvents::ClipperBoost(oscillator_index, boost) => {
                    match usize::try_from(oscillator_index) {
                        Ok(index) if index < module_parameters.oscillators.len() => {
                            set_oscillator_clip_boost(&module_parameters.oscillators[index], boost);

                            if let Err(e) = ui_update_sender
                                .send(UIUpdates::OscillatorClipperBoost(oscillator_index, boost))
                            {
                                log::error!(target: "synthesizer::event_listener", "Failed to send oscillator clipper boost display value to the UI: {e}");
                            }
                        }
                        _ => {
                            log::warn!(

                                target: "synthesizer::events",
                                "start_update_event_listener(): Invalid oscillator index: {oscillator_index}"
                            );
                        }
                    }
                }
                SynthesizerUpdateEvents::PitchEnvelopeAmount(oscillator_index, amount) => {
                    match usize::try_from(oscillator_index) {
                        Ok(index) if index < module_parameters.oscillators.len() => {
                            set_oscillator_pitch_envelope_amount(
                                &module_parameters.oscillators[index],
                                amount,
                            );
                        }
                        _ => {
                            log::warn!(
                                target: "synthesizer::events",
                                "start_update_event_listener(): Invalid oscillator index: {oscillator_index}"
                            );
                        }
                    }
                }
                SynthesizerUpdateEvents::Parameter1(oscillator_index, parameter) => {
                    match usize::try_from(oscillator_index) {
                        Ok(idx) if idx < module_parameters.oscillators.len() => {
                            set_oscillator_shape_parameter1(
                                &module_parameters.oscillators[idx],
                                parameter,
                            );

                            if let Err(e) = ui_update_sender
                                .send(UIUpdates::OscillatorParameter1(oscillator_index, parameter))
                            {
                                log::error!(target: "synthesizer::event_listener", "Failed to send oscillator shape \
                                parameter 1 display value to the UI: {e}");
                            }
                        }
                        _ => {
                            log::warn!(
                                target: "synthesizer::events",
                                "start_update_event_listener(): Invalid oscillator index: {oscillator_index}"
                            );
                        }
                    }
                }
                SynthesizerUpdateEvents::Parameter2(oscillator_index, parameter) => {
                    match usize::try_from(oscillator_index) {
                        Ok(idx) if idx < module_parameters.oscillators.len() => {
                            set_oscillator_shape_parameter2(
                                &module_parameters.oscillators[idx],
                                parameter,
                            );

                            if let Err(e) = ui_update_sender
                                .send(UIUpdates::OscillatorParameter2(oscillator_index, parameter))
                            {
                                log::error!(target: "synthesizer::event_listener", "Failed to send oscillator shape \
                                parameter 2 display value to the UI: {e}");
                            }
                        }
                        _ => {
                            log::warn!(
                                target: "synthesizer::events",
                                "start_update_event_listener(): Invalid oscillator index: {oscillator_index}"
                            );
                        }
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

                    if let Err(e) = ui_update_sender.send(UIUpdates::FilterPoles(poles)) {
                        log::error!(target: "synthesizer::event_listener", "Failed to send filter poles \
                        display value to the UI: {e}");
                    }
                }
                SynthesizerUpdateEvents::FilterKeyTrackingAmount(amount) => {
                    set_key_tracking_amount(&module_parameters.filter, amount);

                    if let Err(e) = ui_update_sender.send(UIUpdates::FilterKeyTracking(amount)) {
                        log::error!(target: "synthesizer::event_listener", "Failed to send filter key-tracking \
                        display value to the UI: {e}");
                    }
                }
                SynthesizerUpdateEvents::FilterEnvelopeAmount(amount) => {
                    set_envelope_amount(
                        &module_parameters.envelopes[EnvelopeIndex::Filter as usize],
                        amount,
                    );

                    if let Err(e) = ui_update_sender.send(UIUpdates::FilterEnvelopeAmount(amount)) {
                        log::error!(target: "synthesizer::event_listener", "Failed to send filter envelope amount \
                        display value to the UI: {e}");
                    }
                }
                SynthesizerUpdateEvents::FilterLfoAmount(amount) => {
                    set_lfo_range(&module_parameters.lfos[LFOIndex::Filter as usize], amount);

                    if let Err(e) = ui_update_sender.send(UIUpdates::FilterLFOAmount(amount)) {
                        log::error!(target: "synthesizer::event_listener", "Failed to send filter LFO amount \
                        display value to the UI: {e}");
                    }
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
                        ENVELOPE_INDEX_PITCH => set_envelope_attack_time(
                            &module_parameters.envelopes[EnvelopeIndex::Pitch as usize],
                            milliseconds,
                        ),
                        _ => {
                            log::warn!(
                                target: "synthesizer::events",
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
                        ENVELOPE_INDEX_PITCH => set_envelope_decay_time(
                            &module_parameters.envelopes[EnvelopeIndex::Pitch as usize],
                            milliseconds,
                        ),
                        _ => {
                            log::warn!(
                                target: "synthesizer::events",
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
                        ENVELOPE_INDEX_PITCH => set_envelope_sustain_level(
                            &module_parameters.envelopes[EnvelopeIndex::Pitch as usize],
                            level,
                        ),
                        _ => {
                            log::warn!(
                                target: "synthesizer::events",
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
                        ENVELOPE_INDEX_PITCH => set_envelope_release_time(
                            &module_parameters.envelopes[EnvelopeIndex::Pitch as usize],
                            milliseconds,
                        ),
                        _ => {
                            log::warn!(
                                target: "synthesizer::events",
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
                        ENVELOPE_INDEX_PITCH => set_envelope_inverted(
                            &module_parameters.envelopes[EnvelopeIndex::Pitch as usize],
                            f32::from(is_inverted),
                        ),
                        _ => {
                            log::warn!(
                                target: "synthesizer::events",
                                "start_ui_event_listener():SynthesizerUpdateEvents::FilterEnvelopeInvert: Invalid \
                                Envelope index: {envelope_index}"
                            );
                        }
                    }
                }
                SynthesizerUpdateEvents::LfoFrequency(lfo_index, normal_value) => {
                    if LFOIndex::from_i32(lfo_index).is_none() {
                        log::warn!(
                            target: "synthesizer::events",
                            "start_ui_event_listener():SynthesizerUpdateEvents::LfoFrequency: Invalid LFO index: {lfo_index}"
                        );
                        continue;
                    }

                    let lfo_sync_interval_index = LfoSyncInterval::from_normal_value(normal_value);
                    let lfo_sync_interval =
                        LFO_SYNC_INTERVAL_NAMES[lfo_sync_interval_index as usize];

                    if let Err(e) = ui_update_sender.send(UIUpdates::LFOClockSyncIntervalDisplay(
                        lfo_index,
                        lfo_sync_interval.to_string(),
                    )) {
                        log::error!(target: "synthesizer::event_listener", "Failed to send LFO clock sync interval \
                            display value to the UI: {e}");
                    }

                    module_parameters.lfos[lfo_index.unsigned_abs() as usize]
                        .thirty_second_notes
                        .store(lfo_sync_interval_index.to_thirty_second_notes(), Relaxed);

                    let display_frequency = set_lfo_frequency(
                        &module_parameters.lfos[lfo_index.unsigned_abs() as usize],
                        normal_value,
                    );

                    if let Err(e) = ui_update_sender
                        .send(UIUpdates::LFOFrequencyDisplay(lfo_index, display_frequency))
                    {
                        log::error!(target: "synthesizer::event_listener", "Failed to send LFO frequency display value to the UI: {e}");
                    }
                }
                SynthesizerUpdateEvents::LfoShapeIndex(lfo_index, wave_shape_index) => {
                    match lfo_index {
                        LFO_INDEX_MOD_WHEEL => {
                            module_parameters.lfos[LFOIndex::ModWheel as usize]
                                .wave_shape
                                .store(i32_to_u8_clamped(wave_shape_index), Relaxed);
                        }
                        LFO_INDEX_FILTER => {
                            module_parameters.lfos[LFOIndex::Filter as usize]
                                .wave_shape
                                .store(i32_to_u8_clamped(wave_shape_index), Relaxed);
                        }
                        _ => {
                            log::warn!(
                                target: "synthesizer::events",
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
                                target: "synthesizer::events",
                                "start_ui_event_listener():SynthesizerUpdateEvents::LfoPhase: Invalid LFO index: {lfo_index}"
                            );
                        }
                    }
                    if let Err(e) = ui_update_sender.send(UIUpdates::LFOPhase(lfo_index, phase)) {
                        log::error!(target: "synthesizer::event_listener", "Failed to send LFO phase to the UI: {e}");
                    }
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
                                target: "synthesizer::events",
                                "start_ui_event_listener():SynthesizerUpdateEvents::LfoPhaseReset: Invalid LFO index: {lfo_index}"
                            );
                        }
                    }
                    if let Err(e) =
                        ui_update_sender.send(UIUpdates::LFOPhase(lfo_index, DEFAULT_LFO_PHASE))
                    {
                        log::error!(target: "synthesizer::event_listener", "Failed to send LFO phase reset to the UI: {e}");
                    }
                }
                SynthesizerUpdateEvents::LfoClockSyncEnabled(lfo_index, is_enabled) => {
                    match lfo_index {
                        LFO_INDEX_MOD_WHEEL => {
                            set_lfo_clock_sync(
                                &module_parameters.lfos[LFOIndex::ModWheel as usize],
                                is_enabled,
                            );
                        }
                        LFO_INDEX_FILTER => {
                            set_lfo_clock_sync(
                                &module_parameters.lfos[LFOIndex::Filter as usize],
                                is_enabled,
                            );
                        }
                        _ => {
                            log::warn!(
                                target: "synthesizer::events",
                                "start_ui_event_listener():SynthesizerUpdateEvents::LfoClockSyncEnabled: Invalid LFO index: \
                                {lfo_index}"
                            );
                        }
                    }
                }
                SynthesizerUpdateEvents::LfoKeySyncEnabled(lfo_index, is_enabled) => {
                    match lfo_index {
                        LFO_INDEX_MOD_WHEEL => {
                            set_lfo_key_sync(
                                &module_parameters.lfos[LFOIndex::ModWheel as usize],
                                is_enabled,
                            );
                        }
                        LFO_INDEX_FILTER => {
                            set_lfo_key_sync(
                                &module_parameters.lfos[LFOIndex::Filter as usize],
                                is_enabled,
                            );
                        }
                        _ => {
                            log::warn!(
                                target: "synthesizer::events",
                                "start_ui_event_listener():SynthesizerUpdateEvents::LfoKeySyncEnabled: Invalid LFO index: \
                                {lfo_index}"
                            );
                        }
                    }
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
                    set_velocity_curve(&module_parameters.keyboard, curve);
                }
                SynthesizerUpdateEvents::SustainPedal(is_enabled) => {
                    set_envelope_sustain_pedal(
                        &module_parameters.envelopes,
                        bool_to_normal_value(is_enabled),
                    );
                }
                SynthesizerUpdateEvents::HardSyncEnabled(is_enabled) => {
                    set_oscillator_hard_sync(&module_parameters.oscillators, f32::from(is_enabled));
                }
                SynthesizerUpdateEvents::KeySyncEnabled(is_enabled) => {
                    set_oscillator_key_sync(&module_parameters.oscillators, f32::from(is_enabled));
                }
                SynthesizerUpdateEvents::PolarityFlipped(is_flipped) => {
                    set_oscillator_polarity(&module_parameters.keyboard, f32::from(is_flipped));
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
                            target: "synthesizer::events",
                            "start_ui_event_listener():SynthesizerUpdateEvents::OscillatorMixerBalance: Invalid oscillator index: {oscillator_index}"
                        );
                    }
                }
                SynthesizerUpdateEvents::OscillatorMixerLevel(oscillator_index, level) => {
                    if let Some(oscillator) = OscillatorIndex::from_i32(oscillator_index) {
                        set_oscillator_level(&module_parameters.mixer, oscillator, level);
                    } else {
                        log::warn!(
                            target: "synthesizer::events",
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
                            target: "synthesizer::events",
                            "start_ui_event_listener():SynthesizerUpdateEvents::OscillatorMixerMute: Invalid oscillator index: {oscillator_index}"
                        );
                    }
                }
                SynthesizerUpdateEvents::EffectEnabled(effect_index, is_enabled) => {
                    if let Some(effect) = EffectIndex::from_i32(effect_index) {
                        set_effect_is_enabled(&module_parameters.effects, effect, is_enabled);
                    } else {
                        log::warn!(
                            target: "synthesizer::events",
                            "start_ui_event_listener():SynthesizerUpdateEvents::EffectEnabled: Invalid effect index: \
                            {effect_index}"
                        );
                    }
                }
                SynthesizerUpdateEvents::EffectParameterValues(
                    effect_index,
                    parameter_index,
                    value,
                ) => {
                    if let Some(effect) = EffectIndex::from_i32(effect_index) {
                        set_effect_parameter(
                            &module_parameters.effects,
                            effect,
                            parameter_index,
                            value,
                        );
                    } else {
                        log::warn!(
                            target: "synthesizer::events",
                            "start_ui_event_listener():SynthesizerUpdateEvents::EffectEnabled: Invalid effect index: \
                            {effect_index}"
                        );
                    }
                }
                SynthesizerUpdateEvents::PatchChanged(preset_index) => {
                    let thread_patches = patches.lock().unwrap_or_else(PoisonError::into_inner);
                    let patch_list = thread_patches.patch_list();

                    let Some(preset_idx) = usize::try_from(preset_index)
                        .ok()
                        .filter(|&idx| idx < patch_list.all_names().len())
                    else {
                        log::warn!(target: "synthesizer::event_listener", "Invalid preset index: {preset_index}");
                        continue;
                    };

                    let patch = match get_module_parameters_from_patch_index(
                        preset_idx,
                        &patch_list,
                    ) {
                        Ok(preset) => preset,
                        Err(e) => {
                            log::error!(target: "synthesizer::event_listener", "Failed to get preset from index {preset_index}: {e}");
                            continue;
                        }
                    };

                    set_module_parameters_from_preset(&module_parameters, &patch);
                    log::info!(target: "synthesizer::event_listener", "Preset changed to index {preset_index}");
                }
                SynthesizerUpdateEvents::PatchSaved(patch_name) => {
                    let mut thread_patches = patches.lock().unwrap_or_else(PoisonError::into_inner);

                    let save_status = match thread_patches
                        .save_patch(&patch_name, &module_parameters)
                    {
                        Ok(()) => {
                            log::info!(target: "synthesizer::event_listener", "Saved patch: {patch_name}");
                            (true, PATCH_SAVE_SUCCESS.to_string())
                        }
                        Err(err) => {
                            if err == PatchesError::FileAlreadyExists {
                                log::error!(target: "synthesizer::event_listener", "Patch name already exists: {patch_name}: {err}");
                                (false, PATCH_SAVE_ALREADY_EXISTS.to_string())
                            } else {
                                log::error!(target: "synthesizer::event_listener", "Could not save patch: {patch_name}: {err}");
                                (false, PATCH_SAVE_FAILURE.to_string())
                            }
                        }
                    };

                    if let Err(e) =
                        ui_update_sender.send(UIUpdates::PatchSaveStatus(save_status.clone()))
                    {
                        log::error!(target: "synthesizer::event_listener", "Failed to send patch save status to the UI: {e}");
                        continue;
                    }

                    if save_status.0 {
                        let patch_list = thread_patches.patch_list().all_names();

                        if let Err(e) = ui_update_sender.send(UIUpdates::PatchList(patch_list)) {
                            log::error!(target: "synthesizer::event_listener", "Failed to send new patch list to the UI: {e}");
                        }

                        let user_patch_list = thread_patches.user_patch_names();
                        if let Err(e) =
                            ui_update_sender.send(UIUpdates::UserPatchList(user_patch_list))
                        {
                            log::error!(target: "synthesizer::event_listener", "Failed to send new patch list to the UI: {e}");
                        }
                    }
                }
                SynthesizerUpdateEvents::PatchDeleted(patch_name) => {
                    let mut thread_patches = patches.lock().unwrap_or_else(PoisonError::into_inner);

                    let delete_status = match thread_patches
                        .delete_patch_by_name(patch_name.clone())
                    {
                        Ok(()) => {
                            log::info!(target: "synthesizer::event_listener", "Deleted patch: {patch_name}");
                            (true, PATCH_DELETE_SUCCESS.to_string())
                        }
                        Err(err)
                            if err == PatchesError::PatchNameDoesNotExist(patch_name.clone()) =>
                        {
                            log::warn!(target: "synthesizer::event_listener", "Patch file at {patch_name} does not exist");
                            (false, PATCH_DELETE_FILE_DOES_NOT_EXIST.to_string())
                        }
                        Err(err) => {
                            log::warn!(target: "synthesizer::event_listener", "Could not delete patch: {patch_name} -\
                            {err}");
                            (false, PATCH_DELETE_FAILURE.to_string())
                        }
                    };

                    if let Err(e) =
                        ui_update_sender.send(UIUpdates::PatchDeleteStatus(delete_status.clone()))
                    {
                        log::error!(target: "synthesizer::event_listener", "Failed to send patch save status to the UI: {e}");
                        continue;
                    }

                    let patch_list = thread_patches.patch_list().all_names();
                    if let Err(e) = ui_update_sender.send(UIUpdates::PatchList(patch_list)) {
                        log::error!(target: "synthesizer::event_listener", "Failed to send new patch list to the UI: {e}");
                    }

                    let user_patch_list = thread_patches.user_patch_names();
                    if let Err(e) = ui_update_sender.send(UIUpdates::UserPatchList(user_patch_list))
                    {
                        log::error!(target: "synthesizer::event_listener", "Failed to send new patch list to the UI: {e}");
                    }
                }
                SynthesizerUpdateEvents::ThirtySecondNote => {
                    let Some(last_thirty_second_note) = last_thirty_second_note_time_now else {
                        last_thirty_second_note_time_now = Some(Instant::now());
                        continue;
                    };

                    let current_note_time = Instant::now();
                    last_thirty_second_note_time_now = Some(current_note_time);
                    let thirty_second_note_duration = current_note_time - last_thirty_second_note;

                    let new_bpm = bpm_from_thirty_second_note_duration(thirty_second_note_duration);
                    module_parameters.clock.bpm.store(new_bpm, Relaxed);

                    if let Err(e) = ui_update_sender.send(UIUpdates::MidiClock(i32::from(new_bpm)))
                    {
                        log::error!(target: "synthesizer::event_listener", "Failed to send midi \
                                clock bpm update to the UI: {e}");
                    }

                    module_parameters
                        .lfos
                        .iter()
                        .filter(|lfo| lfo.clock_synced.load(Relaxed))
                        .for_each(|lfo| {
                            let thirty_second_notes_per_interval =
                                lfo.thirty_second_notes.load(Relaxed);
                            let new_period = f64::from(thirty_second_notes_per_interval)
                                * thirty_second_note_duration.as_secs_f64();
                            let new_frequency = 1.0 / new_period;

                            // LFO frequencies live in roughly 0.1–50 Hz, which fits will within an f32
                            #[allow(clippy::cast_possible_truncation)]
                            lfo.synced_frequency.store(new_frequency as f32);
                            lfo.sync_triggered.store(true, Relaxed);
                        });
                }
            }
        }
    });
}
