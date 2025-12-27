use crate::AccidentalSynth;
use crate::defaults::Defaults;
use crate::synthesizer::midi_value_converters::{
    exponential_curve_lfo_frequency_from_normal_value, normal_value_to_bool,
    normal_value_to_integer_range, normal_value_to_number_of_filter_poles,
    normal_value_to_wave_shape_index,
};
use crate::ui::constants::MAX_PHASE_VALUE;
use crate::ui::{
    EnvelopeIndex, EnvelopeStage, LFOIndex, ParameterValues, UIUpdates,
    set_audio_device_channel_indexes, set_audio_device_channel_list, set_audio_device_values,
    set_envelope_inverted, set_envelope_stage_value, set_filter_cutoff_values,
    set_filter_options_values, set_global_options_values, set_lfo_frequency_display,
    set_lfo_phase_display, set_lfo_values, set_midi_port_values, set_midi_screen_values,
    set_oscillator_fine_tune_display, set_oscillator_mixer_values, set_oscillator_values,
    set_output_mixer_values,
};
use crossbeam_channel::Receiver;
use slint::Weak;
use std::sync::{Arc, Mutex};
use std::thread;

#[allow(clippy::too_many_lines)]
pub fn start_ui_update_listener(
    ui_update_receiver: Receiver<UIUpdates>,
    ui_weak: &Weak<AccidentalSynth>,
    parameter_values: &Arc<Mutex<ParameterValues>>,
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
                UIUpdates::MidiScreen(message) => {
                    let midi_screen_values = &mut values.midi_screen;
                    set_midi_screen_values(&ui_weak_thread, midi_screen_values, message);
                }
                UIUpdates::MidiPortList(port_list) => {
                    let midi_port_values = &mut values.midi_port;
                    midi_port_values.input_ports = port_list;
                    set_midi_port_values(&ui_weak_thread, &mut values.midi_port);
                }
                UIUpdates::MidiPortIndex(index) => {
                    let midi_port_values = &mut values.midi_port;
                    midi_port_values.input_port_index = index;
                    set_midi_port_values(&ui_weak_thread, &mut values.midi_port);
                }
                UIUpdates::MidiChannelIndex(index) => {
                    let midi_port_values = &mut values.midi_port;
                    midi_port_values.channel_index = index;
                    set_midi_port_values(&ui_weak_thread, &mut values.midi_port);
                }
                UIUpdates::AudioDeviceList(device_list) => {
                    let audio_device_values = &mut values.audio_device;
                    audio_device_values.output_devices = device_list;
                    set_audio_device_values(&ui_weak_thread, audio_device_values);
                }
                UIUpdates::AudioDeviceIndex(index) => {
                    let audio_device_values = &mut values.audio_device;
                    audio_device_values.output_device_index = index;
                    set_audio_device_values(&ui_weak_thread, audio_device_values);
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
                UIUpdates::AudioDeviceSampleRateIndex(index) => {
                    let audio_device_values = &mut values.audio_device;
                    audio_device_values.sample_rate_index = index;
                    set_audio_device_values(&ui_weak_thread, audio_device_values);
                }
                UIUpdates::AudioDeviceBufferSizeIndex(index) => {
                    let audio_device_values = &mut values.audio_device;
                    audio_device_values.buffer_size_index = index;
                    set_audio_device_values(&ui_weak_thread, audio_device_values);
                }
                UIUpdates::OscillatorWaveShape(oscillator_index, shape_index) => {
                    let oscillator_values = &mut values.oscillators;
                    oscillator_values[oscillator_index as usize].wave_shape_index = shape_index;
                    set_oscillator_values(&ui_weak_thread, &mut values.oscillators);
                }
                UIUpdates::OscillatorFineTune(oscillator_index, normal_value, cents) => {
                    let oscillator_values = &mut values.oscillators;
                    oscillator_values[oscillator_index as usize].fine_tune = normal_value;
                    set_oscillator_values(&ui_weak_thread, &mut values.oscillators);

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
                UIUpdates::OscillatorCourseTune(oscillator_index, intervals) => {
                    let oscillator_values = &mut values.oscillators;
                    oscillator_values[oscillator_index as usize].course_tune = intervals;

                    set_oscillator_values(&ui_weak_thread, &mut values.oscillators);
                }
                UIUpdates::OscillatorClipperBoost(oscillator_index, level) => {
                    let oscillator_values = &mut values.oscillators;
                    oscillator_values[oscillator_index as usize].clipper_boost = level;

                    set_oscillator_values(&ui_weak_thread, &mut values.oscillators);
                }
                UIUpdates::OscillatorParameter1(oscillator_index, value) => {
                    let oscillator_values = &mut values.oscillators;
                    oscillator_values[oscillator_index as usize].parameter1 = value;

                    set_oscillator_values(&ui_weak_thread, &mut values.oscillators);
                }
                UIUpdates::OscillatorParameter2(oscillator_index, value) => {
                    let oscillator_values = &mut values.oscillators;
                    oscillator_values[oscillator_index as usize].parameter2 = value;

                    set_oscillator_values(&ui_weak_thread, &mut values.oscillators);
                }
                UIUpdates::LFOFrequency(lfo_index, value) => {
                    if let Some(lfo_index) = LFOIndex::from_i32(lfo_index) {
                        let lfo_values = match lfo_index {
                            LFOIndex::ModWheel => &mut values.mod_wheel_lfo,
                            LFOIndex::Filter => &mut values.filter_lfo,
                        };
                        lfo_values.frequency = value;
                        set_lfo_values(&ui_weak_thread, lfo_index, lfo_values);
                        let lfo_display_value =
                            exponential_curve_lfo_frequency_from_normal_value(value);
                        set_lfo_frequency_display(&ui_weak_thread, lfo_index, lfo_display_value);
                    }
                }
                UIUpdates::LFOFrequencyDisplay(lfo_index, value) => {
                    if let Some(lfo_index) = LFOIndex::from_i32(lfo_index) {
                        set_lfo_frequency_display(&ui_weak_thread, lfo_index, value);
                    }
                }
                UIUpdates::LFOWaveShape(lfo_index, value) => {
                    if let Some(lfo_index) = LFOIndex::from_i32(lfo_index) {
                        let lfo_values = match lfo_index {
                            LFOIndex::ModWheel => &mut values.mod_wheel_lfo,
                            LFOIndex::Filter => &mut values.filter_lfo,
                        };
                        lfo_values.wave_shape_index =
                            i32::from(normal_value_to_wave_shape_index(value));
                        set_lfo_values(&ui_weak_thread, lfo_index, lfo_values);
                    }
                }
                UIUpdates::LFOPhase(lfo_index, value) => {
                    if let Some(lfo_index) = LFOIndex::from_i32(lfo_index) {
                        let lfo_values = match lfo_index {
                            LFOIndex::ModWheel => &mut values.mod_wheel_lfo,
                            LFOIndex::Filter => &mut values.filter_lfo,
                        };
                        lfo_values.phase = value;
                        set_lfo_values(&ui_weak_thread, lfo_index, lfo_values);

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
                    filter_cutoff_values.cutoff = value;
                    set_filter_cutoff_values(&ui_weak_thread, filter_cutoff_values);
                }
                UIUpdates::FilterResonance(value) => {
                    let filter_cutoff_values = &mut values.filter_cutoff;
                    filter_cutoff_values.resonance = value;
                    set_filter_cutoff_values(&ui_weak_thread, filter_cutoff_values);
                }
                UIUpdates::FilterPoles(value) => {
                    let filter_option_values = &mut values.filter_options;
                    filter_option_values.poles =
                        i32::from(normal_value_to_number_of_filter_poles(value));
                    set_filter_options_values(&ui_weak_thread, filter_option_values);
                }
                UIUpdates::FilterKeyTracking(value) => {
                    let filter_option_values = &mut values.filter_options;
                    filter_option_values.key_track = value;
                    set_filter_options_values(&ui_weak_thread, filter_option_values);
                }
                UIUpdates::FilterEnvelopeAmount(value) => {
                    let filter_option_values = &mut values.filter_options;
                    filter_option_values.envelope_amount = value;
                    set_filter_options_values(&ui_weak_thread, filter_option_values);
                }
                UIUpdates::FilterLFOAmount(value) => {
                    let filter_option_values = &mut values.filter_options;
                    filter_option_values.lfo_amount = value;
                    set_filter_options_values(&ui_weak_thread, filter_option_values);
                }

                UIUpdates::OutputMixerBalance(value) => {
                    let output_mixer_values = &mut values.output_mixer;
                    output_mixer_values.balance = value;
                    set_output_mixer_values(&ui_weak_thread, output_mixer_values);
                }

                UIUpdates::OutputMixerLevel(value) => {
                    let output_mixer_values = &mut values.output_mixer;
                    output_mixer_values.level = value;
                    set_output_mixer_values(&ui_weak_thread, output_mixer_values);
                }

                UIUpdates::OutputMixerIsMuted(value) => {
                    let output_mixer_values = &mut values.output_mixer;
                    output_mixer_values.is_muted = normal_value_to_bool(value);
                    set_output_mixer_values(&ui_weak_thread, output_mixer_values);
                }

                UIUpdates::OscillatorMixerBalance(oscillator_index, value) => {
                    let output_mixer_values = &mut values.oscillator_mixer;
                    output_mixer_values[oscillator_index as usize].balance = value;
                    set_oscillator_mixer_values(&ui_weak_thread, output_mixer_values);
                }

                UIUpdates::OscillatorMixerLevel(oscillator_index, value) => {
                    let output_mixer_values = &mut values.oscillator_mixer;
                    output_mixer_values[oscillator_index as usize].level = value;
                    set_oscillator_mixer_values(&ui_weak_thread, output_mixer_values);
                }

                UIUpdates::OscillatorMixerIsMuted(oscillator_index, value) => {
                    let output_mixer_values = &mut values.oscillator_mixer;
                    output_mixer_values[oscillator_index as usize].is_muted =
                        normal_value_to_bool(value);
                    set_oscillator_mixer_values(&ui_weak_thread, output_mixer_values);
                }
                UIUpdates::PortamentoTime(time) => {
                    let global_options_values = &mut values.global_options;
                    global_options_values.portamento_time = time;
                    set_global_options_values(&ui_weak_thread, global_options_values);
                }
                UIUpdates::PortamentoEnabled(is_enabled) => {
                    let global_options_values = &mut values.global_options;
                    global_options_values.portamento_is_enabled = normal_value_to_bool(is_enabled);
                    set_global_options_values(&ui_weak_thread, global_options_values);
                }
                UIUpdates::PitchBendRange(range) => {
                    let global_options_values = &mut values.global_options;
                    global_options_values.pitch_bend_range = normal_value_to_integer_range(
                        range,
                        Defaults::MINIMUM_PITCH_BEND_RANGE,
                        Defaults::MAXIMUM_PITCH_BEND_RANGE,
                    ) as i32;
                    set_global_options_values(&ui_weak_thread, global_options_values);
                }
                UIUpdates::VelocityCurve(slope) => {
                    let global_options_values = &mut values.global_options;
                    global_options_values.velocity_curve_slope = slope;
                    set_global_options_values(&ui_weak_thread, global_options_values);
                }
                UIUpdates::HardSync(is_enabled) => {
                    let global_options_values = &mut values.global_options;
                    global_options_values.hard_sync_is_enabled = normal_value_to_bool(is_enabled);
                    set_global_options_values(&ui_weak_thread, global_options_values);
                }
                UIUpdates::KeySync(is_enabled) => {
                    let global_options_values = &mut values.global_options;
                    global_options_values.key_sync_is_enabled = normal_value_to_bool(is_enabled);
                    set_global_options_values(&ui_weak_thread, global_options_values);
                }
            }
        }
    });
}
