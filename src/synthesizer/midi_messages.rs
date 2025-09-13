use crate::math::store_f32_as_atomic_u32;
use crate::midi::CC;
use crate::modules::lfo::LfoParameters;
use crate::modules::oscillator::OscillatorParameters;
use crate::modules::oscillator::triangle::{
    MAX_PORTAMENTO_SPEED_IN_BUFFERS, MIN_PORTAMENTO_SPEED_IN_BUFFERS,
};
use crate::synthesizer::constants::{
    MAX_FILTER_RESONANCE, MAX_PITCH_BEND_RANGE, MIN_FILTER_RESONANCE, MIN_PITCH_BEND_RANGE,
    OSCILLATOR_COURSE_TUNE_MAX_INTERVAL, OSCILLATOR_COURSE_TUNE_MIN_INTERVAL,
    OSCILLATOR_FINE_TUNE_MAX_CENTS, OSCILLATOR_FINE_TUNE_MIN_CENTS,
};
use crate::synthesizer::{
    CurrentNote, EnvelopeParameters, FilterParameters, KeyboardParameters, MidiGateEvent,
    MidiNoteEvent, MixerParameters, ModuleParameters, OscillatorIndex, midi_value_converters,
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
    let aftertouch_amount = midi_value_converters::midi_value_to_f32_0_to_1(pressure_value);
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
    let scaled_velocity =
        midi_value_converters::continuously_variable_curve_mapping_from_midi_value(
            current_note.velocity_curve.load(Relaxed),
            velocity,
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
            set_mod_wheel(&module_parameters.keyboard, value);
        }
        CC::VelocityCurve(value) => {
            set_velocity_curve(current_note, value);
        }
        CC::PitchBendRange(value) => {
            set_pitch_bend_range(&module_parameters.keyboard, value);
        }
        CC::Volume(value) => {
            set_output_volume(&module_parameters.mixer, value);
        }
        CC::Balance(value) => {
            set_output_balance(&module_parameters.mixer, value);
        }
        CC::SubOscillatorShapeParameter1(value) => {
            set_oscillator_shape_parameter1(
                &module_parameters.oscillators[OscillatorIndex::Sub as usize],
                value,
            );
        }
        CC::SubOscillatorShapeParameter2(value) => {
            set_oscillator_shape_parameter2(
                &module_parameters.oscillators[OscillatorIndex::Sub as usize],
                value,
            );
        }
        CC::Oscillator1ShapeParameter1(value) => {
            set_oscillator_shape_parameter1(
                &module_parameters.oscillators[OscillatorIndex::One as usize],
                value,
            );
        }
        CC::Oscillator1ShapeParameter2(value) => {
            set_oscillator_shape_parameter2(
                &module_parameters.oscillators[OscillatorIndex::One as usize],
                value,
            );
        }
        CC::Oscillator2ShapeParameter1(value) => {
            set_oscillator_shape_parameter1(
                &module_parameters.oscillators[OscillatorIndex::Two as usize],
                value,
            );
        }
        CC::Oscillator2ShapeParameter2(value) => {
            set_oscillator_shape_parameter2(
                &module_parameters.oscillators[OscillatorIndex::Two as usize],
                value,
            );
        }
        CC::Oscillator3ShapeParameter1(value) => {
            set_oscillator_shape_parameter1(
                &module_parameters.oscillators[OscillatorIndex::Three as usize],
                value,
            );
        }
        CC::Oscillator3ShapeParameter2(value) => {
            set_oscillator_shape_parameter2(
                &module_parameters.oscillators[OscillatorIndex::Three as usize],
                value,
            );
        }
        CC::OscillatorKeySyncEnabled(value) => {
            set_oscillator_key_sync(&module_parameters.oscillators, value);
        }
        CC::PortamentoTime(value) => {
            set_portamento_time(&module_parameters.oscillators, value);
        }
        CC::OscillatorHardSync(value) => {
            set_oscillator_hard_sync(&module_parameters.oscillators, value);
        }
        CC::SubOscillatorShape(value) => {
            set_oscillator_wave_shape(
                &module_parameters.oscillators[OscillatorIndex::Sub as usize],
                value,
            );
        }
        CC::Oscillator1Shape(value) => {
            set_oscillator_wave_shape(
                &module_parameters.oscillators[OscillatorIndex::One as usize],
                value,
            );
        }
        CC::Oscillator2Shape(value) => {
            set_oscillator_wave_shape(
                &module_parameters.oscillators[OscillatorIndex::Two as usize],
                value,
            );
        }
        CC::Oscillator3Shape(value) => {
            set_oscillator_wave_shape(
                &module_parameters.oscillators[OscillatorIndex::Three as usize],
                value,
            );
        }
        CC::SubOscillatorCourseTune(value) => {
            set_oscillator_course_tune(
                &module_parameters.oscillators[OscillatorIndex::Sub as usize],
                value,
            );
        }
        CC::Oscillator1CourseTune(value) => {
            set_oscillator_course_tune(
                &module_parameters.oscillators[OscillatorIndex::One as usize],
                value,
            );
        }
        CC::Oscillator2CourseTune(value) => {
            set_oscillator_course_tune(
                &module_parameters.oscillators[OscillatorIndex::Two as usize],
                value,
            );
        }
        CC::Oscillator3CourseTune(value) => {
            set_oscillator_course_tune(
                &module_parameters.oscillators[OscillatorIndex::Three as usize],
                value,
            );
        }
        CC::SubOscillatorFineTune(value) => {
            set_oscillator_fine_tune(
                &module_parameters.oscillators[OscillatorIndex::Sub as usize],
                value,
            );
        }
        CC::Oscillator1FineTune(value) => {
            set_oscillator_fine_tune(
                &module_parameters.oscillators[OscillatorIndex::One as usize],
                value,
            );
        }
        CC::Oscillator2FineTune(value) => {
            set_oscillator_fine_tune(
                &module_parameters.oscillators[OscillatorIndex::Two as usize],
                value,
            );
        }
        CC::Oscillator3FineTune(value) => {
            set_oscillator_fine_tune(
                &module_parameters.oscillators[OscillatorIndex::Three as usize],
                value,
            );
        }
        CC::SubOscillatorLevel(value) => {
            set_oscillator_level(&module_parameters.mixer, OscillatorIndex::Sub, value);
        }
        CC::Oscillator1Level(value) => {
            set_oscillator_level(&module_parameters.mixer, OscillatorIndex::One, value);
        }
        CC::Oscillator2Level(value) => {
            set_oscillator_level(&module_parameters.mixer, OscillatorIndex::Two, value);
        }
        CC::Oscillator3Level(value) => {
            set_oscillator_level(&module_parameters.mixer, OscillatorIndex::Three, value);
        }
        CC::SubOscillatorMute(value) => {
            set_oscillator_mute(&module_parameters.mixer, OscillatorIndex::Sub, value);
        }
        CC::Oscillator1Mute(value) => {
            set_oscillator_mute(&module_parameters.mixer, OscillatorIndex::One, value);
        }
        CC::Oscillator2Mute(value) => {
            set_oscillator_mute(&module_parameters.mixer, OscillatorIndex::Two, value);
        }
        CC::Oscillator3Mute(value) => {
            set_oscillator_mute(&module_parameters.mixer, OscillatorIndex::Three, value);
        }
        CC::SubOscillatorBalance(value) => {
            set_oscillator_balance(&module_parameters.mixer, OscillatorIndex::Sub, value);
        }
        CC::Oscillator1Balance(value) => {
            set_oscillator_balance(&module_parameters.mixer, OscillatorIndex::One, value);
        }
        CC::Oscillator2Balance(value) => {
            set_oscillator_balance(&module_parameters.mixer, OscillatorIndex::Two, value);
        }
        CC::Oscillator3Balance(value) => {
            set_oscillator_balance(&module_parameters.mixer, OscillatorIndex::Three, value);
        }
        CC::PortamentoEnabled(value) => {
            set_portamento_enabled(&module_parameters.oscillators, value);
        }
        CC::FilterPoles(value) => {
            set_filter_poles(&module_parameters.filter, value);
        }
        CC::FilterResonance(value) => {
            set_filter_resonance(&module_parameters.filter, value);
        }
        CC::AmpEGReleaseTime(value) => {
            set_envelope_release_time(&module_parameters.amp_envelope, value);
        }
        CC::AmpEGAttackTime(value) => {
            set_envelope_attack_time(&module_parameters.amp_envelope, value);
        }
        CC::FilterCutoff(value) => {
            set_filter_cutoff(&module_parameters.filter, value);
        }
        CC::AmpEGDecayTime(value) => {
            set_envelope_decay_time(&module_parameters.amp_envelope, value);
        }
        CC::AmpEGSustainLevel(value) => {
            set_envelope_sustain_level(&module_parameters.amp_envelope, value);
        }
        CC::AmpEGInverted(value) => {
            set_envelope_inverted(&module_parameters.amp_envelope, value);
        }
        CC::FilterEGAttackTime(value) => {
            set_envelope_attack_time(&module_parameters.filter_envelope, value);
        }
        CC::FilterEGDecayTime(value) => {
            set_envelope_decay_time(&module_parameters.filter_envelope, value);
        }
        CC::FilterEGSustainLevel(value) => {
            set_envelope_sustain_level(&module_parameters.filter_envelope, value);
        }
        CC::FilterEGReleaseTime(value) => {
            set_envelope_release_time(&module_parameters.filter_envelope, value);
        }
        CC::FilterEGInverted(value) => {
            set_envelope_inverted(&module_parameters.filter_envelope, value);
        }
        CC::FilterEGAmount(value) => {
            set_envelope_amount(&module_parameters.filter_envelope, value);
        }
        CC::KeyTrackingAmount(value) => {
            set_key_tracking_amount(&module_parameters.filter, value);
        }
        CC::LFO1Frequency(value) => {
            set_lfo_frequency(&module_parameters.lfo1, value);
        }
        CC::LFO1CenterValue(value) => {
            set_lfo_center_value(&module_parameters.lfo1, value);
        }
        CC::LFO1Range(value) => {
            set_lfo_range(&module_parameters.lfo1, value);
        }
        CC::LFO1WaveShape(value) => {
            set_lfo_wave_shape(&module_parameters.lfo1, value);
        }
        CC::LFO1Phase(value) => {
            set_lfo_phase(&module_parameters.lfo1, value);
        }
        CC::LFO1Reset => {
            lfo_reset(&module_parameters.lfo1);
        }
        CC::FilterModLFOFrequency(value) => {
            set_lfo_frequency(&module_parameters.filter_lfo, value);
        }
        CC::FilterModLFOAmount(value) => {
            set_lfo_range(&module_parameters.filter_lfo, value);
        }
        CC::FilterModLFOWaveShape(value) => {
            set_lfo_wave_shape(&module_parameters.filter_lfo, value);
        }
        CC::FilterModLFOPhase(value) => {
            set_lfo_phase(&module_parameters.filter_lfo, value);
        }
        CC::FilterModLFOReset => {
            lfo_reset(&module_parameters.filter_lfo);
        }
        CC::AllNotesOff => {
            process_midi_note_off_message(module_parameters);
        }
    }
}

fn set_lfo_frequency(parameters: &LfoParameters, value: u8) {
    let frequency = midi_value_converters::exponential_curve_lfo_frequency_from_midi_value(value);
    store_f32_as_atomic_u32(&parameters.frequency, frequency);
}

fn set_lfo_center_value(parameters: &LfoParameters, value: u8) {
    let center_value = midi_value_converters::midi_value_to_f32_negative_1_to_1(value);
    store_f32_as_atomic_u32(&parameters.center_value, center_value);
}

fn set_lfo_range(parameters: &LfoParameters, value: u8) {
    let lfo_range = if value == 0 {
        0.0
    } else {
        midi_value_converters::midi_value_to_f32_0_to_1(value)
    };

    store_f32_as_atomic_u32(&parameters.range, lfo_range);
}

fn set_lfo_phase(parameters: &LfoParameters, value: u8) {
    let phase = midi_value_converters::midi_value_to_f32_0_to_1(value);
    store_f32_as_atomic_u32(&parameters.phase, phase);
}

fn set_lfo_wave_shape(parameters: &LfoParameters, value: u8) {
    let wave_shape_index = midi_value_converters::midi_value_to_wave_shape_index(value);
    parameters.wave_shape.store(wave_shape_index, Relaxed);
}

fn lfo_reset(parameters: &LfoParameters) {
    parameters.reset.store(true, Relaxed);
}

fn set_key_tracking_amount(filter_parameters: &FilterParameters, value: u8) {
    let amount = midi_value_converters::midi_value_to_f32_range(value, 0.0, 2.0);
    store_f32_as_atomic_u32(&filter_parameters.key_tracking_amount, amount);
}

fn set_envelope_amount(envelope_parameters: &EnvelopeParameters, value: u8) {
    let amount = midi_value_converters::midi_value_to_f32_0_to_1(value);
    store_f32_as_atomic_u32(&envelope_parameters.amount, amount);
}

fn set_envelope_release_time(envelope_parameters: &EnvelopeParameters, value: u8) {
    let milliseconds = midi_value_converters::midi_value_to_envelope_milliseconds(value);
    envelope_parameters.release_ms.store(milliseconds, Relaxed);
}

fn set_envelope_sustain_level(envelope_parameters: &EnvelopeParameters, value: u8) {
    let sustain_level =
        midi_value_converters::exponential_curve_level_adjustment_from_midi_value(value);
    store_f32_as_atomic_u32(&envelope_parameters.sustain_level, sustain_level);
}

fn set_envelope_decay_time(envelope_parameters: &EnvelopeParameters, value: u8) {
    let milliseconds = midi_value_converters::midi_value_to_envelope_milliseconds(value);
    envelope_parameters.decay_ms.store(milliseconds, Relaxed);
}

fn set_envelope_attack_time(envelope_parameters: &EnvelopeParameters, value: u8) {
    let milliseconds = midi_value_converters::midi_value_to_envelope_milliseconds(value);
    envelope_parameters.attack_ms.store(milliseconds, Relaxed);
}

fn set_envelope_inverted(envelope_parameters: &EnvelopeParameters, value: u8) {
    let is_inverted = midi_value_converters::midi_value_to_bool(value);
    envelope_parameters.is_inverted.store(is_inverted, Relaxed);
}

fn set_filter_resonance(filter_parameters: &FilterParameters, value: u8) {
    let resonance = midi_value_converters::midi_value_to_f32_range(
        value,
        MIN_FILTER_RESONANCE,
        MAX_FILTER_RESONANCE,
    );
    store_f32_as_atomic_u32(&filter_parameters.resonance, resonance);
}

fn set_filter_poles(filter_parameters: &FilterParameters, value: u8) {
    let filter_poles = midi_value_converters::midi_value_to_number_of_filter_poles(value);
    filter_parameters.filter_poles.swap(filter_poles, Relaxed);
}

fn set_filter_cutoff(filter_parameters: &FilterParameters, value: u8) {
    let cutoff_frequency =
        midi_value_converters::exponential_curve_filter_cutoff_from_midi_value(value);
    store_f32_as_atomic_u32(&filter_parameters.cutoff_frequency, cutoff_frequency);
}

fn set_output_balance(parameters: &MixerParameters, value: u8) {
    let output_balance = midi_value_converters::midi_value_to_f32_negative_1_to_1(value);
    store_f32_as_atomic_u32(&parameters.output_balance, output_balance);
}

fn set_output_volume(parameters: &MixerParameters, value: u8) {
    let output_level =
        midi_value_converters::exponential_curve_level_adjustment_from_midi_value(value);
    store_f32_as_atomic_u32(&parameters.output_level, output_level);
}

fn set_velocity_curve(current_note: &mut Arc<CurrentNote>, value: u8) {
    current_note.velocity_curve.store(value, Relaxed);
}

fn set_pitch_bend_range(parameters: &KeyboardParameters, value: u8) {
    let range = midi_value_converters::midi_value_to_u8_range(
        value,
        MIN_PITCH_BEND_RANGE,
        MAX_PITCH_BEND_RANGE,
    );
    parameters.pitch_bend_range.store(range, Relaxed);
}

fn set_mod_wheel(parameters: &KeyboardParameters, value: u8) {
    let mod_wheel_amount = midi_value_converters::midi_value_to_f32_0_to_1(value);
    store_f32_as_atomic_u32(&parameters.mod_wheel_amount, mod_wheel_amount);
}

fn set_oscillator_shape_parameter1(parameters: &OscillatorParameters, value: u8) {
    let shape_parameter1 = midi_value_converters::midi_value_to_f32_0_to_1(value);
    store_f32_as_atomic_u32(&parameters.shape_parameter1, shape_parameter1);
}

fn set_oscillator_shape_parameter2(parameters: &OscillatorParameters, value: u8) {
    let shape_parameter2 = midi_value_converters::midi_value_to_f32_0_to_1(value);
    store_f32_as_atomic_u32(&parameters.shape_parameter2, shape_parameter2);
}

fn set_oscillator_key_sync(parameters: &[OscillatorParameters; 4], value: u8) {
    for parameters in parameters {
        parameters
            .key_sync_enabled
            .store(midi_value_converters::midi_value_to_bool(value), Relaxed);
    }
}

fn set_oscillator_hard_sync(parameters: &[OscillatorParameters; 4], value: u8) {
    for parameters in parameters {
        parameters
            .hard_sync_enabled
            .store(midi_value_converters::midi_value_to_bool(value), Relaxed);
    }
}

fn set_portamento_time(parameters: &[OscillatorParameters; 4], value: u8) {
    let speed = midi_value_converters::midi_value_to_u16_range(
        value,
        MIN_PORTAMENTO_SPEED_IN_BUFFERS,
        MAX_PORTAMENTO_SPEED_IN_BUFFERS,
    );

    for parameters in parameters {
        parameters.portamento_speed.store(speed, Relaxed);
    }
}

fn set_portamento_enabled(parameters: &[OscillatorParameters; 4], value: u8) {
    for parameters in parameters {
        parameters
            .portamento_is_enabled
            .store(midi_value_converters::midi_value_to_bool(value), Relaxed);
    }
}

fn set_oscillator_balance(parameters: &MixerParameters, oscillator: OscillatorIndex, value: u8) {
    let balance = midi_value_converters::midi_value_to_f32_negative_1_to_1(value);
    store_f32_as_atomic_u32(
        &parameters.quad_mixer_inputs[oscillator as usize].balance,
        balance,
    );
}

fn set_oscillator_mute(parameters: &MixerParameters, oscillator: OscillatorIndex, value: u8) {
    let mute = midi_value_converters::midi_value_to_bool(value);
    parameters.quad_mixer_inputs[oscillator as usize]
        .mute
        .swap(mute, Relaxed);
}

fn set_oscillator_level(parameters: &MixerParameters, oscillator: OscillatorIndex, value: u8) {
    let level = midi_value_converters::exponential_curve_level_adjustment_from_midi_value(value);
    store_f32_as_atomic_u32(
        &parameters.quad_mixer_inputs[oscillator as usize].level,
        level,
    );
}

fn set_oscillator_fine_tune(parameters: &OscillatorParameters, value: u8) {
    let cents = midi_value_converters::midi_value_to_i8_range(
        value,
        OSCILLATOR_FINE_TUNE_MIN_CENTS,
        OSCILLATOR_FINE_TUNE_MAX_CENTS,
    );

    parameters.fine_tune.store(cents, Relaxed);
}

fn set_oscillator_course_tune(parameters: &OscillatorParameters, value: u8) {
    let interval = midi_value_converters::midi_value_to_i8_range(
        value,
        OSCILLATOR_COURSE_TUNE_MIN_INTERVAL,
        OSCILLATOR_COURSE_TUNE_MAX_INTERVAL,
    );

    parameters.course_tune.store(interval, Relaxed);
}

fn set_oscillator_wave_shape(parameters: &OscillatorParameters, value: u8) {
    let wave_shape_index = midi_value_converters::midi_value_to_wave_shape_index(value);
    parameters.wave_shape_index.store(wave_shape_index, Relaxed);
}
