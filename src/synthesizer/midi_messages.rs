use crate::midi::CC;
use crate::modules::envelope::{ENVELOPE_MAX_MILLISECONDS, ENVELOPE_MIN_MILLISECONDS, Envelope};
use crate::modules::filter::{Filter, FilterSlope};
use crate::modules::lfo::Lfo;
use crate::modules::oscillator::{Oscillator, WaveShape};
use crate::synthesizer::constants::*;
use crate::synthesizer::{
    CurrentNote, EnvelopeIndex, LfoIndex, MidiNoteEvent, Modules, OscillatorIndex, Parameters,
};
use std::sync::atomic::{AtomicU32, Ordering::Relaxed};
use std::sync::{Arc, Mutex, MutexGuard};

pub fn action_midi_note_events(
    midi_events: MidiNoteEvent,
    oscillators: &mut MutexGuard<[Oscillator; 4]>,
    envelopes: &mut MutexGuard<[Envelope; 2]>,
    oscillator_key_sync_enabled: bool,
) {
    match midi_events {
        MidiNoteEvent::NoteOn => {
            for envelope in envelopes.iter_mut() {
                envelope.gate_on();
            }
            if oscillator_key_sync_enabled {
                for oscillator in oscillators.iter_mut() {
                    oscillator.reset()
                }
            }
        }
        MidiNoteEvent::NoteOff => {
            for envelope in envelopes.iter_mut() {
                envelope.gate_off();
            }
        }
    }
}

pub fn process_midi_channel_pressure_message(parameters: &mut Arc<Parameters>, pressure_value: u8) {
    let aftertouch_amount = midi_value_to_f32_0_to_1(pressure_value);
    store_f32_as_atomic_u32(&parameters.aftertouch_amount, aftertouch_amount);
}

pub fn process_midi_pitch_bend_message(
    oscillators_arc: &mut Arc<Mutex<[Oscillator; 4]>>,
    current_note: &mut Arc<CurrentNote>,
    bend_amount: i16,
) {
    let mut oscillators = oscillators_arc
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());

    let midi_note = current_note.midi_note.load(Relaxed);

    update_current_note_from_midi_pitch_bend(bend_amount, &mut oscillators);
    update_current_note_from_stored_parameters(midi_note, &mut oscillators);
}

pub fn process_midi_note_off_message(
    oscillators_arc: &mut Arc<Mutex<[Oscillator; 4]>>,
    envelopes_arc: &mut Arc<Mutex<[Envelope; 2]>>,
) {
    let mut oscillators = oscillators_arc
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());

    let mut envelopes = envelopes_arc
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());

    action_midi_note_events(
        MidiNoteEvent::NoteOn,
        &mut oscillators,
        &mut envelopes,
        false,
    );
}

pub fn process_midi_note_on_message(
    oscillators_arc: &mut Arc<Mutex<[Oscillator; 4]>>,
    envelopes_arc: &mut Arc<Mutex<[Envelope; 2]>>,
    current_note: &mut Arc<CurrentNote>,
    midi_note: u8,
    velocity: u8,
) {
    let scaled_velocity = continuously_variable_curve_mapping_from_midi_value(
        current_note.velocity_curve.load(Relaxed),
        velocity,
    );

    store_f32_as_atomic_u32(&current_note.velocity, scaled_velocity);

    current_note.midi_note.store(midi_note, Relaxed);

    let mut oscillators = oscillators_arc
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());

    let mut envelopes = envelopes_arc
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());

    update_current_note_from_stored_parameters(midi_note, &mut oscillators);
    action_midi_note_events(
        MidiNoteEvent::NoteOn,
        &mut oscillators,
        &mut envelopes,
        current_note.oscillator_key_sync_enabled.load(Relaxed),
    );
}

pub fn process_midi_cc_values(
    cc_value: CC,
    current_note: &mut Arc<CurrentNote>,
    parameters: &mut Arc<Parameters>,
    midi_event_arc: &mut Arc<Mutex<Option<MidiNoteEvent>>>,
    modules: &mut Modules,
) {
    log::debug!("process_midi_cc_values(): CC received: {:?}", cc_value);
    match cc_value {
        CC::ModWheel(value) => {
            set_mod_wheel(parameters, value);
        }
        CC::VelocityCurve(value) => {
            set_velocity_curve(current_note, value);
        }
        CC::Volume(value) => {
            set_output_volume(parameters, value);
        }
        CC::Balance(value) => {
            set_output_balance(parameters, value);
        }
        CC::SubOscillatorShapeParameter1(value) => {
            set_oscillator_shape_parameter1(&mut modules.oscillators, OscillatorIndex::Sub, value);
        }
        CC::SubOscillatorShapeParameter2(value) => {
            set_oscillator_shape_parameter2(&mut modules.oscillators, OscillatorIndex::Sub, value);
        }
        CC::Oscillator1ShapeParameter1(value) => {
            set_oscillator_shape_parameter1(&mut modules.oscillators, OscillatorIndex::One, value);
        }
        CC::Oscillator1ShapeParameter2(value) => {
            set_oscillator_shape_parameter2(&mut modules.oscillators, OscillatorIndex::One, value);
        }
        CC::Oscillator2ShapeParameter1(value) => {
            set_oscillator_shape_parameter1(&mut modules.oscillators, OscillatorIndex::Two, value);
        }
        CC::Oscillator2ShapeParameter2(value) => {
            set_oscillator_shape_parameter2(&mut modules.oscillators, OscillatorIndex::Two, value);
        }
        CC::Oscillator3ShapeParameter1(value) => {
            set_oscillator_shape_parameter1(
                &mut modules.oscillators,
                OscillatorIndex::Three,
                value,
            );
        }
        CC::Oscillator3ShapeParameter2(value) => {
            set_oscillator_shape_parameter2(
                &mut modules.oscillators,
                OscillatorIndex::Three,
                value,
            );
        }
        CC::OscillatorKeySyncEnabled(value) => {
            set_oscillator_key_sync(current_note, value);
        }
        CC::SubOscillatorShape(value) => {
            set_oscillator_wave_shape(&mut modules.oscillators, OscillatorIndex::Sub, value);
        }
        CC::Oscillator1Shape(value) => {
            set_oscillator_wave_shape(&mut modules.oscillators, OscillatorIndex::One, value);
        }
        CC::Oscillator2Shape(value) => {
            set_oscillator_wave_shape(&mut modules.oscillators, OscillatorIndex::Two, value);
        }
        CC::Oscillator3Shape(value) => {
            set_oscillator_wave_shape(&mut modules.oscillators, OscillatorIndex::Three, value);
        }
        CC::SubOscillatorCourseTune(value) => {
            set_oscillator_course_tune(
                current_note,
                &mut modules.oscillators,
                OscillatorIndex::Sub,
                value,
            );
        }
        CC::Oscillator1CourseTune(value) => {
            set_oscillator_course_tune(
                current_note,
                &mut modules.oscillators,
                OscillatorIndex::One,
                value,
            );
        }
        CC::Oscillator2CourseTune(value) => {
            set_oscillator_course_tune(
                current_note,
                &mut modules.oscillators,
                OscillatorIndex::Two,
                value,
            );
        }
        CC::Oscillator3CourseTune(value) => {
            set_oscillator_course_tune(
                current_note,
                &mut modules.oscillators,
                OscillatorIndex::Three,
                value,
            );
        }
        CC::SubOscillatorFineTune(value) => {
            set_oscillator_fine_tune(
                current_note,
                &mut modules.oscillators,
                OscillatorIndex::Sub,
                value,
            );
        }
        CC::Oscillator1FineTune(value) => {
            set_oscillator_fine_tune(
                current_note,
                &mut modules.oscillators,
                OscillatorIndex::One,
                value,
            );
        }
        CC::Oscillator2FineTune(value) => {
            set_oscillator_fine_tune(
                current_note,
                &mut modules.oscillators,
                OscillatorIndex::Two,
                value,
            );
        }
        CC::Oscillator3FineTune(value) => {
            set_oscillator_fine_tune(
                current_note,
                &mut modules.oscillators,
                OscillatorIndex::Three,
                value,
            );
        }
        CC::SubOscillatorLevel(value) => {
            set_oscillator_level(parameters, OscillatorIndex::Sub, value);
        }
        CC::Oscillator1Level(value) => {
            set_oscillator_level(parameters, OscillatorIndex::One, value);
        }
        CC::Oscillator2Level(value) => {
            set_oscillator_level(parameters, OscillatorIndex::Two, value);
        }
        CC::Oscillator3Level(value) => {
            set_oscillator_level(parameters, OscillatorIndex::Three, value);
        }
        CC::SubOscillatorMute(value) => {
            set_oscillator_mute(parameters, OscillatorIndex::Sub, value);
        }
        CC::Oscillator1Mute(value) => {
            set_oscillator_mute(parameters, OscillatorIndex::One, value);
        }
        CC::Oscillator2Mute(value) => {
            set_oscillator_mute(parameters, OscillatorIndex::Two, value);
        }
        CC::Oscillator3Mute(value) => {
            set_oscillator_mute(parameters, OscillatorIndex::Three, value);
        }
        CC::SubOscillatorBalance(value) => {
            set_oscillator_balance(parameters, OscillatorIndex::Sub, value);
        }
        CC::Oscillator1Balance(value) => {
            set_oscillator_balance(parameters, OscillatorIndex::One, value);
        }
        CC::Oscillator2Balance(value) => {
            set_oscillator_balance(parameters, OscillatorIndex::Two, value);
        }
        CC::Oscillator3Balance(value) => {
            set_oscillator_balance(parameters, OscillatorIndex::Three, value);
        }
        CC::FilterPoles(value) => {
            set_filter_poles(&mut modules.filter, value);
        }
        CC::FilterResonance(value) => {
            set_filter_resonance(&mut modules.filter, value);
        }
        CC::AmpEGReleaseTime(value) => {
            set_amp_eg_release_time(&mut modules.envelopes, EnvelopeIndex::Amplifier, value);
        }
        CC::AmpEGAttackTime(value) => {
            set_amp_eg_attack_time(&mut modules.envelopes, EnvelopeIndex::Amplifier, value);
        }
        CC::FilterCutoff(value) => {
            set_filter_cutoff(&mut modules.filter, value);
        }
        CC::AmpEGDecayTime(value) => {
            set_amp_eg_decay_time(&mut modules.envelopes, EnvelopeIndex::Amplifier, value);
        }
        CC::AmpEGSustainLevel(value) => {
            set_amp_eq_sustain_level(&mut modules.envelopes, EnvelopeIndex::Amplifier, value);
        }
        CC::AmpEGInverted(value) => {
            set_amp_eg_inverted(&mut modules.envelopes, EnvelopeIndex::Amplifier, value);
        }
        CC::FilterEGAttackTime(value) => {
            set_filter_eg_attack_time(&mut modules.envelopes, EnvelopeIndex::Filter, value);
        }
        CC::FilterEGDecayTime(value) => {
            set_filter_eq_decay_time(&mut modules.envelopes, EnvelopeIndex::Filter, value);
        }
        CC::FilterEGSustainLevel(value) => {
            set_filter_eq_sustain_level(&mut modules.envelopes, EnvelopeIndex::Filter, value);
        }
        CC::FilterEGReleaseTime(value) => {
            set_filter_eq_release_time(&mut modules.envelopes, EnvelopeIndex::Filter, value);
        }
        CC::FilterEGInverted(value) => {
            set_filter_eq_inverted(&mut modules.envelopes, EnvelopeIndex::Filter, value);
        }
        CC::FilterEGAmount(value) => {
            set_filter_eg_amount(&mut modules.envelopes, EnvelopeIndex::Filter, value);
        }
        CC::LFO1Frequency(value) => {
            set_lfo_frequency(&mut modules.lfos, LfoIndex::Lfo1, value);
        }
        CC::LFO1CenterValue(value) => {
            set_lfo_center_value(&mut modules.lfos, LfoIndex::Lfo1, value);
        }
        CC::LFO1Range(value) => {
            set_lfo_range(&mut modules.lfos, LfoIndex::Lfo1, value);
        }
        CC::LFO1WaveShape(value) => {
            set_lfo_wave_shape(&mut modules.lfos, LfoIndex::Lfo1, value);
        }
        CC::LFO1Phase(value) => {
            set_lfo_phase(&mut modules.lfos, LfoIndex::Lfo1, value);
        }
        CC::LFO1Reset => {
            lfo_reset(&mut modules.lfos, LfoIndex::Lfo1);
        }
        CC::FilterModLFOFrequency(value) => {
            set_lfo_frequency(&mut modules.lfos, LfoIndex::Filter, value);
        }
        CC::FilterModLFOAmount(value) => {
            set_lfo_range(&mut modules.lfos, LfoIndex::Filter, value);
        }
        CC::FilterModLFOWaveShape(value) => {
            set_lfo_wave_shape(&mut modules.lfos, LfoIndex::Filter, value);
        }
        CC::FilterModLFOPhase(value) => {
            set_lfo_phase(&mut modules.lfos, LfoIndex::Filter, value);
        }
        CC::FilterModLFOReset => {
            lfo_reset(&mut modules.lfos, LfoIndex::Filter);
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

fn set_lfo_frequency(lfos_arc: &mut Arc<Mutex<[Lfo; 2]>>, lfo: LfoIndex, value: u8) {
    let mut lfos = lfos_arc
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());

    lfos[lfo as usize].set_frequency(exponential_curve_lfo_frequency_from_midi_value(value));
}

fn set_lfo_center_value(lfos_arc: &mut Arc<Mutex<[Lfo; 2]>>, lfo: LfoIndex, value: u8) {
    let mut lfos = lfos_arc
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());

    lfos[lfo as usize].set_center_value(midi_value_to_f32_negative_1_to_1(value));
}

fn set_lfo_range(lfos_arc: &mut Arc<Mutex<[Lfo; 2]>>, lfo: LfoIndex, value: u8) {
    let lfo_range = if value == 0 {
        0.0
    } else {
        midi_value_to_f32_0_to_1(value)
    };

    let mut lfos = lfos_arc
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());

    lfos[lfo as usize].set_range(lfo_range);
}

fn set_lfo_wave_shape(lfos_arc: &mut Arc<Mutex<[Lfo; 2]>>, lfo: LfoIndex, value: u8) {
    let mut lfos = lfos_arc
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());

    lfos[lfo as usize].set_wave_shape(midi_value_to_oscillator_wave_shape(value));
}

fn set_lfo_phase(lfos_arc: &mut Arc<Mutex<[Lfo; 2]>>, lfo: LfoIndex, value: u8) {
    let mut lfos = lfos_arc
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());

    lfos[lfo as usize].set_phase(midi_value_to_f32_0_to_1(value));
}

fn lfo_reset(lfos_arc: &mut Arc<Mutex<[Lfo; 2]>>, lfo: LfoIndex) {
    let mut lfos = lfos_arc
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());

    lfos[lfo as usize].reset()
}

fn set_filter_eg_amount(
    envelopes_arc: &mut Arc<Mutex<[Envelope; 2]>>,
    envelope: EnvelopeIndex,
    value: u8,
) {
    let mut envelopes = envelopes_arc
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    envelopes[envelope as usize].set_amount(midi_value_to_f32_0_to_1(value));
}

fn set_filter_eq_inverted(
    envelopes_arc: &mut Arc<Mutex<[Envelope; 2]>>,
    envelope: EnvelopeIndex,
    value: u8,
) {
    let mut envelopes = envelopes_arc
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    envelopes[envelope as usize].set_is_inverted(midi_value_to_bool(value));
}

fn set_filter_eq_release_time(
    envelopes_arc: &mut Arc<Mutex<[Envelope; 2]>>,
    envelope: EnvelopeIndex,
    value: u8,
) {
    let time = midi_value_to_f32_range(value, ENVELOPE_MIN_MILLISECONDS, ENVELOPE_MAX_MILLISECONDS);

    let mut envelopes = envelopes_arc
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());

    envelopes[envelope as usize].set_release_milliseconds(time);
}

fn set_filter_eq_sustain_level(
    envelopes_arc: &mut Arc<Mutex<[Envelope; 2]>>,
    envelope: EnvelopeIndex,
    value: u8,
) {
    let mut envelopes = envelopes_arc
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());

    envelopes[envelope as usize].set_sustain_level(midi_value_to_f32_0_to_1(value));
}

fn set_filter_eq_decay_time(
    envelopes_arc: &mut Arc<Mutex<[Envelope; 2]>>,
    envelope: EnvelopeIndex,
    value: u8,
) {
    let time = midi_value_to_f32_range(value, ENVELOPE_MIN_MILLISECONDS, ENVELOPE_MAX_MILLISECONDS);

    let mut envelopes = envelopes_arc
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());

    envelopes[envelope as usize].set_decay_milliseconds(time);
}

fn set_filter_eg_attack_time(
    envelopes_arc: &mut Arc<Mutex<[Envelope; 2]>>,
    envelope: EnvelopeIndex,
    value: u8,
) {
    let time = midi_value_to_f32_range(value, ENVELOPE_MIN_MILLISECONDS, ENVELOPE_MAX_MILLISECONDS);

    let mut envelopes = envelopes_arc
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());

    envelopes[envelope as usize].set_attack_milliseconds(time);
}

fn set_amp_eg_inverted(
    envelopes_arc: &mut Arc<Mutex<[Envelope; 2]>>,
    envelope: EnvelopeIndex,
    value: u8,
) {
    let mut envelopes = envelopes_arc
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    envelopes[envelope as usize].set_is_inverted(midi_value_to_bool(value));
}

fn set_amp_eq_sustain_level(
    envelopes_arc: &mut Arc<Mutex<[Envelope; 2]>>,
    envelope: EnvelopeIndex,
    value: u8,
) {
    let mut envelopes = envelopes_arc
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());

    envelopes[envelope as usize]
        .set_sustain_level(exponential_curve_level_adjustment_from_midi_value(value));
}

fn set_amp_eg_decay_time(
    envelopes_arc: &mut Arc<Mutex<[Envelope; 2]>>,
    envelope: EnvelopeIndex,
    value: u8,
) {
    let time = midi_value_to_f32_range(value, ENVELOPE_MIN_MILLISECONDS, ENVELOPE_MAX_MILLISECONDS);

    let mut envelopes = envelopes_arc
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());

    envelopes[envelope as usize].set_decay_milliseconds(time);
}

fn set_filter_cutoff(filter_arc: &mut Arc<Mutex<Filter>>, value: u8) {
    let cutoff_frequency = exponential_curve_filter_cutoff_from_midi_value(value);

    let mut filter = filter_arc
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());

    filter.set_cutoff_frequency(cutoff_frequency);
}

fn set_amp_eg_attack_time(
    envelopes_arc: &mut Arc<Mutex<[Envelope; 2]>>,
    envelope: EnvelopeIndex,
    value: u8,
) {
    let time = midi_value_to_f32_range(value, ENVELOPE_MIN_MILLISECONDS, ENVELOPE_MAX_MILLISECONDS);

    let mut envelopes = envelopes_arc
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());

    envelopes[envelope as usize].set_attack_milliseconds(time);
}

fn set_amp_eg_release_time(
    envelopes_arc: &mut Arc<Mutex<[Envelope; 2]>>,
    envelope: EnvelopeIndex,
    value: u8,
) {
    let time = midi_value_to_f32_range(value, ENVELOPE_MIN_MILLISECONDS, ENVELOPE_MAX_MILLISECONDS);
    let mut envelopes = envelopes_arc
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());

    envelopes[envelope as usize].set_release_milliseconds(time);
}

fn set_filter_resonance(filter_arc: &mut Arc<Mutex<Filter>>, value: u8) {
    let mut filter = filter_arc
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());

    filter.set_resonance(midi_value_to_f32_range(value, 0.0, 1.0));
}

fn set_filter_poles(filter_arc: &mut Arc<Mutex<Filter>>, value: u8) {
    let mut filter = filter_arc
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    filter.set_filter_slope(midi_value_to_filter_slope(value));
}

fn set_output_balance(parameters: &mut Arc<Parameters>, value: u8) {
    let output_balance = midi_value_to_f32_negative_1_to_1(value);
    store_f32_as_atomic_u32(&parameters.output_balance, output_balance);
}

fn set_output_volume(parameters: &mut Arc<Parameters>, value: u8) {
    let output_level = exponential_curve_level_adjustment_from_midi_value(value);
    store_f32_as_atomic_u32(&parameters.output_level, output_level);
}

fn set_velocity_curve(current_note: &mut Arc<CurrentNote>, value: u8) {
    current_note.velocity_curve.store(value, Relaxed);
}

fn set_mod_wheel(parameters: &mut Arc<Parameters>, value: u8) {
    let mod_wheel_amount = midi_value_to_f32_0_to_1(value);
    store_f32_as_atomic_u32(&parameters.mod_wheel_amount, mod_wheel_amount);
}

fn set_oscillator_shape_parameter1(
    oscillators_arc: &mut Arc<Mutex<[Oscillator; 4]>>,
    oscillator: OscillatorIndex,
    value: u8,
) {
    let mut oscillators = oscillators_arc
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());

    oscillators[oscillator as usize].set_shape_parameter1(midi_value_to_f32_0_to_1(value));
}

fn set_oscillator_shape_parameter2(
    oscillators_arc: &mut Arc<Mutex<[Oscillator; 4]>>,
    oscillator: OscillatorIndex,
    value: u8,
) {
    let mut oscillators = oscillators_arc
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());

    oscillators[oscillator as usize].set_shape_parameter2(midi_value_to_f32_0_to_1(value));
}

fn set_oscillator_key_sync(current_note: &mut Arc<CurrentNote>, value: u8) {
    current_note
        .oscillator_key_sync_enabled
        .store(midi_value_to_bool(value), Relaxed);
}

fn set_oscillator_balance(
    parameters: &mut Arc<Parameters>,
    oscillator: OscillatorIndex,
    value: u8,
) {
    let balance = midi_value_to_f32_negative_1_to_1(value);
    store_f32_as_atomic_u32(
        &parameters.oscillators[oscillator as usize].mixer_balance,
        balance,
    );
}

fn set_oscillator_mute(parameters: &mut Arc<Parameters>, oscillator: OscillatorIndex, value: u8) {
    let mute = midi_value_to_bool(value);
    parameters.oscillators[oscillator as usize]
        .mixer_mute
        .swap(mute, Relaxed);
}

fn set_oscillator_level(parameters: &mut Arc<Parameters>, oscillator: OscillatorIndex, value: u8) {
    let level = exponential_curve_level_adjustment_from_midi_value(value);
    store_f32_as_atomic_u32(
        &parameters.oscillators[oscillator as usize].mixer_level,
        level,
    );
}

fn set_oscillator_fine_tune(
    current_note: &mut Arc<CurrentNote>,
    oscillators_arc: &mut Arc<Mutex<[Oscillator; 4]>>,
    oscillator: OscillatorIndex,
    value: u8,
) {
    let mut oscillators = oscillators_arc
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());

    oscillators[oscillator as usize].set_fine_tune(Some(midi_value_to_fine_tune_cents(value)));

    let midi_note = current_note.midi_note.load(Relaxed);
    update_current_note_from_stored_parameters(midi_note, &mut oscillators);
}

fn set_oscillator_course_tune(
    current_note: &mut Arc<CurrentNote>,
    oscillators_arc: &mut Arc<Mutex<[Oscillator; 4]>>,
    oscillator: OscillatorIndex,
    value: u8,
) {
    let mut oscillators = oscillators_arc
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());

    oscillators[oscillator as usize]
        .set_course_tune(Some(midi_value_to_course_tune_intervals(value)));

    let midi_note = current_note.midi_note.load(Relaxed);
    update_current_note_from_stored_parameters(midi_note, &mut oscillators);
}

fn set_oscillator_wave_shape(
    oscillators_arc: &mut Arc<Mutex<[Oscillator; 4]>>,
    oscillator: OscillatorIndex,
    value: u8,
) {
    let wave_shape = midi_value_to_oscillator_wave_shape(value);

    let mut oscillators = oscillators_arc
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());

    oscillators[oscillator as usize].set_wave_shape(wave_shape);
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
    oscillators: &mut [Oscillator; 4],
) {
    for oscillator in oscillators.iter_mut() {
        if pitch_bend_amount == PITCH_BEND_AMOUNT_ZERO_POINT {
            oscillator.set_pitch_bend(None);
        } else if pitch_bend_amount == PITCH_BEND_AMOUNT_MAX_VALUE {
            oscillator.set_pitch_bend(Some(PITCH_BEND_AMOUNT_CENTS));
        } else {
            let pitch_bend_in_cents = (pitch_bend_amount - PITCH_BEND_AMOUNT_ZERO_POINT) as f32
                / PITCH_BEND_AMOUNT_ZERO_POINT as f32
                * PITCH_BEND_AMOUNT_CENTS as f32;
            oscillator.set_pitch_bend(Some(pitch_bend_in_cents as i16));
        }
    }
}

fn update_current_note_from_stored_parameters(
    midi_note: u8,
    oscillators: &mut MutexGuard<[Oscillator; 4]>,
) {
    for oscillator in oscillators.iter_mut() {
        oscillator.tune(midi_note);
    }
}

pub fn store_f32_as_atomic_u32(atomic: &AtomicU32, value: f32) {
    atomic.store(value.to_bits(), Relaxed);
}

pub fn load_f32_from_atomic_u32(atomic: &AtomicU32) -> f32 {
    f32::from_bits(atomic.load(Relaxed))
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
