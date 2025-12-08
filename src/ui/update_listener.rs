use crate::AccidentalSynth;
use crate::synthesizer::midi_value_converters::exponential_curve_lfo_frequency_from_normal_value;
use crate::ui::constants::MAX_PHASE_VALUE;
use crate::ui::{EnvelopeIndex, EnvelopeStage, LFOIndex, ParameterValues, UIUpdates, set_audio_device_channel_indexes, set_audio_device_channel_list, set_audio_device_index, set_audio_device_list, set_envelope_inverted, set_envelope_stage_value, set_lfo_frequency, set_lfo_frequency_display, set_lfo_phase, set_lfo_phase_display, set_lfo_wave_shape, set_midi_channel_index, set_midi_port_index, set_midi_port_list, set_oscillator_clipper_boost, set_oscillator_course_tune, set_oscillator_fine_tune, set_oscillator_fine_tune_display, set_oscillator_parameter1, set_oscillator_parameter2, set_oscillator_wave_shape, set_filter_cutoff_value, set_filter_resonance_value};
use crossbeam_channel::Receiver;
use slint::Weak;
use std::sync::{Arc, Mutex};
use std::thread;

pub fn start_ui_update_listener(
    ui_update_receiver: Receiver<UIUpdates>,
    ui_weak: &Weak<AccidentalSynth>,
    parameter_values: Arc<Mutex<ParameterValues>>,
) {

    let values = parameter_values.clone();
    let ui_weak_thread = ui_weak.clone();

    thread::spawn(move || {
        log::debug!("start_ui_update_listener(): spawned thread to receive ui update events");
        let mut values = values
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        while let Ok(update) = ui_update_receiver.recv() {
            match update {
                UIUpdates::MidiPortList(port_list) => {
                    set_midi_port_list(&ui_weak_thread, &mut values.midi_port, port_list);
                }
                UIUpdates::MidiPortIndex(index) => {
                    set_midi_port_index(&ui_weak_thread, &mut values.midi_port, index);
                }
                UIUpdates::MidiChannelIndex(index) => {
                    set_midi_channel_index(&ui_weak_thread, &mut values.midi_port, index);
                }
                UIUpdates::AudioDeviceList(device_list) => {
                    set_audio_device_list(&ui_weak_thread, &mut values.audio_device, device_list);
                }
                UIUpdates::AudioDeviceIndex(index) => {
                    set_audio_device_index(&ui_weak_thread, &mut values.audio_device, index);
                }
                UIUpdates::AudioDeviceChannelCount(count) => {
                    set_audio_device_channel_list(&ui_weak_thread, &mut values.audio_device, count);
                }
                UIUpdates::AudioDeviceChannelIndexes { left, right } => {
                    set_audio_device_channel_indexes(
                        &ui_weak_thread,
                        &mut values.audio_device,
                        left,
                        right,
                    );
                }
                UIUpdates::OscillatorWaveShape(oscillator_index, shape_index) => {
                    set_oscillator_wave_shape(
                        &ui_weak_thread,
                        &mut values.oscillators,
                        oscillator_index,
                        shape_index,
                    );
                }
                UIUpdates::OscillatorFineTune(oscillator_index, normal_value, cents) => {
                    set_oscillator_fine_tune(
                        &ui_weak_thread,
                        &mut values.oscillators,
                        oscillator_index,
                        normal_value,
                    );
                    set_oscillator_fine_tune_display(
                        &ui_weak_thread,
                        &mut values.oscillator_fine_tune,
                        oscillator_index,
                        cents,
                    );
                }
                UIUpdates::OscillatorFineTuneCents(oscillator_index, cents) => {
                    set_oscillator_fine_tune_display(
                        &ui_weak_thread,
                        &mut values.oscillator_fine_tune,
                        oscillator_index,
                        cents,
                    );
                }
                UIUpdates::OscillatorCourseTune(oscillator_index, normal_value) => {
                    set_oscillator_course_tune(
                        &ui_weak_thread,
                        &mut values.oscillators,
                        oscillator_index,
                        normal_value,
                    );
                }
                UIUpdates::OscillatorClipperBoost(oscillator_index, level) => {
                    set_oscillator_clipper_boost(
                        &ui_weak_thread,
                        &mut values.oscillators,
                        oscillator_index,
                        level,
                    );
                }
                UIUpdates::OscillatorParameter1(oscillator_index, value) => {
                    set_oscillator_parameter1(
                        &ui_weak_thread,
                        &mut values.oscillators,
                        oscillator_index,
                        value,
                    );
                }
                UIUpdates::OscillatorParameter2(oscillator_index, value) => {
                    set_oscillator_parameter2(
                        &ui_weak_thread,
                        &mut values.oscillators,
                        oscillator_index,
                        value,
                    );
                }
                UIUpdates::LFOFrequency(lfo_index, value) => {
                    if let Some(lfo_index) = LFOIndex::from_i32(lfo_index) {
                        let lfo_values = match lfo_index {
                            LFOIndex::ModWheel => &mut values.mod_wheel_lfo,
                            LFOIndex::Filter => &mut values.filter_lfo,
                        };
                        set_lfo_frequency(&ui_weak_thread, lfo_index, lfo_values, value);
                        let lfo_display_value =
                            exponential_curve_lfo_frequency_from_normal_value(value);
                        set_lfo_frequency_display(&ui_weak_thread, lfo_index, lfo_display_value)
                    }
                }
                UIUpdates::LFOFrequencyDisplay(lfo_index, value) => {
                    if let Some(lfo_index) = LFOIndex::from_i32(lfo_index) {
                        set_lfo_frequency_display(&ui_weak_thread, lfo_index, value)
                    }
                }
                UIUpdates::LFOWaveShape(lfo_index, value) => {
                    if let Some(lfo_index) = LFOIndex::from_i32(lfo_index) {
                        let lfo_values = match lfo_index {
                            LFOIndex::ModWheel => &mut values.mod_wheel_lfo,
                            LFOIndex::Filter => &mut values.filter_lfo,
                        };
                        set_lfo_wave_shape(&ui_weak_thread, lfo_index, lfo_values, value);
                    }
                }
                UIUpdates::LFOPhase(lfo_index, value) => {
                    if let Some(lfo_index) = LFOIndex::from_i32(lfo_index) {
                        let lfo_values = match lfo_index {
                            LFOIndex::ModWheel => &mut values.mod_wheel_lfo,
                            LFOIndex::Filter => &mut values.filter_lfo,
                        };
                        set_lfo_phase(&ui_weak_thread, lfo_index, lfo_values, value);
                        let lfo_display_value = (value * MAX_PHASE_VALUE).ceil() as i32;
                        set_lfo_phase_display(&ui_weak_thread, lfo_index, lfo_display_value);
                    }
                }
                UIUpdates::EnvelopeAttackTime(envelope_index, value) => {
                    if let Some(envelope_index) = EnvelopeIndex::from_i32(envelope_index) {
                        let envelope_values = match envelope_index {
                            EnvelopeIndex::Amp => &mut values.amp_envelope,
                            EnvelopeIndex::Filter => &mut values.filter_envelope,
                        };
                        set_envelope_stage_value(
                            &ui_weak_thread,
                            envelope_index,
                            EnvelopeStage::Attack,
                            envelope_values,
                            value,
                        );
                    }
                }
                UIUpdates::EnvelopeDecayTime(envelope_index, value) => {
                    if let Some(envelope_index) = EnvelopeIndex::from_i32(envelope_index) {
                        let envelope_values = match envelope_index {
                            EnvelopeIndex::Amp => &mut values.amp_envelope,
                            EnvelopeIndex::Filter => &mut values.filter_envelope,
                        };
                        set_envelope_stage_value(
                            &ui_weak_thread,
                            envelope_index,
                            EnvelopeStage::Decay,
                            envelope_values,
                            value,
                        );
                    }
                }
                UIUpdates::EnvelopeSustainLevel(envelope_index, value) => {
                    if let Some(envelope_index) = EnvelopeIndex::from_i32(envelope_index) {
                        let envelope_values = match envelope_index {
                            EnvelopeIndex::Amp => &mut values.amp_envelope,
                            EnvelopeIndex::Filter => &mut values.filter_envelope,
                        };
                        set_envelope_stage_value(
                            &ui_weak_thread,
                            envelope_index,
                            EnvelopeStage::Sustain,
                            envelope_values,
                            value,
                        );
                    }
                }
                UIUpdates::EnvelopeReleaseTime(envelope_index, value) => {
                    if let Some(envelope_index) = EnvelopeIndex::from_i32(envelope_index) {
                        let envelope_values = match envelope_index {
                            EnvelopeIndex::Amp => &mut values.amp_envelope,
                            EnvelopeIndex::Filter => &mut values.filter_envelope,
                        };
                        set_envelope_stage_value(
                            &ui_weak_thread,
                            envelope_index,
                            EnvelopeStage::Release,
                            envelope_values,
                            value,
                        );
                    }
                }
                UIUpdates::EnvelopeInverted(envelope_index, value) => {
                    if let Some(envelope_index) = EnvelopeIndex::from_i32(envelope_index) {
                        let envelope_values = match envelope_index {
                            EnvelopeIndex::Amp => &mut values.amp_envelope,
                            EnvelopeIndex::Filter => &mut values.filter_envelope,
                        };
                        set_envelope_inverted(
                            &ui_weak_thread,
                            envelope_index,
                            envelope_values,
                            value,
                        );
                    }
                }
                UIUpdates::FilterCutoff(value) => {
                    let filter_cutoff_values = &mut values.filter_cutoff;
                    set_filter_cutoff_value(
                        &ui_weak_thread,
                        filter_cutoff_values,
                        value,
                    );
                },
                UIUpdates::FilterResonance(value) => {
                    let filter_cutoff_values = &mut values.filter_cutoff;
                    set_filter_resonance_value(
                        &ui_weak_thread,
                        filter_cutoff_values,
                        value,
                    );
                },
            }
        }
    });
}
