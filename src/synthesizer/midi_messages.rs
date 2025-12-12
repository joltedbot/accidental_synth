use crate::defaults::Defaults;
use crate::math::{load_f32_from_atomic_u32, normalize_midi_value, store_f32_as_atomic_u32};
use crate::midi::control_change::CC;
use crate::modules::lfo::DEFAULT_LFO_PHASE;
use crate::modules::oscillator::OscillatorParameters;
use crate::synthesizer::midi_value_converters::scaled_velocity_from_normal_value;
use crate::synthesizer::set_parameters::{
    set_envelope_amount, set_envelope_attack_time, set_envelope_decay_time, set_envelope_inverted,
    set_envelope_release_time, set_envelope_sustain_level, set_filter_cutoff, set_filter_poles,
    set_filter_resonance, set_key_tracking_amount, set_lfo_center_value, set_lfo_frequency,
    set_lfo_phase, set_lfo_phase_reset, set_lfo_range, set_lfo_wave_shape, set_mod_wheel,
    set_oscillator_balance, set_oscillator_clip_boost, set_oscillator_course_tune,
    set_oscillator_fine_tune, set_oscillator_hard_sync, set_oscillator_key_sync,
    set_oscillator_level, set_oscillator_mute, set_oscillator_shape_parameter1,
    set_oscillator_shape_parameter2, set_oscillator_wave_shape, set_output_balance,
    set_output_level, set_output_mute, set_pitch_bend_range, set_portamento_enabled,
    set_portamento_time, set_velocity_curve,
};
use crate::synthesizer::{
    CurrentNote, KeyboardParameters, MidiGateEvent, MidiNoteEvent, ModuleParameters,
    OscillatorIndex, midi_value_converters,
};
use crate::ui::{EnvelopeIndex, LFOIndex, UIUpdates};
use crossbeam_channel::Sender;
use std::sync::Arc;
use std::sync::atomic::Ordering::{Relaxed, Release};

pub fn action_midi_note_events(
    midi_events: MidiNoteEvent,
    module_parameters: &Arc<ModuleParameters>,
) {
    match midi_events {
        MidiNoteEvent::NoteOn => {
            module_parameters
                .amp_envelope
                .gate_flag
                .store(MidiGateEvent::GateOn as u8, Relaxed);
            module_parameters
                .filter_envelope
                .gate_flag
                .store(MidiGateEvent::GateOn as u8, Relaxed);
            for oscillator in &module_parameters.oscillators {
                oscillator.gate_flag.store(true, Release);
            }
        }
        MidiNoteEvent::NoteOff => {
            module_parameters
                .amp_envelope
                .gate_flag
                .store(MidiGateEvent::GateOff as u8, Relaxed);
            module_parameters
                .filter_envelope
                .gate_flag
                .store(MidiGateEvent::GateOff as u8, Relaxed);
        }
    }
}

pub fn process_midi_channel_pressure_message(parameters: &KeyboardParameters, pressure_value: u8) {
    let aftertouch_amount = normalize_midi_value(pressure_value);
    store_f32_as_atomic_u32(&parameters.aftertouch_amount, aftertouch_amount);
}

pub fn process_midi_pitch_bend_message(
    oscillators: &[OscillatorParameters; 4],
    range: u8,
    bend_amount: u16,
) {
    midi_value_converters::update_current_note_from_midi_pitch_bend(
        bend_amount,
        range,
        oscillators,
    );
}

pub fn process_midi_note_off_message(module_parameters: &mut Arc<ModuleParameters>) {
    action_midi_note_events(MidiNoteEvent::NoteOff, module_parameters);
}

pub fn process_midi_note_on_message(
    module_parameters: &mut Arc<ModuleParameters>,
    current_note: &mut Arc<CurrentNote>,
    midi_note: u8,
    velocity: u8,
    ui_update_sender: &Sender<UIUpdates>,
) {
    let scaled_velocity = scaled_velocity_from_normal_value(
        load_f32_from_atomic_u32(&current_note.velocity_curve),
        normalize_midi_value(velocity),
    );

    store_f32_as_atomic_u32(&current_note.velocity, scaled_velocity);
    current_note.midi_note.store(midi_note, Relaxed);

    module_parameters
        .filter
        .current_note_number
        .store(midi_note, Relaxed);

    action_midi_note_events(MidiNoteEvent::NoteOn, module_parameters);

    let note_name = Defaults::MIDI_NOTE_FREQUENCIES[midi_note as usize]
        .1
        .to_string();
    ui_update_sender.send(UIUpdates::MidiScreen(note_name)).expect("process_midi_note_on_message(): Could not send the MIDI screen message to the UI. Exiting.");
}

// This function has to match every CC value, so it is going to be very long.
#[allow(clippy::too_many_lines)]
pub fn process_midi_cc_values(
    cc_value: CC,
    current_note: &mut Arc<CurrentNote>,
    module_parameters: &mut Arc<ModuleParameters>,
    ui_update_sender: &Sender<UIUpdates>,
) {
    log::trace!("CC received: {cc_value:?}");
    match cc_value {
        CC::ModWheel(value) => {
            set_mod_wheel(&module_parameters.keyboard, normalize_midi_value(value));
        }
        CC::VelocityCurve(value) => {
            let normal_value = normalize_midi_value(value);
            set_velocity_curve(current_note, normalize_midi_value(value));

            ui_update_sender.send(UIUpdates::VelocityCurve(normal_value))
                            .expect("process_cc_midi_values(): Could not send the velocity curve value to the UI. Exiting");
        }
        CC::PitchBendRange(value) => {
            let normal_value = normalize_midi_value(value);
            set_pitch_bend_range(&module_parameters.keyboard, normal_value);

            ui_update_sender
                .send(UIUpdates::PitchBendRange(normal_value))
                .expect(
                    "process_cc_midi_values(): Could not send the pitch bend range value to \
                            the UI. Exiting",
                );
        }
        CC::Volume(value) => {
            let normal_value = normalize_midi_value(value);
            set_output_level(&module_parameters.mixer, normal_value);
            ui_update_sender.send(UIUpdates::OutputMixerLevel(normal_value))
                            .expect(
                                "process_midi_cc_values(): Could not send the output mixer level value to the UI. \
                                Exiting.", );
        }
        CC::Balance(value) => {
            let normal_value = normalize_midi_value(value);
            set_output_balance(&module_parameters.mixer, normal_value);
            ui_update_sender.send(UIUpdates::OutputMixerBalance(normal_value))
                            .expect(
                                "process_midi_cc_values(): Could not send the output mixer balance value to the UI. \
                                Exiting.", );
        }
        CC::Mute(value) => {
            let normal_value = normalize_midi_value(value);
            set_output_mute(&module_parameters.mixer, normal_value);
            ui_update_sender.send(UIUpdates::OutputMixerIsMuted(normal_value))
                            .expect(
                                "process_midi_cc_values(): Could not send the output mixer mute state value to the UI. \
                                Exiting.", );
        }
        CC::SubOscillatorShapeParameter1(value) => {
            let parameter1_value = normalize_midi_value(value);
            let oscillator_index = OscillatorIndex::Sub;
            set_oscillator_shape_parameter1(
                &module_parameters.oscillators[oscillator_index as usize],
                parameter1_value,
            );

            ui_update_sender.send(UIUpdates::OscillatorParameter1(oscillator_index as i32, parameter1_value))
                .expect(
                    "process_midi_cc_values(): Could not send the oscillator-specific parameter 1 value to the UI. \
                    Exiting.",
                );
        }
        CC::SubOscillatorShapeParameter2(value) => {
            let parameter2_value = normalize_midi_value(value);
            let oscillator_index = OscillatorIndex::Sub;
            set_oscillator_shape_parameter2(
                &module_parameters.oscillators[oscillator_index as usize],
                parameter2_value,
            );
            ui_update_sender
                .send(UIUpdates::OscillatorParameter2(oscillator_index as i32, parameter2_value))
                .expect(
                "process_midi_cc_values(): Could not send the oscillator-specific parameter 2 value to the UI. \
                Exiting.",
            );
        }
        CC::Oscillator1ShapeParameter1(value) => {
            let parameter1_value = normalize_midi_value(value);
            let oscillator_index = OscillatorIndex::One;
            set_oscillator_shape_parameter1(
                &module_parameters.oscillators[oscillator_index as usize],
                parameter1_value,
            );
            ui_update_sender
                .send(UIUpdates::OscillatorParameter1(oscillator_index as i32, parameter1_value))
                .expect(
                "process_midi_cc_values(): Could not send the oscillator-specific parameter 1 value to the UI. \
                Exiting.",
            );
        }
        CC::Oscillator1ShapeParameter2(value) => {
            let parameter2_value = normalize_midi_value(value);
            let oscillator_index = OscillatorIndex::One;
            set_oscillator_shape_parameter2(
                &module_parameters.oscillators[oscillator_index as usize],
                parameter2_value,
            );
            ui_update_sender
                .send(UIUpdates::OscillatorParameter2(oscillator_index as i32, parameter2_value))
                .expect(
                "process_midi_cc_values(): Could not send the oscillator-specific parameter 2 value to the UI. \
                Exiting.",
            );
        }
        CC::Oscillator2ShapeParameter1(value) => {
            let parameter1_value = normalize_midi_value(value);
            let oscillator_index = OscillatorIndex::Two;
            set_oscillator_shape_parameter1(
                &module_parameters.oscillators[oscillator_index as usize],
                parameter1_value,
            );
            ui_update_sender
                .send(UIUpdates::OscillatorParameter1(oscillator_index as i32, parameter1_value))
                .expect(
                "process_midi_cc_values(): Could not send the oscillator-specific parameter 1 value to the UI. \
                Exiting.",
            );
        }
        CC::Oscillator2ShapeParameter2(value) => {
            let parameter2_value = normalize_midi_value(value);
            let oscillator_index = OscillatorIndex::Two;
            set_oscillator_shape_parameter2(
                &module_parameters.oscillators[oscillator_index as usize],
                parameter2_value,
            );
            ui_update_sender
                .send(UIUpdates::OscillatorParameter2(oscillator_index as i32, parameter2_value))
                .expect(
                "process_midi_cc_values(): Could not send the oscillator-specific parameter 2 value to the UI. \
                Exiting.",
            );
        }
        CC::Oscillator3ShapeParameter1(value) => {
            let parameter1_value = normalize_midi_value(value);
            let oscillator_index = OscillatorIndex::Three;
            set_oscillator_shape_parameter2(
                &module_parameters.oscillators[oscillator_index as usize],
                parameter1_value,
            );
            ui_update_sender
                .send(UIUpdates::OscillatorParameter1(oscillator_index as i32, parameter1_value))
                .expect(
                "process_midi_cc_values(): Could not send the oscillator-specific parameter 1 value to the UI. \
                Exiting.",
            );
        }
        CC::Oscillator3ShapeParameter2(value) => {
            let parameter2_value = normalize_midi_value(value);
            let oscillator_index = OscillatorIndex::Three;
            set_oscillator_shape_parameter2(
                &module_parameters.oscillators[oscillator_index as usize],
                parameter2_value,
            );
            ui_update_sender
                .send(UIUpdates::OscillatorParameter2(oscillator_index as i32, parameter2_value))
                .expect(
                "process_midi_cc_values(): Could not send the oscillator-specific parameter 2 value to the UI. \
                Exiting.",
            );
        }
        CC::OscillatorKeySyncEnabled(value) => {
            let normal_value = normalize_midi_value(value);
            set_oscillator_key_sync(&module_parameters.oscillators, normal_value);

            ui_update_sender
                .send(UIUpdates::KeySync(normal_value))
                .expect(
                    "process_cc_midi_values(): Could not send the oscillator key sync state to \
                            the UI. Exiting",
                );
        }
        CC::PortamentoTime(value) => {
            let normal_value = normalize_midi_value(value);
            set_portamento_time(&module_parameters.oscillators, normal_value);

            ui_update_sender.send(UIUpdates::PortamentoTime(normal_value))
                            .expect("process_cc_midi_values(): Could not send the poratmento time value to the UI. Exiting");
        }
        CC::OscillatorHardSync(value) => {
            let normal_value = normalize_midi_value(value);
            set_oscillator_hard_sync(&module_parameters.oscillators, normal_value);

            ui_update_sender
                .send(UIUpdates::HardSync(normal_value))
                .expect(
                    "process_cc_midi_values(): Could not send the oscillator hard sync state to \
                            the UI. Exiting",
                );
        }
        CC::SubOscillatorShape(value) => {
            let normal_value = normalize_midi_value(value);
            let oscillator_index = OscillatorIndex::Sub;
            let shape_index = set_oscillator_wave_shape(
                &module_parameters.oscillators[OscillatorIndex::Sub as usize],
                normal_value,
            );
            ui_update_sender
                .send(UIUpdates::OscillatorWaveShape(oscillator_index as i32, shape_index as i32))
                .expect(
                    "process_midi_cc_values(): Could not send the oscillator wave shape value to the UI. Exiting.",
                );
        }
        CC::Oscillator1Shape(value) => {
            let normal_value = normalize_midi_value(value);
            let oscillator_index = OscillatorIndex::One;
            let shape_index = set_oscillator_wave_shape(
                &module_parameters.oscillators[oscillator_index as usize],
                normal_value,
            );
            ui_update_sender
                .send(UIUpdates::OscillatorWaveShape(oscillator_index as i32, shape_index as i32))
                .expect(
                    "process_midi_cc_values(): Could not send the oscillator wave shape value to the UI. Exiting.",
                );
        }
        CC::Oscillator2Shape(value) => {
            let normal_value = normalize_midi_value(value);
            let oscillator_index = OscillatorIndex::Two;
            let shape_index = set_oscillator_wave_shape(
                &module_parameters.oscillators[oscillator_index as usize],
                normal_value,
            );
            ui_update_sender
                .send(UIUpdates::OscillatorWaveShape(oscillator_index as i32, shape_index as i32))
                .expect(
                    "process_midi_cc_values(): Could not send the oscillator wave shape value to the UI. Exiting.",
                );
        }
        CC::Oscillator3Shape(value) => {
            let normal_value = normalize_midi_value(value);
            let oscillator_index = OscillatorIndex::Three;
            let shape_index = set_oscillator_wave_shape(
                &module_parameters.oscillators[oscillator_index as usize],
                normal_value,
            );
            ui_update_sender
                .send(UIUpdates::OscillatorWaveShape(oscillator_index as i32, shape_index as i32))
                .expect(
                    "process_midi_cc_values(): Could not send the oscillator wave shape value to the UI. Exiting.",
                );
        }
        CC::SubOscillatorCourseTune(value) => {
            let oscillator_index = OscillatorIndex::Sub;
            let course_tune = set_oscillator_course_tune(
                &module_parameters.oscillators[oscillator_index as usize],
                normalize_midi_value(value),
            );

            ui_update_sender
                .send(UIUpdates::OscillatorCourseTune(oscillator_index as i32, course_tune as i32))
                .expect(
                    "process_midi_cc_values(): Could not send the oscillator course-tune interval value to the UI. \
                    Exiting.",
                );
        }
        CC::Oscillator1CourseTune(value) => {
            let oscillator_index = OscillatorIndex::One;
            let course_tune = set_oscillator_course_tune(
                &module_parameters.oscillators[oscillator_index as usize],
                normalize_midi_value(value),
            );

            ui_update_sender
                .send(UIUpdates::OscillatorCourseTune(oscillator_index as i32, course_tune as i32))
                .expect(
                    "process_midi_cc_values(): Could not send the oscillator course-tune interval value to the UI. \
                    Exiting.",
                );
        }
        CC::Oscillator2CourseTune(value) => {
            let oscillator_index = OscillatorIndex::Two;
            let course_tune = set_oscillator_course_tune(
                &module_parameters.oscillators[oscillator_index as usize],
                normalize_midi_value(value),
            );

            ui_update_sender
                .send(UIUpdates::OscillatorCourseTune(oscillator_index as i32, course_tune as i32))
                .expect(
                    "process_midi_cc_values(): Could not send the oscillator course-tune interval value to the UI. \
                    Exiting.",
                );
        }
        CC::Oscillator3CourseTune(value) => {
            let oscillator_index = OscillatorIndex::Three;
            let course_tune = set_oscillator_course_tune(
                &module_parameters.oscillators[oscillator_index as usize],
                normalize_midi_value(value),
            );

            ui_update_sender
                .send(UIUpdates::OscillatorCourseTune(oscillator_index as i32, course_tune as i32))
                .expect(
                    "process_midi_cc_values(): Could not send the oscillator course-tune interval value to the UI. \
                    Exiting.",
                );
        }
        CC::SubOscillatorFineTune(value) => {
            let oscillator_index = OscillatorIndex::Sub;
            let fine_tune_normal_value = normalize_midi_value(value);
            let cents = set_oscillator_fine_tune(
                &module_parameters.oscillators[oscillator_index as usize],
                fine_tune_normal_value,
            );

            ui_update_sender
                .send(UIUpdates::OscillatorFineTune(oscillator_index as i32, fine_tune_normal_value,cents as i32))
                .expect(
                    "process_midi_cc_values(): Could not send the oscillator fine-tune value to the UI. Exiting.",
                );
        }
        CC::Oscillator1FineTune(value) => {
            let oscillator_index = OscillatorIndex::One;
            let fine_tune_normal_value = normalize_midi_value(value);
            let cents = set_oscillator_fine_tune(
                &module_parameters.oscillators[oscillator_index as usize],
                fine_tune_normal_value,
            );
            ui_update_sender
                .send(UIUpdates::OscillatorFineTune(oscillator_index as i32, fine_tune_normal_value,cents as i32))
                .expect(
                    "process_midi_cc_values(): Could not send the oscillator fine-tune cents value to the UI. Exiting.",
                );
        }
        CC::Oscillator2FineTune(value) => {
            let oscillator_index = OscillatorIndex::Two;
            let fine_tune_normal_value = normalize_midi_value(value);
            let cents = set_oscillator_fine_tune(
                &module_parameters.oscillators[oscillator_index as usize],
                fine_tune_normal_value,
            );
            ui_update_sender
                .send(UIUpdates::OscillatorFineTune(oscillator_index as i32, fine_tune_normal_value, cents as i32))
                .expect(
                    "process_midi_cc_values(): Could not send the oscillator fine-tune cents value to the UI. Exiting.",
                );
        }
        CC::Oscillator3FineTune(value) => {
            let oscillator_index = OscillatorIndex::Three;
            let fine_tune_normal_value = normalize_midi_value(value);
            let cents = set_oscillator_fine_tune(
                &module_parameters.oscillators[oscillator_index as usize],
                fine_tune_normal_value,
            );

            ui_update_sender
                .send(UIUpdates::OscillatorFineTune(oscillator_index as i32, fine_tune_normal_value, cents as
                    i32))
                .expect(
                    "process_midi_cc_values(): Could not send the oscillator fine-tune cents value to the UI. Exiting.",
                );
        }
        CC::SubOscillatorLevel(value) => {
            let normal_value = normalize_midi_value(value);
            set_oscillator_level(&module_parameters.mixer, OscillatorIndex::Sub, normal_value);
            ui_update_sender.send(UIUpdates::OscillatorMixerLevel(OscillatorIndex::Sub as i32,
                                                                              normal_value))
                .expect("process_cc_midi_values(): Could not send the oscillator mixer sub level value to the UI. \
                Exiting");
        }
        CC::Oscillator1Level(value) => {
            let normal_value = normalize_midi_value(value);
            set_oscillator_level(&module_parameters.mixer, OscillatorIndex::One, normal_value);
            ui_update_sender.send(UIUpdates::OscillatorMixerLevel(OscillatorIndex::One as i32,
                                                                              normal_value))
                .expect("process_cc_midi_values(): Could not send the oscillator mixer 1 level value to the UI. \
                Exiting");
        }
        CC::Oscillator2Level(value) => {
            let normal_value = normalize_midi_value(value);
            set_oscillator_level(&module_parameters.mixer, OscillatorIndex::Two, normal_value);
            ui_update_sender.send(UIUpdates::OscillatorMixerLevel(OscillatorIndex::Two as i32,
                                                                              normal_value))
                .expect("process_cc_midi_values(): Could not send the oscillator mixer 2 level value to the UI. \
                Exiting");
        }
        CC::Oscillator3Level(value) => {
            let normal_value = normalize_midi_value(value);
            set_oscillator_level(
                &module_parameters.mixer,
                OscillatorIndex::Three,
                normal_value,
            );
            ui_update_sender.send(UIUpdates::OscillatorMixerLevel(OscillatorIndex::Three as i32,
                                                                              normal_value))
                .expect("process_cc_midi_values(): Could not send the oscillator mixer 3 level value to the UI. \
                Exiting");
        }
        CC::SubOscillatorMute(value) => {
            let normal_value = normalize_midi_value(value);
            set_oscillator_mute(&module_parameters.mixer, OscillatorIndex::Sub, normal_value);
            ui_update_sender.send(UIUpdates::OscillatorMixerIsMuted(OscillatorIndex::Sub as i32,
                                                                              normal_value))
                .expect("process_cc_midi_values(): Could not send the oscillator mixer sub mute state to the UI. \
                Exiting");
        }
        CC::Oscillator1Mute(value) => {
            let normal_value = normalize_midi_value(value);
            set_oscillator_mute(&module_parameters.mixer, OscillatorIndex::One, normal_value);
            ui_update_sender.send(UIUpdates::OscillatorMixerIsMuted(OscillatorIndex::One as i32,
                                                                              normal_value))
                .expect("process_cc_midi_values(): Could not send the oscillator mixer 1 mute state to the UI. \
                Exiting");
        }
        CC::Oscillator2Mute(value) => {
            let normal_value = normalize_midi_value(value);
            set_oscillator_mute(&module_parameters.mixer, OscillatorIndex::Two, normal_value);
            ui_update_sender.send(UIUpdates::OscillatorMixerIsMuted(OscillatorIndex::Two as i32,
                                                                              normal_value))
                .expect("process_cc_midi_values(): Could not send the oscillator mixer 2 mute state to the UI. \
                Exiting");
        }
        CC::Oscillator3Mute(value) => {
            let normal_value = normalize_midi_value(value);
            set_oscillator_mute(
                &module_parameters.mixer,
                OscillatorIndex::Three,
                normal_value,
            );
            ui_update_sender.send(UIUpdates::OscillatorMixerIsMuted(OscillatorIndex::Three as i32,
                                                                              normal_value))
                .expect("process_cc_midi_values(): Could not send the oscillator mixer 3 mute state to the UI. \
                Exiting");
        }
        CC::SubOscillatorBalance(value) => {
            let normal_value = normalize_midi_value(value);
            set_oscillator_balance(&module_parameters.mixer, OscillatorIndex::Sub, normal_value);
            ui_update_sender.send(UIUpdates::OscillatorMixerBalance(OscillatorIndex::Sub as i32, normal_value))
                .expect("process_cc_midi_values(): Could not send the oscillator mixer sub balance value to the UI. \
                Exiting");
        }
        CC::Oscillator1Balance(value) => {
            let normal_value = normalize_midi_value(value);
            set_oscillator_balance(&module_parameters.mixer, OscillatorIndex::One, normal_value);
            ui_update_sender.send(UIUpdates::OscillatorMixerBalance(OscillatorIndex::One as i32, normal_value))
                .expect("process_cc_midi_values(): Could not send the oscillator mixer 1 balance  value to the UI. \
                Exiting");
        }
        CC::Oscillator2Balance(value) => {
            let normal_value = normalize_midi_value(value);
            set_oscillator_balance(&module_parameters.mixer, OscillatorIndex::Two, normal_value);
            ui_update_sender.send(UIUpdates::OscillatorMixerBalance(OscillatorIndex::Two as i32, normal_value))
                .expect("process_cc_midi_values(): Could not send the oscillator mixer 2 balance value to the UI. \
                Exiting");
        }
        CC::Oscillator3Balance(value) => {
            let normal_value = normalize_midi_value(value);
            set_oscillator_balance(
                &module_parameters.mixer,
                OscillatorIndex::Three,
                normal_value,
            );
            ui_update_sender.send(UIUpdates::OscillatorMixerBalance(OscillatorIndex::Three as i32, normal_value))
                .expect("process_cc_midi_values(): Could not send the oscillator mixer 3 balance value to the UI. \
                Exiting");
        }
        CC::PortamentoEnabled(value) => {
            let normal_value = normalize_midi_value(value);
            set_portamento_enabled(&module_parameters.oscillators, normal_value);

            ui_update_sender.send(UIUpdates::PortamentoEnabled(normal_value))
                            .expect("process_cc_midi_values(): Could not send the poratmento enabled state to the UI. \
                            Exiting");
        }
        CC::SubOscillatorClipBoost(value) => {
            let boost_level = normalize_midi_value(value);
            set_oscillator_clip_boost(
                &module_parameters.oscillators[OscillatorIndex::Sub as usize],
                boost_level,
            );
            ui_update_sender
                .send(UIUpdates::OscillatorClipperBoost(OscillatorIndex::Sub as i32, boost_level))
                .expect(
                    "process_midi_cc_values(): Could not send the oscillator clipper boost level value to the UI. \
                    Exiting.",
                );
        }
        CC::Oscillator1ClipBoost(value) => {
            let boost_level = normalize_midi_value(value);
            set_oscillator_clip_boost(
                &module_parameters.oscillators[OscillatorIndex::One as usize],
                normalize_midi_value(value),
            );
            ui_update_sender
                .send(UIUpdates::OscillatorClipperBoost(OscillatorIndex::One as i32, boost_level))
                .expect(
                    "process_midi_cc_values(): Could not send the oscillator clipper boost level value to the UI. \
                    Exiting.",
                );
        }
        CC::Oscillator2ClipBoost(value) => {
            let boost_level = normalize_midi_value(value);
            set_oscillator_clip_boost(
                &module_parameters.oscillators[OscillatorIndex::Two as usize],
                normalize_midi_value(value),
            );

            ui_update_sender
                .send(UIUpdates::OscillatorClipperBoost(OscillatorIndex::Two as i32, boost_level))
                .expect(
                    "process_midi_cc_values(): Could not send the oscillator clipper boost level value to the UI. \
                    Exiting.",
            );
        }
        CC::Oscillator3ClipBoost(value) => {
            let boost_level = normalize_midi_value(value);
            set_oscillator_clip_boost(
                &module_parameters.oscillators[OscillatorIndex::Three as usize],
                normalize_midi_value(value),
            );
            ui_update_sender
                .send(UIUpdates::OscillatorClipperBoost(OscillatorIndex::Three as i32, boost_level))
                .expect(
                    "process_midi_cc_values(): Could not send the oscillator clipper boost level value to the UI. \
                    Exiting.",
            );
        }
        CC::FilterPoles(value) => {
            let normal_value = normalize_midi_value(value);
            set_filter_poles(&module_parameters.filter, normal_value);
            ui_update_sender
                .send(UIUpdates::FilterPoles(normal_value))
                .expect(
                    "process_midi_cc_values(): Could not send the filter poles value to the UI. \
                    Exiting.",
                );
        }
        CC::FilterResonance(value) => {
            let normal_value = normalize_midi_value(value);
            set_filter_resonance(&module_parameters.filter, normal_value);
            ui_update_sender
                .send(UIUpdates::FilterResonance(normal_value))
                .expect(
                    "process_midi_cc_values(): Could not send the filter resonance value to the UI. \
                    Exiting.",
                );
        }
        CC::AmpEGReleaseTime(value) => {
            let normal_value = normalize_midi_value(value);
            set_envelope_release_time(&module_parameters.amp_envelope, normal_value);
            ui_update_sender
                .send(UIUpdates::EnvelopeReleaseTime(EnvelopeIndex::Amp as i32, normal_value))
                .expect(
                    "process_midi_cc_values(): Could not send the amp envelope release time value to the UI. \
                    Exiting.",
                );
        }
        CC::AmpEGAttackTime(value) => {
            let normal_value = normalize_midi_value(value);
            set_envelope_attack_time(&module_parameters.amp_envelope, normal_value);
            ui_update_sender
                .send(UIUpdates::EnvelopeAttackTime(EnvelopeIndex::Amp as i32, normal_value))
                .expect(
                    "process_midi_cc_values(): Could not send the amp envelope attack time value to the UI. \
                    Exiting.",
                );
        }
        CC::FilterCutoff(value) => {
            let normal_value = normalize_midi_value(value);
            set_filter_cutoff(&module_parameters.filter, normal_value);
            ui_update_sender
                .send(UIUpdates::FilterCutoff(normal_value))
                .expect(
                    "process_midi_cc_values(): Could not send the filter cutoff value to the UI. \
                    Exiting.",
                );
        }
        CC::AmpEGDecayTime(value) => {
            let normal_value = normalize_midi_value(value);
            set_envelope_decay_time(&module_parameters.amp_envelope, normal_value);
            ui_update_sender
                .send(UIUpdates::EnvelopeDecayTime(EnvelopeIndex::Amp as i32, normal_value))
                .expect(
                    "process_midi_cc_values(): Could not send the amp envelope decay time value to the UI. \
                    Exiting.",
                );
        }
        CC::AmpEGSustainLevel(value) => {
            let normal_value = normalize_midi_value(value);
            set_envelope_sustain_level(&module_parameters.amp_envelope, normal_value);

            ui_update_sender
                .send(UIUpdates::EnvelopeSustainLevel(EnvelopeIndex::Amp as i32, normal_value))
                .expect(
                    "process_midi_cc_values(): Could not send the amp envelope sustain level value to the UI. \
                    Exiting.",
                );
        }
        CC::AmpEGInverted(value) => {
            let normal_value = normalize_midi_value(value);
            set_envelope_inverted(&module_parameters.amp_envelope, normal_value);

            ui_update_sender
                .send(UIUpdates::EnvelopeInverted(EnvelopeIndex::Amp as i32, normal_value))
                .expect(
                    "process_midi_cc_values(): Could not send the amp envelope inverted state value to the UI. \
                    Exiting.",
                );
        }
        CC::FilterEnvelopeAttackTime(value) => {
            let normal_value = normalize_midi_value(value);
            set_envelope_attack_time(&module_parameters.filter_envelope, normal_value);
            ui_update_sender
                .send(UIUpdates::EnvelopeAttackTime(EnvelopeIndex::Filter as i32, normal_value))
                .expect(
                    "process_midi_cc_values(): Could not send the filter envelope attack time value to the UI. \
                    Exiting.",
                );
        }
        CC::FilterEnvelopeDecayTime(value) => {
            let normal_value = normalize_midi_value(value);
            set_envelope_decay_time(&module_parameters.filter_envelope, normal_value);
            ui_update_sender
                .send(UIUpdates::EnvelopeDecayTime(EnvelopeIndex::Filter as i32, normal_value))
                .expect(
                    "process_midi_cc_values(): Could not send the filter envelope decay time value to the UI. \
                    Exiting.",
                );
        }
        CC::FilterEnvelopeSustainLevel(value) => {
            let normal_value = normalize_midi_value(value);
            set_envelope_sustain_level(&module_parameters.filter_envelope, normal_value);
            ui_update_sender
                .send(UIUpdates::EnvelopeSustainLevel(EnvelopeIndex::Filter as i32, normal_value))
                .expect(
                    "process_midi_cc_values(): Could not send the filter envelope sustain level value to the UI. \
                    Exiting.",
                );
        }
        CC::FilterEnvelopeReleaseTime(value) => {
            let normal_value = normalize_midi_value(value);
            set_envelope_release_time(&module_parameters.filter_envelope, normal_value);
            ui_update_sender
                .send(UIUpdates::EnvelopeReleaseTime(EnvelopeIndex::Filter as i32, normal_value))
                .expect(
                    "process_midi_cc_values(): Could not send the filter envelope release time value to the UI. \
                    Exiting.",
                );
        }
        CC::FilterEnvelopeInverted(value) => {
            let normal_value = normalize_midi_value(value);
            set_envelope_inverted(&module_parameters.filter_envelope, normal_value);
            ui_update_sender
                .send(UIUpdates::EnvelopeInverted(EnvelopeIndex::Filter as i32, normal_value))
                .expect(
                    "process_midi_cc_values(): Could not send the filter envelope inverted state value to the UI. \
                    Exiting.",
                );
        }
        CC::FilterEnvelopeAmount(value) => {
            let normal_value = normalize_midi_value(value);
            set_envelope_amount(&module_parameters.filter_envelope, normal_value);
            ui_update_sender
                .send(UIUpdates::FilterEnvelopeAmount(normal_value))
                .expect(
                    "process_midi_cc_values(): Could not send the filter envelope amount value to the UI. \
                    Exiting.",
                );
        }
        CC::KeyTrackingAmount(value) => {
            let normal_value = normalize_midi_value(value);
            set_key_tracking_amount(&module_parameters.filter, normal_value);
            ui_update_sender
                .send(UIUpdates::FilterKeyTracking(normal_value))
                .expect(
                    "process_midi_cc_values(): Could not send the filter key tracking value to the UI. \
                    Exiting.",
                );
        }
        CC::ModWheelLFOFrequency(value) => {
            let normal_value = normalize_midi_value(value);
            set_lfo_frequency(&module_parameters.mod_wheel_lfo, normal_value);

            ui_update_sender
                .send(UIUpdates::LFOFrequency(LFOIndex::ModWheel as i32, normal_value))
                .expect(
                    "process_midi_cc_values(): Could not send the mod wheel lfo frequency level value to the UI. \
                    Exiting.",
                );
        }
        CC::ModWheelLFOCenterValue(value) => {
            let normal_value = normalize_midi_value(value);
            set_lfo_center_value(&module_parameters.mod_wheel_lfo, normal_value);
        }
        CC::ModWheelLFORange(value) => {
            let normal_value = normalize_midi_value(value);
            set_lfo_range(&module_parameters.mod_wheel_lfo, normal_value);
        }
        CC::ModWheelLFOWaveShape(value) => {
            let normal_value = normalize_midi_value(value);
            set_lfo_wave_shape(&module_parameters.mod_wheel_lfo, normal_value);

            ui_update_sender
                .send(UIUpdates::LFOWaveShape(LFOIndex::ModWheel as i32, normal_value))
                .expect(
                    "process_midi_cc_values(): Could not send the mod wheel lfo wave shape value to the UI. \
                    Exiting.",
                );
        }
        CC::ModWheelLFOPhase(value) => {
            let normal_value = normalize_midi_value(value);
            set_lfo_phase(&module_parameters.mod_wheel_lfo, normal_value);

            ui_update_sender
                .send(UIUpdates::LFOPhase(LFOIndex::ModWheel as i32, normal_value))
                .expect(
                    "process_midi_cc_values(): Could not send the mod wheel lfo phase value to the UI. \
                    Exiting.",
                );
        }
        CC::ModWheelLFOReset => {
            set_lfo_phase_reset(&module_parameters.mod_wheel_lfo);
            ui_update_sender
                .send(UIUpdates::LFOPhase(LFOIndex::ModWheel as i32, DEFAULT_LFO_PHASE))
                .expect(
                    "process_midi_cc_values(): Could not send the mod wheel phase reset value to the UI. \
                    Exiting.",
                );
        }
        CC::FilterModLFOFrequency(value) => {
            let normal_value = normalize_midi_value(value);
            set_lfo_frequency(&module_parameters.filter_lfo, normal_value);

            ui_update_sender
                .send(UIUpdates::LFOFrequency(LFOIndex::Filter as i32, normal_value))
                .expect(
                    "process_midi_cc_values(): Could not send the filter lfo frequency value to the UI. \
                    Exiting.",
                );
        }
        CC::FilterModLFOAmount(value) => {
            let normal_value = normalize_midi_value(value);
            set_lfo_range(&module_parameters.filter_lfo, normal_value);
            ui_update_sender
                .send(UIUpdates::FilterLFOAmount(normal_value))
                .expect(
                    "process_midi_cc_values(): Could not send the filter lfo amount value to the UI. \
                    Exiting.",
                );
        }
        CC::FilterModLFOWaveShape(value) => {
            let normal_value = normalize_midi_value(value);
            set_lfo_wave_shape(&module_parameters.filter_lfo, normal_value);

            ui_update_sender
                .send(UIUpdates::LFOWaveShape(LFOIndex::Filter as i32, normal_value))
                .expect(
                    "process_midi_cc_values(): Could not send the filter lfo wave shape value to the UI. \
                    Exiting.",
                );
        }
        CC::FilterModLFOPhase(value) => {
            let normal_value = normalize_midi_value(value);
            set_lfo_phase(&module_parameters.filter_lfo, normal_value);

            ui_update_sender
                .send(UIUpdates::LFOPhase(LFOIndex::Filter as i32, normal_value))
                .expect(
                    "process_midi_cc_values(): Could not send the filter lfo phase value to the UI. \
                    Exiting.",
                );
        }
        CC::FilterModLFOReset => {
            set_lfo_phase_reset(&module_parameters.filter_lfo);

            ui_update_sender
                .send(UIUpdates::LFOPhase(LFOIndex::Filter as i32, DEFAULT_LFO_PHASE))
                .expect(
                    "process_midi_cc_values(): Could not send the filter lfo phase reset  value to the UI. \
                    Exiting.",
                );
        }
        CC::AllNotesOff => {
            process_midi_note_off_message(module_parameters);
        }
    }
}
