use crate::midi::CC;
use crate::modules::envelope::{ENVELOPE_MAX_MILLISECONDS, ENVELOPE_MIN_MILLISECONDS};
use crate::modules::filter::FilterSlope;
use crate::modules::mixer::MixerInput;
use crate::modules::oscillator::WaveShape;
use crate::synthesizer::constants::*;
use crate::synthesizer::tuner::tune;
use crate::synthesizer::{
    EnvelopeIndex, LfoIndex, MidiNoteEvent, Modules, OscillatorIndex, Parameters,
};
use std::sync::{Arc, Mutex, MutexGuard};

pub fn action_midi_note_events(
    midi_events: MidiNoteEvent,
    modules: &mut MutexGuard<Modules>,
    oscillator_key_sync_enabled: bool,
) {
    match midi_events {
        MidiNoteEvent::NoteOn => {
            for envelope in modules.envelopes.iter_mut() {
                envelope.gate_on();
            }
            if oscillator_key_sync_enabled {
                for oscillator in modules.oscillators.iter_mut() {
                    oscillator.reset()
                }
            }
        }
        MidiNoteEvent::NoteOff => {
            for envelope in modules.envelopes.iter_mut() {
                envelope.gate_off();
            }
        }
    }
}

pub fn process_midi_channel_pressure_message(
    parameters_arc: &mut Arc<Mutex<Parameters>>,
    pressure_value: u8,
) {
    let mut parameters = parameters_arc
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    parameters.aftertouch_amount = midi_value_to_f32_0_to_1(pressure_value);
}

pub fn process_midi_pitch_bend_message(
    parameters_arc: &mut Arc<Mutex<Parameters>>,
    bend_amount: i16,
) {
    let mut parameters = parameters_arc
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());

    update_current_note_from_midi_pitch_bend(bend_amount, &mut parameters);
    update_current_note_from_stored_parameters(&mut parameters);
}

pub fn process_midi_note_off_message(midi_event_arc: &mut Arc<Mutex<Option<MidiNoteEvent>>>) {
    let mut midi_events = midi_event_arc
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());

    *midi_events = Some(MidiNoteEvent::NoteOff);
}

pub fn process_midi_note_on_message(
    parameters_arc: &mut Arc<Mutex<Parameters>>,
    midi_event_arc: &mut Arc<Mutex<Option<MidiNoteEvent>>>,
    midi_note: u8,
    velocity: u8,
) {
    let mut parameters = parameters_arc
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());

    parameters.current_note.velocity = continuously_variable_curve_mapping_from_midi_value(
        parameters.current_note.velocity_curve,
        velocity,
    );

    parameters.current_note.midi_note = midi_note;
    update_current_note_from_stored_parameters(&mut parameters);

    let mut midi_events = midi_event_arc
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());

    *midi_events = Some(MidiNoteEvent::NoteOn);
}

pub fn process_midi_cc_values(
    cc_value: CC,
    parameters_arc: &mut Arc<Mutex<Parameters>>,
    midi_event_arc: &mut Arc<Mutex<Option<MidiNoteEvent>>>,
    modules_arc: &mut Arc<Mutex<Modules>>,
) {
    log::debug!("process_midi_cc_values(): CC received: {:?}", cc_value);
    match cc_value {
        CC::ModWheel(value) => {
            set_mod_wheel(parameters_arc, value);
        }
        CC::VelocityCurve(value) => {
            set_velocity_curve(parameters_arc, value);
        }
        CC::Volume(value) => {
            set_output_volume(modules_arc, value);
        }
        CC::ConstantLevel(value) => {
            set_output_constant_level(modules_arc, value);
        }
        CC::Pan(value) => {
            set_output_balance(modules_arc, value);
        }
        CC::SubOscillatorShapeParameter1(value) => {
            set_oscillator_shape_parameter1(modules_arc, OscillatorIndex::Sub, value);
        }
        CC::SubOscillatorShapeParameter2(value) => {
            set_oscillator_shape_parameter2(modules_arc, OscillatorIndex::Sub, value);
        }
        CC::Oscillator1ShapeParameter1(value) => {
            set_oscillator_shape_parameter1(modules_arc, OscillatorIndex::One, value);
        }
        CC::Oscillator1ShapeParameter2(value) => {
            set_oscillator_shape_parameter2(modules_arc, OscillatorIndex::One, value);
        }
        CC::Oscillator2ShapeParameter1(value) => {
            set_oscillator_shape_parameter1(modules_arc, OscillatorIndex::Two, value);
        }
        CC::Oscillator2ShapeParameter2(value) => {
            set_oscillator_shape_parameter2(modules_arc, OscillatorIndex::Two, value);
        }
        CC::Oscillator3ShapeParameter1(value) => {
            set_oscillator_shape_parameter1(modules_arc, OscillatorIndex::Three, value);
        }
        CC::Oscillator3ShapeParameter2(value) => {
            set_oscillator_shape_parameter2(modules_arc, OscillatorIndex::Three, value);
        }
        CC::OscillatorKeySyncEnabled(value) => {
            set_oscillator_key_sync(parameters_arc, value);
        }
        CC::SubOscillatorShape(value) => {
            set_oscillator_wave_shape(modules_arc, OscillatorIndex::Sub, value);
        }
        CC::Oscillator1Shape(value) => {
            set_oscillator_wave_shape(modules_arc, OscillatorIndex::One, value);
        }
        CC::Oscillator2Shape(value) => {
            set_oscillator_wave_shape(modules_arc, OscillatorIndex::Two, value);
        }
        CC::Oscillator3Shape(value) => {
            set_oscillator_wave_shape(modules_arc, OscillatorIndex::Three, value);
        }
        CC::SubOscillatorCourseTune(value) => {
            set_oscillator_course_tune(parameters_arc, OscillatorIndex::Sub, value);
        }
        CC::Oscillator1CourseTune(value) => {
            set_oscillator_course_tune(parameters_arc, OscillatorIndex::One, value);
        }
        CC::Oscillator2CourseTune(value) => {
            set_oscillator_course_tune(parameters_arc, OscillatorIndex::Two, value);
        }
        CC::Oscillator3CourseTune(value) => {
            set_oscillator_course_tune(parameters_arc, OscillatorIndex::Three, value);
        }
        CC::SubOscillatorFineTune(value) => {
            set_oscillator_fine_tune(parameters_arc, OscillatorIndex::Sub, value);
        }
        CC::Oscillator1FineTune(value) => {
            set_oscillator_fine_tune(parameters_arc, OscillatorIndex::One, value);
        }
        CC::Oscillator2FineTune(value) => {
            set_oscillator_fine_tune(parameters_arc, OscillatorIndex::Two, value);
        }
        CC::Oscillator3FineTune(value) => {
            set_oscillator_fine_tune(parameters_arc, OscillatorIndex::Three, value);
        }
        CC::SubOscillatorLevel(value) => {
            set_oscillator_level(modules_arc, OscillatorIndex::Sub, value);
        }
        CC::Oscillator1Level(value) => {
            set_oscillator_level(modules_arc, OscillatorIndex::One, value);
        }
        CC::Oscillator2Level(value) => {
            set_oscillator_level(modules_arc, OscillatorIndex::Two, value);
        }
        CC::Oscillator3Level(value) => {
            set_oscillator_level(modules_arc, OscillatorIndex::Three, value);
        }
        CC::SubOscillatorMute(value) => {
            set_oscillator_mute(modules_arc, OscillatorIndex::Sub, value);
        }
        CC::Oscillator1Mute(value) => {
            set_oscillator_mute(modules_arc, OscillatorIndex::One, value);
        }
        CC::Oscillator2Mute(value) => {
            set_oscillator_mute(modules_arc, OscillatorIndex::Two, value);
        }
        CC::Oscillator3Mute(value) => {
            set_oscillator_mute(modules_arc, OscillatorIndex::Three, value);
        }
        CC::SubOscillatorPan(value) => {
            set_oscillator_balance(modules_arc, OscillatorIndex::Sub, value);
        }
        CC::Oscillator1Pan(value) => {
            set_oscillator_balance(modules_arc, OscillatorIndex::One, value);
        }
        CC::Oscillator2Pan(value) => {
            set_oscillator_balance(modules_arc, OscillatorIndex::Two, value);
        }
        CC::Oscillator3Pan(value) => {
            set_oscillator_balance(modules_arc, OscillatorIndex::Three, value);
        }
        CC::FilterPoles(value) => {
            set_filter_poles(modules_arc, value);
        }
        CC::FilterResonance(value) => {
            set_filter_resonance(modules_arc, value);
        }
        CC::AmpEGReleaseTime(value) => {
            set_amp_eg_release_time(modules_arc, EnvelopeIndex::Amplifier, value);
        }
        CC::AmpEGAttackTime(value) => {
            set_amp_eg_attack_time(modules_arc, EnvelopeIndex::Amplifier, value);
        }
        CC::FilterCutoff(value) => {
            set_filter_cutoff(modules_arc, value);
        }
        CC::AmpEGDecayTime(value) => {
            set_amp_eg_decay_time(modules_arc, EnvelopeIndex::Amplifier, value);
        }
        CC::AmpEGSustainLevel(value) => {
            set_amp_eq_sustain_level(modules_arc, EnvelopeIndex::Amplifier, value);
        }
        CC::AmpEGInverted(value) => {
            set_amp_eg_inverted(modules_arc, EnvelopeIndex::Amplifier, value);
        }
        CC::FilterEGAttackTime(value) => {
            set_filter_eg_attack_time(modules_arc, EnvelopeIndex::Filter, value);
        }
        CC::FilterEGDecayTime(value) => {
            set_filter_eq_decay_time(modules_arc, EnvelopeIndex::Filter, value);
        }
        CC::FilterEGSustainLevel(value) => {
            set_filter_eq_sustain_level(modules_arc, EnvelopeIndex::Filter, value);
        }
        CC::FilterEGReleaseTime(value) => {
            set_filter_eq_release_time(modules_arc, EnvelopeIndex::Filter, value);
        }
        CC::FilterEGInverted(value) => {
            set_filter_eq_inverted(modules_arc, EnvelopeIndex::Filter, value);
        }
        CC::FilterEGAmount(value) => {
            set_filter_eg_amount(modules_arc, EnvelopeIndex::Filter, value);
        }
        CC::LFO1Frequency(value) => {
            set_lfo_frequency(modules_arc, LfoIndex::Lfo1, value);
        }
        CC::LFO1CenterValue(value) => {
            set_lfo_center_value(modules_arc, LfoIndex::Lfo1, value);
        }
        CC::LFO1Range(value) => {
            set_lfo_range(modules_arc, LfoIndex::Lfo1, value);
        }
        CC::LFO1WaveShape(value) => {
            set_lfo_wave_shape(modules_arc, LfoIndex::Lfo1, value);
        }
        CC::LFO1Phase(value) => {
            set_lfo_phase(modules_arc, LfoIndex::Lfo1, value);
        }
        CC::LFO1Reset => {
            lfo_reset(modules_arc, LfoIndex::Lfo1);
        }
        CC::FilterModLFOFrequency(value) => {
            set_lfo_frequency(modules_arc, LfoIndex::Filter, value);
        }
        CC::FilterModLFOAmount(value) => {
            set_lfo_range(modules_arc, LfoIndex::Filter, value);
        }
        CC::FilterModLFOWaveShape(value) => {
            set_lfo_wave_shape(modules_arc, LfoIndex::Filter, value);
        }
        CC::FilterModLFOPhase(value) => {
            set_lfo_phase(modules_arc, LfoIndex::Filter, value);
        }
        CC::FilterModLFOReset => {
            lfo_reset(modules_arc, LfoIndex::Filter);
        }
        CC::AllNotesOff => {
            set_all_note_off(midi_event_arc);
        }
    }
}

fn set_all_note_off(midi_event_arc: &mut Arc<Mutex<Option<MidiNoteEvent>>>) {
    let mut midi_events = midi_event_arc
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());

    *midi_events = Some(MidiNoteEvent::NoteOff);
}

fn set_lfo_frequency(modules_arc: &mut Arc<Mutex<Modules>>, lfo: LfoIndex, value: u8) {
    let mut modules = modules_arc
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());

    modules.lfos[lfo as usize]
        .set_frequency(exponential_curve_lfo_frequency_from_midi_value(value));
}

fn set_lfo_center_value(modules_arc: &mut Arc<Mutex<Modules>>, lfo: LfoIndex, value: u8) {
    let mut modules = modules_arc
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());

    modules.lfos[lfo as usize].set_center_value(midi_value_to_f32_negative_1_to_1(value));
}

fn set_lfo_range(modules_arc: &mut Arc<Mutex<Modules>>, lfo: LfoIndex, value: u8) {
    let lfo_range = if value == 0 {
        0.0
    } else {
        midi_value_to_f32_0_to_1(value)
    };

    let mut modules = modules_arc
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());

    modules.lfos[lfo as usize].set_range(lfo_range);
}

fn set_lfo_wave_shape(modules_arc: &mut Arc<Mutex<Modules>>, lfo: LfoIndex, value: u8) {
    let mut modules = modules_arc
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());

    modules.lfos[lfo as usize].set_wave_shape(midi_value_to_oscillator_wave_shape(value));
}

fn set_lfo_phase(modules_arc: &mut Arc<Mutex<Modules>>, lfo: LfoIndex, value: u8) {
    let mut modules = modules_arc
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());

    modules.lfos[lfo as usize].set_phase(midi_value_to_f32_0_to_1(value));
}

fn lfo_reset(modules_arc: &mut Arc<Mutex<Modules>>, lfo: LfoIndex) {
    let mut modules = modules_arc
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());

    modules.lfos[lfo as usize].reset()
}

fn set_filter_eg_amount(modules_arc: &mut Arc<Mutex<Modules>>, envelope: EnvelopeIndex, value: u8) {
    let mut modules = modules_arc
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    modules.envelopes[envelope as usize].set_amount(midi_value_to_f32_0_to_1(value));
}

fn set_filter_eq_inverted(
    modules_arc: &mut Arc<Mutex<Modules>>,
    envelope: EnvelopeIndex,
    value: u8,
) {
    let mut modules = modules_arc
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    modules.envelopes[envelope as usize].set_is_inverted(midi_value_to_bool(value));
}

fn set_filter_eq_release_time(
    modules_arc: &mut Arc<Mutex<Modules>>,
    envelope: EnvelopeIndex,
    value: u8,
) {
    let time = midi_value_to_f32_range(value, ENVELOPE_MIN_MILLISECONDS, ENVELOPE_MAX_MILLISECONDS);

    let mut modules = modules_arc
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());

    modules.envelopes[envelope as usize].set_release_milliseconds(time);
}

fn set_filter_eq_sustain_level(
    modules_arc: &mut Arc<Mutex<Modules>>,
    envelope: EnvelopeIndex,
    value: u8,
) {
    let mut modules = modules_arc
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());

    modules.envelopes[envelope as usize].set_sustain_level(midi_value_to_f32_0_to_1(value));
}

fn set_filter_eq_decay_time(
    modules_arc: &mut Arc<Mutex<Modules>>,
    envelope: EnvelopeIndex,
    value: u8,
) {
    let time = midi_value_to_f32_range(value, ENVELOPE_MIN_MILLISECONDS, ENVELOPE_MAX_MILLISECONDS);

    let mut modules = modules_arc
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());

    modules.envelopes[envelope as usize].set_decay_milliseconds(time);
}

fn set_filter_eg_attack_time(
    modules_arc: &mut Arc<Mutex<Modules>>,
    envelope: EnvelopeIndex,
    value: u8,
) {
    let time = midi_value_to_f32_range(value, ENVELOPE_MIN_MILLISECONDS, ENVELOPE_MAX_MILLISECONDS);

    let mut modules = modules_arc
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());

    modules.envelopes[envelope as usize].set_attack_milliseconds(time);
}

fn set_amp_eg_inverted(modules_arc: &mut Arc<Mutex<Modules>>, envelope: EnvelopeIndex, value: u8) {
    let mut modules = modules_arc
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    modules.envelopes[envelope as usize].set_is_inverted(midi_value_to_bool(value));
}

fn set_amp_eq_sustain_level(
    modules_arc: &mut Arc<Mutex<Modules>>,
    envelope: EnvelopeIndex,
    value: u8,
) {
    let mut modules = modules_arc
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());

    modules.envelopes[envelope as usize]
        .set_sustain_level(exponential_curve_level_adjustment_from_midi_value(value));
}

fn set_amp_eg_decay_time(
    modules_arc: &mut Arc<Mutex<Modules>>,
    envelope: EnvelopeIndex,
    value: u8,
) {
    let time = midi_value_to_f32_range(value, ENVELOPE_MIN_MILLISECONDS, ENVELOPE_MAX_MILLISECONDS);

    let mut modules = modules_arc
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());

    modules.envelopes[envelope as usize].set_decay_milliseconds(time);
}

fn set_filter_cutoff(modules_arc: &mut Arc<Mutex<Modules>>, value: u8) {
    let cutoff_frequency = exponential_curve_filter_cutoff_from_midi_value(value);

    let mut modules = modules_arc
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());

    modules.filter.set_cutoff_frequency(cutoff_frequency);
}

fn set_amp_eg_attack_time(
    modules_arc: &mut Arc<Mutex<Modules>>,
    envelope: EnvelopeIndex,
    value: u8,
) {
    let time = midi_value_to_f32_range(value, ENVELOPE_MIN_MILLISECONDS, ENVELOPE_MAX_MILLISECONDS);

    let mut modules = modules_arc
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());

    modules.envelopes[envelope as usize].set_attack_milliseconds(time);
}

fn set_amp_eg_release_time(
    modules_arc: &mut Arc<Mutex<Modules>>,
    envelope: EnvelopeIndex,
    value: u8,
) {
    let time = midi_value_to_f32_range(value, ENVELOPE_MIN_MILLISECONDS, ENVELOPE_MAX_MILLISECONDS);
    let mut modules = modules_arc
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());

    modules.envelopes[envelope as usize].set_release_milliseconds(time);
}

fn set_filter_resonance(modules_arc: &mut Arc<Mutex<Modules>>, value: u8) {
    let mut modules = modules_arc
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());

    modules
        .filter
        .set_resonance(midi_value_to_f32_range(value, 0.0, 1.0));
}

fn set_filter_poles(modules_arc: &mut Arc<Mutex<Modules>>, value: u8) {
    let mut modules = modules_arc
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());

    modules
        .filter
        .set_filter_slope(midi_value_to_filter_slope(value));
}

fn set_output_balance(modules_arc: &mut Arc<Mutex<Modules>>, value: u8) {
    let output_balance = midi_value_to_f32_negative_1_to_1(value);

    let mut modules = modules_arc
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());

    modules.mixer.set_output_balance(output_balance);
}

fn set_output_constant_level(modules_arc: &mut Arc<Mutex<Modules>>, value: u8) {
    let is_constant_level = midi_value_to_bool(value);
    let mut modules = modules_arc
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());

    modules.mixer.set_constant_level(is_constant_level)
}

fn set_output_volume(modules_arc: &mut Arc<Mutex<Modules>>, value: u8) {
    let output_level = exponential_curve_level_adjustment_from_midi_value(value);

    let mut modules = modules_arc
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());

    modules.mixer.set_output_level(output_level);
}

fn set_velocity_curve(parameters_arc: &mut Arc<Mutex<Parameters>>, value: u8) {
    let mut parameters = parameters_arc
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());

    parameters.current_note.velocity_curve = value;
}

fn set_mod_wheel(parameters_arc: &mut Arc<Mutex<Parameters>>, value: u8) {
    let mod_wheel_amount = midi_value_to_f32_0_to_1(value);
    let mut parameters = parameters_arc
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());

    parameters.mod_wheel_amount = mod_wheel_amount;
}

fn set_oscillator_shape_parameter1(
    modules_arc: &mut Arc<Mutex<Modules>>,
    oscillator: OscillatorIndex,
    value: u8,
) {
    let mut modules = modules_arc
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());

    modules.oscillators[oscillator as usize].set_shape_parameter1(midi_value_to_f32_0_to_1(value));
}

fn set_oscillator_shape_parameter2(
    modules_arc: &mut Arc<Mutex<Modules>>,
    oscillator: OscillatorIndex,
    value: u8,
) {
    let mut modules = modules_arc
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());

    modules.oscillators[oscillator as usize].set_shape_parameter2(midi_value_to_f32_0_to_1(value));
}

fn set_oscillator_key_sync(parameters_arc: &mut Arc<Mutex<Parameters>>, value: u8) {
    let mut parameters = parameters_arc
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());

    parameters.oscillator_key_sync_enabled = midi_value_to_bool(value);
}

fn set_oscillator_balance(
    modules_arc: &mut Arc<Mutex<Modules>>,
    oscillator: OscillatorIndex,
    value: u8,
) {
    let balance = midi_value_to_f32_negative_1_to_1(value);

    let mut modules = modules_arc
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());

    modules
        .mixer
        .set_quad_balance(mixer_input_from_oscillator_and_value(oscillator, balance));
}

fn set_oscillator_mute(
    modules_arc: &mut Arc<Mutex<Modules>>,
    oscillator: OscillatorIndex,
    value: u8,
) {
    let mute = midi_value_to_bool(value);

    let mut modules = modules_arc
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());

    modules
        .mixer
        .set_quad_mute(mixer_input_from_oscillator_and_value(oscillator, mute));
}

fn set_oscillator_level(
    modules_arc: &mut Arc<Mutex<Modules>>,
    oscillator: OscillatorIndex,
    value: u8,
) {
    let level = exponential_curve_level_adjustment_from_midi_value(value);

    let mut modules = modules_arc
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());

    modules
        .mixer
        .set_quad_level(mixer_input_from_oscillator_and_value(oscillator, level));
}

fn set_oscillator_fine_tune(
    parameters_arc: &mut Arc<Mutex<Parameters>>,
    oscillator: OscillatorIndex,
    value: u8,
) {
    let mut parameters = parameters_arc
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());

    parameters.oscillators[oscillator as usize].fine_tune =
        Some(midi_value_to_fine_tune_cents(value));
    update_current_note_from_stored_parameters(&mut parameters);
}

fn set_oscillator_course_tune(
    parameters_arc: &mut Arc<Mutex<Parameters>>,
    oscillator: OscillatorIndex,
    value: u8,
) {
    let mut parameters = parameters_arc
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());

    parameters.oscillators[oscillator as usize].course_tune =
        Some(midi_value_to_course_tune_intervals(value));
    update_current_note_from_stored_parameters(&mut parameters);
}

fn set_oscillator_wave_shape(
    modules_arc: &mut Arc<Mutex<Modules>>,
    oscillator: OscillatorIndex,
    value: u8,
) {
    let wave_shape = midi_value_to_oscillator_wave_shape(value);

    let mut modules = modules_arc
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());

    modules.oscillators[oscillator as usize].set_wave_shape(wave_shape);
}

pub fn midi_value_to_f32_range(midi_value: u8, minimum: f32, maximum: f32) -> f32 {
    let range = maximum - minimum;
    let increment = range / MAX_MIDI_VALUE as f32;
    minimum + (midi_value as f32 * increment)
}

fn midi_value_to_f32_0_to_1(midi_value: u8) -> f32 {
    midi_value_to_f32_range(midi_value, 0.0, 1.0)
}

fn midi_value_to_f32_negative_1_to_1(midi_value: u8) -> f32 {
    midi_value_to_f32_range(midi_value, -1.0, 1.0)
}

fn midi_value_to_bool(midi_value: u8) -> bool {
    midi_value > MIDI_SWITCH_MAX_OFF_VALUE
}

fn midi_value_to_fine_tune_cents(midi_value: u8) -> i16 {
    midi_value_to_f32_range(
        midi_value,
        OSCILLATOR_FINE_TUNE_MIN_CENTS as f32,
        OSCILLATOR_FINE_TUNE_MAX_CENTS as f32,
    ) as i16
}

fn midi_value_to_course_tune_intervals(midi_value: u8) -> i8 {
    midi_value_to_f32_range(
        midi_value,
        OSCILLATOR_COURSE_TUNE_MIN_INTERVAL as f32,
        OSCILLATOR_COURSE_TUNE_MAX_INTERVAL as f32,
    ) as i8
}

fn midi_value_to_filter_slope(midi_value: u8) -> FilterSlope {
    if midi_value < 32 {
        FilterSlope::Db6
    } else if midi_value < 64 {
        FilterSlope::Db12
    } else if midi_value < 96 {
        FilterSlope::Db18
    } else {
        FilterSlope::Db24
    }
}

fn midi_value_to_oscillator_wave_shape(midi_value: u8) -> WaveShape {
    if midi_value < 13 {
        WaveShape::Sine
    } else if midi_value < 26 {
        WaveShape::Triangle
    } else if midi_value < 39 {
        WaveShape::Square
    } else if midi_value < 52 {
        WaveShape::Saw
    } else if midi_value < 65 {
        WaveShape::Pulse
    } else if midi_value < 78 {
        WaveShape::Ramp
    } else if midi_value < 91 {
        WaveShape::SuperSaw
    } else if midi_value < 104 {
        WaveShape::AM
    } else if midi_value < 117 {
        WaveShape::FM
    } else {
        WaveShape::Noise
    }
}

fn mixer_input_from_oscillator_and_value<T>(
    oscillator: OscillatorIndex,
    value: T,
) -> MixerInput<T> {
    match oscillator {
        OscillatorIndex::Sub => MixerInput::One(value),
        OscillatorIndex::One => MixerInput::Two(value),
        OscillatorIndex::Two => MixerInput::Three(value),
        OscillatorIndex::Three => MixerInput::Four(value),
    }
}

fn exponential_curve_filter_cutoff_from_midi_value(midi_value: u8) -> f32 {
    if midi_value == 0 {
        return 0.0;
    }
    exponential_curve_from_midi_value_and_coefficient(midi_value, EXPONENTIAL_FILTER_COEFFICIENT)
}

fn exponential_curve_lfo_frequency_from_midi_value(midi_value: u8) -> f32 {
    if midi_value == 0 {
        return 0.0;
    }
    exponential_curve_from_midi_value_and_coefficient(midi_value, EXPONENTIAL_LFO_COEFFICIENT)
        / 100.0
}

fn exponential_curve_level_adjustment_from_midi_value(midi_value: u8) -> f32 {
    if midi_value == 0 {
        return 0.0;
    }
    exponential_curve_from_midi_value_and_coefficient(midi_value, EXPONENTIAL_LEVEL_COEFFICIENT)
        / LEVEL_CURVE_LINEAR_RANGE
}

fn exponential_curve_from_midi_value_and_coefficient(
    midi_value: u8,
    exponential_coefficient: f32,
) -> f32 {
    // exponential_coefficient is the log of the magnitude for the linear range you want to map to exponential range
    // If the range max is 1000x then min, then the exponential_coefficient is log(1000) = 6.908
    (exponential_coefficient * (midi_value as f32 / MAX_MIDI_VALUE as f32)).exp()
}

fn continuously_variable_curve_mapping_from_midi_value(
    mut slope_midi_value: u8,
    input_midi_value: u8,
) -> f32 {
    if slope_midi_value == 0 {
        slope_midi_value = 1;
    }

    let curve_exponent = if slope_midi_value == 64 {
        1.0
    } else if slope_midi_value > 64 {
        ((slope_midi_value - 64) as f32 / 63f32) * 7.0
    } else {
        slope_midi_value as f32 / 64f32
    };

    (input_midi_value as f32).powf(curve_exponent) / (MAX_MIDI_VALUE as f32).powf(curve_exponent)
}

fn update_current_note_from_midi_pitch_bend(
    pitch_bend_amount: i16,
    parameters: &mut MutexGuard<Parameters>,
) {
    for oscillator in parameters.oscillators.iter_mut() {
        if pitch_bend_amount == PITCH_BEND_AMOUNT_ZERO_POINT {
            oscillator.pitch_bend = None;
        } else if pitch_bend_amount == PITCH_BEND_AMOUNT_MAX_VALUE {
            oscillator.pitch_bend = Some(PITCH_BEND_AMOUNT_CENTS);
        } else {
            let pitch_bend_in_cents = (pitch_bend_amount - PITCH_BEND_AMOUNT_ZERO_POINT) as f32
                / PITCH_BEND_AMOUNT_ZERO_POINT as f32
                * PITCH_BEND_AMOUNT_CENTS as f32;
            oscillator.pitch_bend = Some(pitch_bend_in_cents as i16);
        }
    }
}

fn update_current_note_from_stored_parameters(parameters: &mut MutexGuard<Parameters>) {
    let sub_osc_frequency = tune(
        parameters.current_note.midi_note,
        &mut parameters.oscillators[0],
    );

    let osc1_frequency = tune(
        parameters.current_note.midi_note,
        &mut parameters.oscillators[1],
    );
    let osc2_frequency = tune(
        parameters.current_note.midi_note,
        &mut parameters.oscillators[2],
    );
    let osc3_frequency = tune(
        parameters.current_note.midi_note,
        &mut parameters.oscillators[3],
    );

    parameters.oscillators[0].frequency = sub_osc_frequency;
    parameters.oscillators[1].frequency = osc1_frequency;
    parameters.oscillators[2].frequency = osc2_frequency;
    parameters.oscillators[3].frequency = osc3_frequency;
}

#[cfg(test)]
mod tests {

    use super::*;

    fn f32_value_equality(value_1: f32, value_2: f32) -> bool {
        (value_1 - value_2).abs() <= f32::EPSILON
    }

    #[test]
    fn midi_value_to_f32_range_correctly_maps_edge_values() {
        assert!(f32_value_equality(
            midi_value_to_f32_range(0, 0.0, 1.0),
            0.0
        ));
        assert!(f32_value_equality(
            midi_value_to_f32_range(127, 0.0, 1.0),
            1.0
        ));
        assert!(f32_value_equality(
            midi_value_to_f32_range(0, -1.0, 1.0),
            -1.0
        ));
        assert!(f32_value_equality(
            midi_value_to_f32_range(127, -1.0, 1.0),
            1.0
        ));
    }

    #[test]
    fn midi_value_to_f32_range_correctly_maps_middle_values() {
        let middle_value1 = midi_value_to_f32_range(64, 0.0, 1.0);
        assert!(middle_value1 > 0.5 && middle_value1 < 0.51);

        let middle_value2 = midi_value_to_f32_range(64, -1.0, 1.0);
        assert!(middle_value2 > 0.0 && middle_value2 < 0.01);

        let middle_value3 = midi_value_to_f32_range(64, 20.0, 800.0);
        assert!(middle_value3 > 410.0 && middle_value3 < 415.0);

        let middle_value4 = midi_value_to_f32_range(64, -800.0, 20.0);
        assert!(middle_value4 < -386.0 && middle_value1 > -387.0);
    }

    #[test]
    fn midi_value_to_f32_0_to_1_correctly_maps_values() {
        assert!(f32_value_equality(midi_value_to_f32_0_to_1(0), 0.0));
        assert!(f32_value_equality(midi_value_to_f32_0_to_1(127), 1.0));
    }

    #[test]
    fn midi_value_to_f32_negative_1_to_1_correctly_maps_values() {
        assert!(f32_value_equality(
            midi_value_to_f32_negative_1_to_1(0),
            -1.0
        ));

        assert!(f32_value_equality(
            midi_value_to_f32_negative_1_to_1(12),
            -0.8110236
        ));
        assert!(f32_value_equality(
            midi_value_to_f32_negative_1_to_1(127),
            1.0
        ));
    }

    #[test]
    fn midi_value_to_bool_correctly_converts_threshold() {
        assert!(!midi_value_to_bool(0));
        assert!(!midi_value_to_bool(63));
        assert!(midi_value_to_bool(64));
        assert!(midi_value_to_bool(127));
    }
}
