use crate::math::{load_f32_from_atomic_u32, normalize_midi_value, store_f32_as_atomic_u32};
use crate::midi::control_change::CC;
use crate::modules::oscillator::OscillatorParameters;
use crate::synthesizer::midi_value_converters::scaled_velocity_from_normal_value;
use crate::synthesizer::set_parameters::{
    lfo_reset, set_envelope_amount, set_envelope_attack_time, set_envelope_decay_time,
    set_envelope_inverted, set_envelope_release_time, set_envelope_sustain_level,
    set_filter_cutoff, set_filter_poles, set_filter_resonance, set_key_tracking_amount,
    set_lfo_center_value, set_lfo_frequency, set_lfo_phase, set_lfo_range, set_lfo_wave_shape,
    set_mod_wheel, set_oscillator_balance, set_oscillator_clip_boost, set_oscillator_course_tune,
    set_oscillator_fine_tune, set_oscillator_hard_sync, set_oscillator_key_sync,
    set_oscillator_level, set_oscillator_mute, set_oscillator_shape_parameter1,
    set_oscillator_shape_parameter2, set_oscillator_wave_shape, set_output_balance,
    set_output_volume, set_pitch_bend_range, set_portamento_enabled, set_portamento_time,
    set_velocity_curve,
};
use crate::synthesizer::{
    CurrentNote, KeyboardParameters, MidiGateEvent, MidiNoteEvent, ModuleParameters,
    OscillatorIndex, midi_value_converters,
};
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
}

// This function has to match every CC value, so it is going to be very long.
#[allow(clippy::too_many_lines)]
pub fn process_midi_cc_values(
    cc_value: CC,
    current_note: &mut Arc<CurrentNote>,
    module_parameters: &mut Arc<ModuleParameters>,
) {
    log::debug!("process_midi_cc_values(): CC received: {cc_value:?}");
    match cc_value {
        CC::ModWheel(value) => {
            set_mod_wheel(&module_parameters.keyboard, normalize_midi_value(value));
        }
        CC::VelocityCurve(value) => {
            set_velocity_curve(current_note, normalize_midi_value(value));
        }
        CC::PitchBendRange(value) => {
            set_pitch_bend_range(&module_parameters.keyboard, normalize_midi_value(value));
        }
        CC::Volume(value) => {
            set_output_volume(&module_parameters.mixer, normalize_midi_value(value));
        }
        CC::Balance(value) => {
            set_output_balance(&module_parameters.mixer, normalize_midi_value(value));
        }
        CC::SubOscillatorShapeParameter1(value) => {
            set_oscillator_shape_parameter1(
                &module_parameters.oscillators[OscillatorIndex::Sub as usize],
                normalize_midi_value(value),
            );
        }
        CC::SubOscillatorShapeParameter2(value) => {
            set_oscillator_shape_parameter2(
                &module_parameters.oscillators[OscillatorIndex::Sub as usize],
                normalize_midi_value(value),
            );
        }
        CC::Oscillator1ShapeParameter1(value) => {
            set_oscillator_shape_parameter1(
                &module_parameters.oscillators[OscillatorIndex::One as usize],
                normalize_midi_value(value),
            );
        }
        CC::Oscillator1ShapeParameter2(value) => {
            set_oscillator_shape_parameter2(
                &module_parameters.oscillators[OscillatorIndex::One as usize],
                normalize_midi_value(value),
            );
        }
        CC::Oscillator2ShapeParameter1(value) => {
            set_oscillator_shape_parameter1(
                &module_parameters.oscillators[OscillatorIndex::Two as usize],
                normalize_midi_value(value),
            );
        }
        CC::Oscillator2ShapeParameter2(value) => {
            set_oscillator_shape_parameter2(
                &module_parameters.oscillators[OscillatorIndex::Two as usize],
                normalize_midi_value(value),
            );
        }
        CC::Oscillator3ShapeParameter1(value) => {
            set_oscillator_shape_parameter1(
                &module_parameters.oscillators[OscillatorIndex::Three as usize],
                normalize_midi_value(value),
            );
        }
        CC::Oscillator3ShapeParameter2(value) => {
            set_oscillator_shape_parameter2(
                &module_parameters.oscillators[OscillatorIndex::Three as usize],
                normalize_midi_value(value),
            );
        }
        CC::OscillatorKeySyncEnabled(value) => {
            set_oscillator_key_sync(&module_parameters.oscillators, normalize_midi_value(value));
        }
        CC::PortamentoTime(value) => {
            set_portamento_time(&module_parameters.oscillators, normalize_midi_value(value));
        }
        CC::OscillatorHardSync(value) => {
            set_oscillator_hard_sync(&module_parameters.oscillators, normalize_midi_value(value));
        }
        CC::SubOscillatorShape(value) => {
            set_oscillator_wave_shape(
                &module_parameters.oscillators[OscillatorIndex::Sub as usize],
                normalize_midi_value(value),
            );
        }
        CC::Oscillator1Shape(value) => {
            set_oscillator_wave_shape(
                &module_parameters.oscillators[OscillatorIndex::One as usize],
                normalize_midi_value(value),
            );
        }
        CC::Oscillator2Shape(value) => {
            set_oscillator_wave_shape(
                &module_parameters.oscillators[OscillatorIndex::Two as usize],
                normalize_midi_value(value),
            );
        }
        CC::Oscillator3Shape(value) => {
            set_oscillator_wave_shape(
                &module_parameters.oscillators[OscillatorIndex::Three as usize],
                normalize_midi_value(value),
            );
        }
        CC::SubOscillatorCourseTune(value) => {
            set_oscillator_course_tune(
                &module_parameters.oscillators[OscillatorIndex::Sub as usize],
                normalize_midi_value(value),
            );
        }
        CC::Oscillator1CourseTune(value) => {
            set_oscillator_course_tune(
                &module_parameters.oscillators[OscillatorIndex::One as usize],
                normalize_midi_value(value),
            );
        }
        CC::Oscillator2CourseTune(value) => {
            set_oscillator_course_tune(
                &module_parameters.oscillators[OscillatorIndex::Two as usize],
                normalize_midi_value(value),
            );
        }
        CC::Oscillator3CourseTune(value) => {
            set_oscillator_course_tune(
                &module_parameters.oscillators[OscillatorIndex::Three as usize],
                normalize_midi_value(value),
            );
        }
        CC::SubOscillatorFineTune(value) => {
            set_oscillator_fine_tune(
                &module_parameters.oscillators[OscillatorIndex::Sub as usize],
                normalize_midi_value(value),
            );
        }
        CC::Oscillator1FineTune(value) => {
            set_oscillator_fine_tune(
                &module_parameters.oscillators[OscillatorIndex::One as usize],
                normalize_midi_value(value),
            );
        }
        CC::Oscillator2FineTune(value) => {
            set_oscillator_fine_tune(
                &module_parameters.oscillators[OscillatorIndex::Two as usize],
                normalize_midi_value(value),
            );
        }
        CC::Oscillator3FineTune(value) => {
            set_oscillator_fine_tune(
                &module_parameters.oscillators[OscillatorIndex::Three as usize],
                normalize_midi_value(value),
            );
        }
        CC::SubOscillatorLevel(value) => {
            set_oscillator_level(
                &module_parameters.mixer,
                OscillatorIndex::Sub,
                normalize_midi_value(value),
            );
        }
        CC::Oscillator1Level(value) => {
            set_oscillator_level(
                &module_parameters.mixer,
                OscillatorIndex::One,
                normalize_midi_value(value),
            );
        }
        CC::Oscillator2Level(value) => {
            set_oscillator_level(
                &module_parameters.mixer,
                OscillatorIndex::Two,
                normalize_midi_value(value),
            );
        }
        CC::Oscillator3Level(value) => {
            set_oscillator_level(
                &module_parameters.mixer,
                OscillatorIndex::Three,
                normalize_midi_value(value),
            );
        }
        CC::SubOscillatorMute(value) => {
            set_oscillator_mute(
                &module_parameters.mixer,
                OscillatorIndex::Sub,
                normalize_midi_value(value),
            );
        }
        CC::Oscillator1Mute(value) => {
            set_oscillator_mute(
                &module_parameters.mixer,
                OscillatorIndex::One,
                normalize_midi_value(value),
            );
        }
        CC::Oscillator2Mute(value) => {
            set_oscillator_mute(
                &module_parameters.mixer,
                OscillatorIndex::Two,
                normalize_midi_value(value),
            );
        }
        CC::Oscillator3Mute(value) => {
            set_oscillator_mute(
                &module_parameters.mixer,
                OscillatorIndex::Three,
                normalize_midi_value(value),
            );
        }
        CC::SubOscillatorBalance(value) => {
            set_oscillator_balance(
                &module_parameters.mixer,
                OscillatorIndex::Sub,
                normalize_midi_value(value),
            );
        }
        CC::Oscillator1Balance(value) => {
            set_oscillator_balance(
                &module_parameters.mixer,
                OscillatorIndex::One,
                normalize_midi_value(value),
            );
        }
        CC::Oscillator2Balance(value) => {
            set_oscillator_balance(
                &module_parameters.mixer,
                OscillatorIndex::Two,
                normalize_midi_value(value),
            );
        }
        CC::Oscillator3Balance(value) => {
            set_oscillator_balance(
                &module_parameters.mixer,
                OscillatorIndex::Three,
                normalize_midi_value(value),
            );
        }
        CC::PortamentoEnabled(value) => {
            set_portamento_enabled(&module_parameters.oscillators, normalize_midi_value(value));
        }
        CC::SubOscillatorClipBoost(value) => {
            set_oscillator_clip_boost(
                &module_parameters.oscillators,
                OscillatorIndex::Sub,
                normalize_midi_value(value),
            );
        }
        CC::Oscillator1ClipBoost(value) => {
            set_oscillator_clip_boost(
                &module_parameters.oscillators,
                OscillatorIndex::One,
                normalize_midi_value(value),
            );
        }
        CC::Oscillator2ClipBoost(value) => {
            set_oscillator_clip_boost(
                &module_parameters.oscillators,
                OscillatorIndex::Two,
                normalize_midi_value(value),
            );
        }
        CC::Oscillator3ClipBoost(value) => {
            set_oscillator_clip_boost(
                &module_parameters.oscillators,
                OscillatorIndex::Three,
                normalize_midi_value(value),
            );
        }
        CC::FilterPoles(value) => {
            set_filter_poles(&module_parameters.filter, normalize_midi_value(value));
        }
        CC::FilterResonance(value) => {
            set_filter_resonance(&module_parameters.filter, normalize_midi_value(value));
        }
        CC::AmpEGReleaseTime(value) => {
            set_envelope_release_time(&module_parameters.amp_envelope, normalize_midi_value(value));
        }
        CC::AmpEGAttackTime(value) => {
            set_envelope_attack_time(&module_parameters.amp_envelope, normalize_midi_value(value));
        }
        CC::FilterCutoff(value) => {
            set_filter_cutoff(&module_parameters.filter, normalize_midi_value(value));
        }
        CC::AmpEGDecayTime(value) => {
            set_envelope_decay_time(&module_parameters.amp_envelope, normalize_midi_value(value));
        }
        CC::AmpEGSustainLevel(value) => {
            set_envelope_sustain_level(
                &module_parameters.amp_envelope,
                normalize_midi_value(value),
            );
        }
        CC::AmpEGInverted(value) => {
            set_envelope_inverted(&module_parameters.amp_envelope, normalize_midi_value(value));
        }
        CC::FilterEGAttackTime(value) => {
            set_envelope_attack_time(
                &module_parameters.filter_envelope,
                normalize_midi_value(value),
            );
        }
        CC::FilterEGDecayTime(value) => {
            set_envelope_decay_time(
                &module_parameters.filter_envelope,
                normalize_midi_value(value),
            );
        }
        CC::FilterEGSustainLevel(value) => {
            set_envelope_sustain_level(
                &module_parameters.filter_envelope,
                normalize_midi_value(value),
            );
        }
        CC::FilterEGReleaseTime(value) => {
            set_envelope_release_time(
                &module_parameters.filter_envelope,
                normalize_midi_value(value),
            );
        }
        CC::FilterEGInverted(value) => {
            set_envelope_inverted(
                &module_parameters.filter_envelope,
                normalize_midi_value(value),
            );
        }
        CC::FilterEGAmount(value) => {
            set_envelope_amount(
                &module_parameters.filter_envelope,
                normalize_midi_value(value),
            );
        }
        CC::KeyTrackingAmount(value) => {
            set_key_tracking_amount(&module_parameters.filter, normalize_midi_value(value));
        }
        CC::LFO1Frequency(value) => {
            set_lfo_frequency(&module_parameters.lfo1, normalize_midi_value(value));
        }
        CC::LFO1CenterValue(value) => {
            set_lfo_center_value(&module_parameters.lfo1, normalize_midi_value(value));
        }
        CC::LFO1Range(value) => {
            set_lfo_range(&module_parameters.lfo1, normalize_midi_value(value));
        }
        CC::LFO1WaveShape(value) => {
            set_lfo_wave_shape(&module_parameters.lfo1, normalize_midi_value(value));
        }
        CC::LFO1Phase(value) => {
            set_lfo_phase(&module_parameters.lfo1, normalize_midi_value(value));
        }
        CC::LFO1Reset => {
            lfo_reset(&module_parameters.lfo1);
        }
        CC::FilterModLFOFrequency(value) => {
            set_lfo_frequency(&module_parameters.filter_lfo, normalize_midi_value(value));
        }
        CC::FilterModLFOAmount(value) => {
            set_lfo_range(&module_parameters.filter_lfo, normalize_midi_value(value));
        }
        CC::FilterModLFOWaveShape(value) => {
            set_lfo_wave_shape(&module_parameters.filter_lfo, normalize_midi_value(value));
        }
        CC::FilterModLFOPhase(value) => {
            set_lfo_phase(&module_parameters.filter_lfo, normalize_midi_value(value));
        }
        CC::FilterModLFOReset => {
            lfo_reset(&module_parameters.filter_lfo);
        }
        CC::AllNotesOff => {
            process_midi_note_off_message(module_parameters);
        }
    }
}
