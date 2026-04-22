use crate::modules::lfo::DEFAULT_LFO_PHASE;
use crate::modules::oscillator::OscillatorParameters;
use crate::synthesizer::midi_value_converters::scaled_velocity_from_normal_value;
use crate::synthesizer::set_parameters::{
    set_envelope_amount, set_envelope_attack_time, set_envelope_decay_time, set_envelope_inverted,
    set_envelope_release_time, set_envelope_sustain_level, set_envelope_sustain_pedal,
    set_filter_cutoff, set_filter_poles, set_filter_resonance, set_key_tracking_amount,
    set_lfo_center_value, set_lfo_frequency, set_lfo_phase, set_lfo_phase_reset, set_lfo_range,
    set_lfo_wave_shape, set_mod_wheel, set_oscillator_balance, set_oscillator_clip_boost,
    set_oscillator_course_tune, set_oscillator_fine_tune, set_oscillator_hard_sync,
    set_oscillator_key_sync, set_oscillator_level, set_oscillator_mute,
    set_oscillator_shape_parameter1, set_oscillator_shape_parameter2, set_oscillator_wave_shape,
    set_output_balance, set_output_level, set_output_mute, set_pitch_bend_range,
    set_portamento_enabled, set_portamento_time, set_velocity_curve,
};
use crate::synthesizer::{
    CurrentNote, KeyboardParameters, MidiGateEvent, MidiNoteEvent, ModuleParameters,
    midi_value_converters,
};
use accsyn_types::defaults::Defaults;
use accsyn_types::math::{normalize_midi_value, store_f32_as_atomic_u32};
use accsyn_types::midi_events::CC;
use accsyn_types::synth_events::{
    EnvelopeIndex, LFOIndex, OscillatorIndex, SynthesizerUpdateEvents,
};
use accsyn_types::ui_events::UIUpdates;
use crossbeam_channel::Sender;
use std::sync::Arc;
use std::sync::atomic::Ordering::{Relaxed, Release};

fn send_ui_update(ui_update_sender: &Sender<UIUpdates>, update: UIUpdates) {
    if let Err(e) = ui_update_sender.send(update) {
        log::error!(target: "synthesizer::midi", "Failed to send UI update: {e}");
    }
}

pub fn action_midi_note_events(
    midi_events: MidiNoteEvent,
    module_parameters: &Arc<ModuleParameters>,
) {
    match midi_events {
        MidiNoteEvent::NoteOn => {
            module_parameters.envelopes[EnvelopeIndex::Amp as usize]
                .gate_flag
                .store(MidiGateEvent::GateOn as u8, Relaxed);
            module_parameters.envelopes[EnvelopeIndex::Filter as usize]
                .gate_flag
                .store(MidiGateEvent::GateOn as u8, Relaxed);
            for oscillator in &module_parameters.oscillators {
                oscillator.gate_flag.store(true, Release);
            }
        }
        MidiNoteEvent::NoteOff => {
            module_parameters.envelopes[EnvelopeIndex::Amp as usize]
                .gate_flag
                .store(MidiGateEvent::GateOff as u8, Relaxed);
            module_parameters.envelopes[EnvelopeIndex::Filter as usize]
                .gate_flag
                .store(MidiGateEvent::GateOff as u8, Relaxed);
        }
    }
}

pub fn process_midi_program_change_message(
    ui_update_sender: &Sender<UIUpdates>,
    synthesizer_update_sender: &Sender<SynthesizerUpdateEvents>,
    program_number: u8,
) {
    log::debug!(target: "synthesizer::midi", "Program change received: {program_number}");

    if let Err(e) = synthesizer_update_sender.send(SynthesizerUpdateEvents::PatchChanged(
        i32::from(program_number),
    )) {
        log::error!(target: "synthesizer::midi", "Failed to send program change to synthesizer: {e}");
    }
    send_ui_update(
        ui_update_sender,
        UIUpdates::Patches(i32::from(program_number)),
    );
}

pub fn process_midi_channel_pressure_message(parameters: &KeyboardParameters, pressure_value: u8) {
    log::debug!(target: "synthesizer::midi", "Channel pressure received: {pressure_value}");
    let aftertouch_amount = normalize_midi_value(pressure_value);
    parameters.aftertouch_amount.store(aftertouch_amount);
}

pub fn process_midi_pitch_bend_message(
    oscillators: &[OscillatorParameters; 4],
    range: u8,
    bend_amount: u16,
) {
    log::debug!(target: "synthesizer::midi", "Pitch bend received: amount={bend_amount}, range={range}");
    midi_value_converters::update_current_note_from_midi_pitch_bend(
        bend_amount,
        range,
        oscillators,
    );
}

pub fn process_midi_note_off_message(module_parameters: &mut Arc<ModuleParameters>) {
    log::debug!(target: "synthesizer::midi", "Note off");
    action_midi_note_events(MidiNoteEvent::NoteOff, module_parameters);
}

pub fn process_midi_note_on_message(
    module_parameters: &mut Arc<ModuleParameters>,
    current_note: &mut Arc<CurrentNote>,
    midi_note: u8,
    velocity: u8,
    ui_update_sender: &Sender<UIUpdates>,
) {
    log::debug!(target: "synthesizer::midi", "Note on: note={midi_note}, velocity={velocity}");

    let scaled_velocity = scaled_velocity_from_normal_value(
        module_parameters.keyboard.velocity_curve.load(),
        normalize_midi_value(velocity),
    );

    store_f32_as_atomic_u32(&current_note.velocity, scaled_velocity);
    current_note.midi_note.store(midi_note, Relaxed);

    module_parameters
        .filter
        .current_note_number
        .store(midi_note, Relaxed);

    action_midi_note_events(MidiNoteEvent::NoteOn, module_parameters);

    let note_name = Defaults::MIDI_NOTE_FREQUENCIES[midi_note as usize & 0x7F]
        .1
        .to_string();
    send_ui_update(ui_update_sender, UIUpdates::MidiScreen(note_name));
}

// This function has to match every CC value, so it is going to be very long.
#[allow(clippy::too_many_lines)]
pub fn process_midi_cc_values(
    cc_value: CC,
    module_parameters: &mut Arc<ModuleParameters>,
    ui_update_sender: &Sender<UIUpdates>,
) {
    log::trace!(target: "synthesizer::midi", "CC received: {cc_value:?}");
    match cc_value {
        CC::ModWheel(value) => {
            set_mod_wheel(&module_parameters.keyboard, normalize_midi_value(value));
        }
        CC::VelocityCurve(value) => {
            let normal_value = normalize_midi_value(value);
            set_velocity_curve(&module_parameters.keyboard, normalize_midi_value(value));

            send_ui_update(ui_update_sender, UIUpdates::VelocityCurve(normal_value));
        }
        CC::PitchBendRange(value) => {
            let normal_value = normalize_midi_value(value);
            set_pitch_bend_range(&module_parameters.keyboard, normal_value);

            send_ui_update(ui_update_sender, UIUpdates::PitchBendRange(normal_value));
        }
        CC::Volume(value) => {
            let normal_value = normalize_midi_value(value);
            set_output_level(&module_parameters.mixer, normal_value);
            send_ui_update(ui_update_sender, UIUpdates::OutputMixerLevel(normal_value));
        }
        CC::Balance(value) => {
            let normal_value = normalize_midi_value(value);
            set_output_balance(&module_parameters.mixer, normal_value);
            send_ui_update(
                ui_update_sender,
                UIUpdates::OutputMixerBalance(normal_value),
            );
        }
        CC::Mute(value) => {
            let normal_value = normalize_midi_value(value);
            set_output_mute(&module_parameters.mixer, normal_value);
            send_ui_update(
                ui_update_sender,
                UIUpdates::OutputMixerIsMuted(normal_value),
            );
        }
        CC::SubOscillatorShapeParameter1(value) => {
            let parameter1_value = normalize_midi_value(value);
            let oscillator_index = OscillatorIndex::Sub;
            set_oscillator_shape_parameter1(
                &module_parameters.oscillators[oscillator_index as usize],
                parameter1_value,
            );

            send_ui_update(
                ui_update_sender,
                UIUpdates::OscillatorParameter1(oscillator_index as i32, parameter1_value),
            );
        }
        CC::SubOscillatorShapeParameter2(value) => {
            let parameter2_value = normalize_midi_value(value);
            let oscillator_index = OscillatorIndex::Sub;
            set_oscillator_shape_parameter2(
                &module_parameters.oscillators[oscillator_index as usize],
                parameter2_value,
            );
            send_ui_update(
                ui_update_sender,
                UIUpdates::OscillatorParameter2(oscillator_index as i32, parameter2_value),
            );
        }
        CC::Oscillator1ShapeParameter1(value) => {
            let parameter1_value = normalize_midi_value(value);
            let oscillator_index = OscillatorIndex::One;
            set_oscillator_shape_parameter1(
                &module_parameters.oscillators[oscillator_index as usize],
                parameter1_value,
            );
            send_ui_update(
                ui_update_sender,
                UIUpdates::OscillatorParameter1(oscillator_index as i32, parameter1_value),
            );
        }
        CC::Oscillator1ShapeParameter2(value) => {
            let parameter2_value = normalize_midi_value(value);
            let oscillator_index = OscillatorIndex::One;
            set_oscillator_shape_parameter2(
                &module_parameters.oscillators[oscillator_index as usize],
                parameter2_value,
            );
            send_ui_update(
                ui_update_sender,
                UIUpdates::OscillatorParameter2(oscillator_index as i32, parameter2_value),
            );
        }
        CC::Oscillator2ShapeParameter1(value) => {
            let parameter1_value = normalize_midi_value(value);
            let oscillator_index = OscillatorIndex::Two;
            set_oscillator_shape_parameter1(
                &module_parameters.oscillators[oscillator_index as usize],
                parameter1_value,
            );
            send_ui_update(
                ui_update_sender,
                UIUpdates::OscillatorParameter1(oscillator_index as i32, parameter1_value),
            );
        }
        CC::Oscillator2ShapeParameter2(value) => {
            let parameter2_value = normalize_midi_value(value);
            let oscillator_index = OscillatorIndex::Two;
            set_oscillator_shape_parameter2(
                &module_parameters.oscillators[oscillator_index as usize],
                parameter2_value,
            );
            send_ui_update(
                ui_update_sender,
                UIUpdates::OscillatorParameter2(oscillator_index as i32, parameter2_value),
            );
        }
        CC::Oscillator3ShapeParameter1(value) => {
            let parameter1_value = normalize_midi_value(value);
            let oscillator_index = OscillatorIndex::Three;
            set_oscillator_shape_parameter1(
                &module_parameters.oscillators[oscillator_index as usize],
                parameter1_value,
            );
            send_ui_update(
                ui_update_sender,
                UIUpdates::OscillatorParameter1(oscillator_index as i32, parameter1_value),
            );
        }
        CC::Oscillator3ShapeParameter2(value) => {
            let parameter2_value = normalize_midi_value(value);
            let oscillator_index = OscillatorIndex::Three;
            set_oscillator_shape_parameter2(
                &module_parameters.oscillators[oscillator_index as usize],
                parameter2_value,
            );
            send_ui_update(
                ui_update_sender,
                UIUpdates::OscillatorParameter2(oscillator_index as i32, parameter2_value),
            );
        }
        CC::OscillatorKeySyncEnabled(value) => {
            let normal_value = normalize_midi_value(value);
            set_oscillator_key_sync(&module_parameters.oscillators, normal_value);

            send_ui_update(ui_update_sender, UIUpdates::KeySync(normal_value));
        }
        CC::PortamentoTime(value) => {
            let normal_value = normalize_midi_value(value);
            set_portamento_time(&module_parameters.oscillators, normal_value);

            send_ui_update(ui_update_sender, UIUpdates::PortamentoTime(normal_value));
        }
        CC::OscillatorHardSync(value) => {
            let normal_value = normalize_midi_value(value);
            set_oscillator_hard_sync(&module_parameters.oscillators, normal_value);

            send_ui_update(ui_update_sender, UIUpdates::HardSync(normal_value));
        }
        CC::SubOscillatorShape(value) => {
            let normal_value = normalize_midi_value(value);
            let oscillator_index = OscillatorIndex::Sub;
            let shape_index = set_oscillator_wave_shape(
                &module_parameters.oscillators[OscillatorIndex::Sub as usize],
                normal_value,
            );
            send_ui_update(
                ui_update_sender,
                UIUpdates::OscillatorWaveShape(oscillator_index as i32, i32::from(shape_index)),
            );
        }
        CC::Oscillator1Shape(value) => {
            let normal_value = normalize_midi_value(value);
            let oscillator_index = OscillatorIndex::One;
            let shape_index = set_oscillator_wave_shape(
                &module_parameters.oscillators[oscillator_index as usize],
                normal_value,
            );
            send_ui_update(
                ui_update_sender,
                UIUpdates::OscillatorWaveShape(oscillator_index as i32, i32::from(shape_index)),
            );
        }
        CC::Oscillator2Shape(value) => {
            let normal_value = normalize_midi_value(value);
            let oscillator_index = OscillatorIndex::Two;
            let shape_index = set_oscillator_wave_shape(
                &module_parameters.oscillators[oscillator_index as usize],
                normal_value,
            );
            send_ui_update(
                ui_update_sender,
                UIUpdates::OscillatorWaveShape(oscillator_index as i32, i32::from(shape_index)),
            );
        }
        CC::Oscillator3Shape(value) => {
            let normal_value = normalize_midi_value(value);
            let oscillator_index = OscillatorIndex::Three;
            let shape_index = set_oscillator_wave_shape(
                &module_parameters.oscillators[oscillator_index as usize],
                normal_value,
            );
            send_ui_update(
                ui_update_sender,
                UIUpdates::OscillatorWaveShape(oscillator_index as i32, i32::from(shape_index)),
            );
        }
        CC::SubOscillatorCourseTune(value) => {
            let oscillator_index = OscillatorIndex::Sub;
            let course_tune = set_oscillator_course_tune(
                &module_parameters.oscillators[oscillator_index as usize],
                normalize_midi_value(value),
            );

            send_ui_update(
                ui_update_sender,
                UIUpdates::OscillatorCourseTune(oscillator_index as i32, i32::from(course_tune)),
            );
        }
        CC::Oscillator1CourseTune(value) => {
            let oscillator_index = OscillatorIndex::One;
            let course_tune = set_oscillator_course_tune(
                &module_parameters.oscillators[oscillator_index as usize],
                normalize_midi_value(value),
            );

            send_ui_update(
                ui_update_sender,
                UIUpdates::OscillatorCourseTune(oscillator_index as i32, i32::from(course_tune)),
            );
        }
        CC::Oscillator2CourseTune(value) => {
            let oscillator_index = OscillatorIndex::Two;
            let course_tune = set_oscillator_course_tune(
                &module_parameters.oscillators[oscillator_index as usize],
                normalize_midi_value(value),
            );

            send_ui_update(
                ui_update_sender,
                UIUpdates::OscillatorCourseTune(oscillator_index as i32, i32::from(course_tune)),
            );
        }
        CC::Oscillator3CourseTune(value) => {
            let oscillator_index = OscillatorIndex::Three;
            let course_tune = set_oscillator_course_tune(
                &module_parameters.oscillators[oscillator_index as usize],
                normalize_midi_value(value),
            );

            send_ui_update(
                ui_update_sender,
                UIUpdates::OscillatorCourseTune(oscillator_index as i32, i32::from(course_tune)),
            );
        }
        CC::SubOscillatorFineTune(value) => {
            let oscillator_index = OscillatorIndex::Sub;
            let fine_tune_normal_value = normalize_midi_value(value);
            let cents = set_oscillator_fine_tune(
                &module_parameters.oscillators[oscillator_index as usize],
                fine_tune_normal_value,
            );

            send_ui_update(
                ui_update_sender,
                UIUpdates::OscillatorFineTune(
                    oscillator_index as i32,
                    fine_tune_normal_value,
                    i32::from(cents),
                ),
            );
        }
        CC::Oscillator1FineTune(value) => {
            let oscillator_index = OscillatorIndex::One;
            let fine_tune_normal_value = normalize_midi_value(value);
            let cents = set_oscillator_fine_tune(
                &module_parameters.oscillators[oscillator_index as usize],
                fine_tune_normal_value,
            );
            send_ui_update(
                ui_update_sender,
                UIUpdates::OscillatorFineTune(
                    oscillator_index as i32,
                    fine_tune_normal_value,
                    i32::from(cents),
                ),
            );
        }
        CC::Oscillator2FineTune(value) => {
            let oscillator_index = OscillatorIndex::Two;
            let fine_tune_normal_value = normalize_midi_value(value);
            let cents = set_oscillator_fine_tune(
                &module_parameters.oscillators[oscillator_index as usize],
                fine_tune_normal_value,
            );
            send_ui_update(
                ui_update_sender,
                UIUpdates::OscillatorFineTune(
                    oscillator_index as i32,
                    fine_tune_normal_value,
                    i32::from(cents),
                ),
            );
        }
        CC::Oscillator3FineTune(value) => {
            let oscillator_index = OscillatorIndex::Three;
            let fine_tune_normal_value = normalize_midi_value(value);
            let cents = set_oscillator_fine_tune(
                &module_parameters.oscillators[oscillator_index as usize],
                fine_tune_normal_value,
            );

            send_ui_update(
                ui_update_sender,
                UIUpdates::OscillatorFineTune(
                    oscillator_index as i32,
                    fine_tune_normal_value,
                    i32::from(cents),
                ),
            );
        }
        CC::SubOscillatorLevel(value) => {
            let normal_value = normalize_midi_value(value);
            set_oscillator_level(&module_parameters.mixer, OscillatorIndex::Sub, normal_value);
            send_ui_update(
                ui_update_sender,
                UIUpdates::OscillatorMixerLevel(OscillatorIndex::Sub as i32, normal_value),
            );
        }
        CC::Oscillator1Level(value) => {
            let normal_value = normalize_midi_value(value);
            set_oscillator_level(&module_parameters.mixer, OscillatorIndex::One, normal_value);
            send_ui_update(
                ui_update_sender,
                UIUpdates::OscillatorMixerLevel(OscillatorIndex::One as i32, normal_value),
            );
        }
        CC::Oscillator2Level(value) => {
            let normal_value = normalize_midi_value(value);
            set_oscillator_level(&module_parameters.mixer, OscillatorIndex::Two, normal_value);
            send_ui_update(
                ui_update_sender,
                UIUpdates::OscillatorMixerLevel(OscillatorIndex::Two as i32, normal_value),
            );
        }
        CC::Oscillator3Level(value) => {
            let normal_value = normalize_midi_value(value);
            set_oscillator_level(
                &module_parameters.mixer,
                OscillatorIndex::Three,
                normal_value,
            );
            send_ui_update(
                ui_update_sender,
                UIUpdates::OscillatorMixerLevel(OscillatorIndex::Three as i32, normal_value),
            );
        }
        CC::SubOscillatorMute(value) => {
            let normal_value = normalize_midi_value(value);
            set_oscillator_mute(&module_parameters.mixer, OscillatorIndex::Sub, normal_value);
            send_ui_update(
                ui_update_sender,
                UIUpdates::OscillatorMixerIsMuted(OscillatorIndex::Sub as i32, normal_value),
            );
        }
        CC::Oscillator1Mute(value) => {
            let normal_value = normalize_midi_value(value);
            set_oscillator_mute(&module_parameters.mixer, OscillatorIndex::One, normal_value);
            send_ui_update(
                ui_update_sender,
                UIUpdates::OscillatorMixerIsMuted(OscillatorIndex::One as i32, normal_value),
            );
        }
        CC::Oscillator2Mute(value) => {
            let normal_value = normalize_midi_value(value);
            set_oscillator_mute(&module_parameters.mixer, OscillatorIndex::Two, normal_value);
            send_ui_update(
                ui_update_sender,
                UIUpdates::OscillatorMixerIsMuted(OscillatorIndex::Two as i32, normal_value),
            );
        }
        CC::Oscillator3Mute(value) => {
            let normal_value = normalize_midi_value(value);
            set_oscillator_mute(
                &module_parameters.mixer,
                OscillatorIndex::Three,
                normal_value,
            );
            send_ui_update(
                ui_update_sender,
                UIUpdates::OscillatorMixerIsMuted(OscillatorIndex::Three as i32, normal_value),
            );
        }
        CC::SubOscillatorBalance(value) => {
            let normal_value = normalize_midi_value(value);
            set_oscillator_balance(&module_parameters.mixer, OscillatorIndex::Sub, normal_value);
            send_ui_update(
                ui_update_sender,
                UIUpdates::OscillatorMixerBalance(OscillatorIndex::Sub as i32, normal_value),
            );
        }
        CC::Oscillator1Balance(value) => {
            let normal_value = normalize_midi_value(value);
            set_oscillator_balance(&module_parameters.mixer, OscillatorIndex::One, normal_value);
            send_ui_update(
                ui_update_sender,
                UIUpdates::OscillatorMixerBalance(OscillatorIndex::One as i32, normal_value),
            );
        }
        CC::Oscillator2Balance(value) => {
            let normal_value = normalize_midi_value(value);
            set_oscillator_balance(&module_parameters.mixer, OscillatorIndex::Two, normal_value);
            send_ui_update(
                ui_update_sender,
                UIUpdates::OscillatorMixerBalance(OscillatorIndex::Two as i32, normal_value),
            );
        }
        CC::Oscillator3Balance(value) => {
            let normal_value = normalize_midi_value(value);
            set_oscillator_balance(
                &module_parameters.mixer,
                OscillatorIndex::Three,
                normal_value,
            );
            send_ui_update(
                ui_update_sender,
                UIUpdates::OscillatorMixerBalance(OscillatorIndex::Three as i32, normal_value),
            );
        }
        CC::Sustain(value) => {
            let normal_value = normalize_midi_value(value);
            set_envelope_sustain_pedal(&module_parameters.envelopes, normal_value);
        }
        CC::PortamentoEnabled(value) => {
            let normal_value = normalize_midi_value(value);
            set_portamento_enabled(&module_parameters.oscillators, normal_value);

            send_ui_update(ui_update_sender, UIUpdates::PortamentoEnabled(normal_value));
        }
        CC::SubOscillatorClipBoost(value) => {
            let boost_level = normalize_midi_value(value);
            set_oscillator_clip_boost(
                &module_parameters.oscillators[OscillatorIndex::Sub as usize],
                boost_level,
            );
            send_ui_update(
                ui_update_sender,
                UIUpdates::OscillatorClipperBoost(OscillatorIndex::Sub as i32, boost_level),
            );
        }
        CC::Oscillator1ClipBoost(value) => {
            let boost_level = normalize_midi_value(value);
            set_oscillator_clip_boost(
                &module_parameters.oscillators[OscillatorIndex::One as usize],
                normalize_midi_value(value),
            );
            send_ui_update(
                ui_update_sender,
                UIUpdates::OscillatorClipperBoost(OscillatorIndex::One as i32, boost_level),
            );
        }
        CC::Oscillator2ClipBoost(value) => {
            let boost_level = normalize_midi_value(value);
            set_oscillator_clip_boost(
                &module_parameters.oscillators[OscillatorIndex::Two as usize],
                normalize_midi_value(value),
            );

            send_ui_update(
                ui_update_sender,
                UIUpdates::OscillatorClipperBoost(OscillatorIndex::Two as i32, boost_level),
            );
        }
        CC::Oscillator3ClipBoost(value) => {
            let boost_level = normalize_midi_value(value);
            set_oscillator_clip_boost(
                &module_parameters.oscillators[OscillatorIndex::Three as usize],
                normalize_midi_value(value),
            );
            send_ui_update(
                ui_update_sender,
                UIUpdates::OscillatorClipperBoost(OscillatorIndex::Three as i32, boost_level),
            );
        }
        CC::FilterPoles(value) => {
            let normal_value = normalize_midi_value(value);
            set_filter_poles(&module_parameters.filter, normal_value);
            send_ui_update(ui_update_sender, UIUpdates::FilterPoles(normal_value));
        }
        CC::FilterResonance(value) => {
            let normal_value = normalize_midi_value(value);
            set_filter_resonance(&module_parameters.filter, normal_value);
            send_ui_update(ui_update_sender, UIUpdates::FilterResonance(normal_value));
        }
        CC::AmpEGReleaseTime(value) => {
            let normal_value = normalize_midi_value(value);
            set_envelope_release_time(
                &module_parameters.envelopes[EnvelopeIndex::Amp as usize],
                normal_value,
            );
            send_ui_update(
                ui_update_sender,
                UIUpdates::EnvelopeReleaseTime(EnvelopeIndex::Amp as i32, normal_value),
            );
        }
        CC::AmpEGAttackTime(value) => {
            let normal_value = normalize_midi_value(value);
            set_envelope_attack_time(
                &module_parameters.envelopes[EnvelopeIndex::Amp as usize],
                normal_value,
            );
            send_ui_update(
                ui_update_sender,
                UIUpdates::EnvelopeAttackTime(EnvelopeIndex::Amp as i32, normal_value),
            );
        }
        CC::FilterCutoff(value) => {
            let normal_value = normalize_midi_value(value);
            set_filter_cutoff(&module_parameters.filter, normal_value);
            send_ui_update(ui_update_sender, UIUpdates::FilterCutoff(normal_value));
        }
        CC::AmpEGDecayTime(value) => {
            let normal_value = normalize_midi_value(value);
            set_envelope_decay_time(
                &module_parameters.envelopes[EnvelopeIndex::Amp as usize],
                normal_value,
            );
            send_ui_update(
                ui_update_sender,
                UIUpdates::EnvelopeDecayTime(EnvelopeIndex::Amp as i32, normal_value),
            );
        }
        CC::AmpEGSustainLevel(value) => {
            let normal_value = normalize_midi_value(value);
            set_envelope_sustain_level(
                &module_parameters.envelopes[EnvelopeIndex::Amp as usize],
                normal_value,
            );

            send_ui_update(
                ui_update_sender,
                UIUpdates::EnvelopeSustainLevel(EnvelopeIndex::Amp as i32, normal_value),
            );
        }
        CC::AmpEGInverted(value) => {
            let normal_value = normalize_midi_value(value);
            set_envelope_inverted(
                &module_parameters.envelopes[EnvelopeIndex::Amp as usize],
                normal_value,
            );

            send_ui_update(
                ui_update_sender,
                UIUpdates::EnvelopeInverted(EnvelopeIndex::Amp as i32, normal_value),
            );
        }
        CC::FilterEnvelopeAttackTime(value) => {
            let normal_value = normalize_midi_value(value);
            set_envelope_attack_time(
                &module_parameters.envelopes[EnvelopeIndex::Filter as usize],
                normal_value,
            );
            send_ui_update(
                ui_update_sender,
                UIUpdates::EnvelopeAttackTime(EnvelopeIndex::Filter as i32, normal_value),
            );
        }
        CC::FilterEnvelopeDecayTime(value) => {
            let normal_value = normalize_midi_value(value);
            set_envelope_decay_time(
                &module_parameters.envelopes[EnvelopeIndex::Filter as usize],
                normal_value,
            );
            send_ui_update(
                ui_update_sender,
                UIUpdates::EnvelopeDecayTime(EnvelopeIndex::Filter as i32, normal_value),
            );
        }
        CC::FilterEnvelopeSustainLevel(value) => {
            let normal_value = normalize_midi_value(value);
            set_envelope_sustain_level(
                &module_parameters.envelopes[EnvelopeIndex::Filter as usize],
                normal_value,
            );
            send_ui_update(
                ui_update_sender,
                UIUpdates::EnvelopeSustainLevel(EnvelopeIndex::Filter as i32, normal_value),
            );
        }
        CC::FilterEnvelopeReleaseTime(value) => {
            let normal_value = normalize_midi_value(value);
            set_envelope_release_time(
                &module_parameters.envelopes[EnvelopeIndex::Filter as usize],
                normal_value,
            );
            send_ui_update(
                ui_update_sender,
                UIUpdates::EnvelopeReleaseTime(EnvelopeIndex::Filter as i32, normal_value),
            );
        }
        CC::FilterEnvelopeInverted(value) => {
            let normal_value = normalize_midi_value(value);
            set_envelope_inverted(
                &module_parameters.envelopes[EnvelopeIndex::Filter as usize],
                normal_value,
            );
            send_ui_update(
                ui_update_sender,
                UIUpdates::EnvelopeInverted(EnvelopeIndex::Filter as i32, normal_value),
            );
        }
        CC::FilterEnvelopeAmount(value) => {
            let normal_value = normalize_midi_value(value);
            set_envelope_amount(
                &module_parameters.envelopes[EnvelopeIndex::Filter as usize],
                normal_value,
            );
            send_ui_update(
                ui_update_sender,
                UIUpdates::FilterEnvelopeAmount(normal_value),
            );
        }
        CC::KeyTrackingAmount(value) => {
            let normal_value = normalize_midi_value(value);
            set_key_tracking_amount(&module_parameters.filter, normal_value);
            send_ui_update(ui_update_sender, UIUpdates::FilterKeyTracking(normal_value));
        }
        CC::ModWheelLFOFrequency(value) => {
            let normal_value = normalize_midi_value(value);
            set_lfo_frequency(
                &module_parameters.lfos[LFOIndex::ModWheel as usize],
                normal_value,
            );

            send_ui_update(
                ui_update_sender,
                UIUpdates::LFOFrequency(LFOIndex::ModWheel as i32, normal_value),
            );
        }
        CC::ModWheelLFOCenterValue(value) => {
            let normal_value = normalize_midi_value(value);
            set_lfo_center_value(
                &module_parameters.lfos[LFOIndex::ModWheel as usize],
                normal_value,
            );
        }
        CC::ModWheelLFORange(value) => {
            let normal_value = normalize_midi_value(value);
            set_lfo_range(
                &module_parameters.lfos[LFOIndex::ModWheel as usize],
                normal_value,
            );
        }
        CC::ModWheelLFOWaveShape(value) => {
            let normal_value = normalize_midi_value(value);
            set_lfo_wave_shape(
                &module_parameters.lfos[LFOIndex::ModWheel as usize],
                normal_value,
            );

            send_ui_update(
                ui_update_sender,
                UIUpdates::LFOWaveShape(LFOIndex::ModWheel as i32, normal_value),
            );
        }
        CC::ModWheelLFOPhase(value) => {
            let normal_value = normalize_midi_value(value);
            set_lfo_phase(
                &module_parameters.lfos[LFOIndex::ModWheel as usize],
                normal_value,
            );

            send_ui_update(
                ui_update_sender,
                UIUpdates::LFOPhase(LFOIndex::ModWheel as i32, normal_value),
            );
        }
        CC::ModWheelLFOReset => {
            set_lfo_phase_reset(&module_parameters.lfos[LFOIndex::ModWheel as usize]);
            send_ui_update(
                ui_update_sender,
                UIUpdates::LFOPhase(LFOIndex::ModWheel as i32, DEFAULT_LFO_PHASE),
            );
        }
        CC::FilterModLFOFrequency(value) => {
            let normal_value = normalize_midi_value(value);
            set_lfo_frequency(
                &module_parameters.lfos[LFOIndex::Filter as usize],
                normal_value,
            );

            send_ui_update(
                ui_update_sender,
                UIUpdates::LFOFrequency(LFOIndex::Filter as i32, normal_value),
            );
        }
        CC::FilterModLFOAmount(value) => {
            let normal_value = normalize_midi_value(value);
            set_lfo_range(
                &module_parameters.lfos[LFOIndex::Filter as usize],
                normal_value,
            );
            send_ui_update(ui_update_sender, UIUpdates::FilterLFOAmount(normal_value));
        }
        CC::FilterModLFOWaveShape(value) => {
            let normal_value = normalize_midi_value(value);
            set_lfo_wave_shape(
                &module_parameters.lfos[LFOIndex::Filter as usize],
                normal_value,
            );

            send_ui_update(
                ui_update_sender,
                UIUpdates::LFOWaveShape(LFOIndex::Filter as i32, normal_value),
            );
        }
        CC::FilterModLFOPhase(value) => {
            let normal_value = normalize_midi_value(value);
            set_lfo_phase(
                &module_parameters.lfos[LFOIndex::Filter as usize],
                normal_value,
            );

            send_ui_update(
                ui_update_sender,
                UIUpdates::LFOPhase(LFOIndex::Filter as i32, normal_value),
            );
        }
        CC::FilterModLFOReset => {
            set_lfo_phase_reset(&module_parameters.lfos[LFOIndex::Filter as usize]);

            send_ui_update(
                ui_update_sender,
                UIUpdates::LFOPhase(LFOIndex::Filter as i32, DEFAULT_LFO_PHASE),
            );
        }
        CC::AllNotesOff => {
            process_midi_note_off_message(module_parameters);
        }
    }
}

#[cfg(test)]
mod tests {
    use accsyn_types::defaults::Defaults;

    #[test]
    fn midi_note_frequencies_covers_all_valid_notes() {
        for note in 0u8..=127 {
            let _ = Defaults::MIDI_NOTE_FREQUENCIES[note as usize];
        }
    }

    #[test]
    fn midi_note_frequency_index_mask_keeps_index_in_range() {
        for note in 128u8..=255 {
            let index = note as usize & 0x7F;
            assert!(index < Defaults::MIDI_NOTE_FREQUENCIES.len());
        }
    }
}
