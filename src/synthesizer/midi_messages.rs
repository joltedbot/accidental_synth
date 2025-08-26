use crate::midi::CC;
use crate::modules::lfo::LfoParameters;
use crate::modules::oscillator::{NUMBER_OF_WAVE_SHAPES, OscillatorParameters};
use crate::synthesizer::constants::*;
use crate::synthesizer::{
    CurrentNote, EnvelopeParameters, FilterParameters, KeyboardParameters, MidiGateEvent,
    MidiNoteEvent, MixerParameters, ModuleParameters, OscillatorIndex,
};
use std::sync::Arc;
use std::sync::atomic::{AtomicU32, Ordering::Relaxed};

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
    let aftertouch_amount = midi_value_to_f32_0_to_1(pressure_value);
    store_f32_as_atomic_u32(&parameters.aftertouch_amount, aftertouch_amount);
}

pub fn process_midi_pitch_bend_message(oscillators: &[OscillatorParameters; 4], bend_amount: i16) {
    update_current_note_from_midi_pitch_bend(bend_amount, &oscillators);
}

pub fn process_midi_note_off_message(module_paramters: &mut Arc<ModuleParameters>) {
    action_midi_note_events(MidiNoteEvent::NoteOff, module_paramters);
}

pub fn process_midi_note_on_message(
    module_paramters: &mut Arc<ModuleParameters>,
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

    action_midi_note_events(MidiNoteEvent::NoteOn, module_paramters);
}

pub fn process_midi_cc_values(
    cc_value: CC,
    current_note: &mut Arc<CurrentNote>,
    module_parameters: &mut Arc<ModuleParameters>,
) {
    log::debug!("process_midi_cc_values(): CC received: {:?}", cc_value);
    match cc_value {
        CC::ModWheel(value) => {
            set_mod_wheel(&module_parameters.keyboard, value);
        }
        CC::VelocityCurve(value) => {
            set_velocity_curve(current_note, value);
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
            set_oscillator_key_sync(current_note, value);
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
        CC::FilterPoles(value) => {
            set_filter_poles(&module_parameters.filter, value);
        }
        CC::FilterResonance(value) => {
            set_filter_resonance(&module_parameters.filter, value);
        }
        CC::AmpEGReleaseTime(value) => {
            set_amp_eg_release_time(&module_parameters.amp_envelope, value);
        }
        CC::AmpEGAttackTime(value) => {
            set_amp_eg_attack_time(&module_parameters.amp_envelope, value);
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
    let frequency = exponential_curve_lfo_frequency_from_midi_value(value);
    store_f32_as_atomic_u32(&parameters.frequency, frequency);
}

fn set_lfo_center_value(parameters: &LfoParameters, value: u8) {
    let center_value = midi_value_to_f32_negative_1_to_1(value);
    store_f32_as_atomic_u32(&parameters.center_value, center_value);
}

fn set_lfo_range(parameters: &LfoParameters, value: u8) {
    let lfo_range = if value == 0 {
        0.0
    } else {
        midi_value_to_f32_0_to_1(value)
    };

    store_f32_as_atomic_u32(&parameters.range, lfo_range);
}

fn set_lfo_phase(parameters: &LfoParameters, value: u8) {
    let phase = midi_value_to_f32_0_to_1(value);
    store_f32_as_atomic_u32(&parameters.phase, phase);
}

fn set_lfo_wave_shape(parameters: &LfoParameters, value: u8) {
    let wave_shape_index = midi_value_to_wave_shape_index(value);
    parameters.wave_shape.store(wave_shape_index, Relaxed);
}

fn lfo_reset(parameters: &LfoParameters) {
    parameters.reset.store(true, Relaxed);
}

fn set_envelope_amount(envelope_parameters: &EnvelopeParameters, value: u8) {
    let amount = midi_value_to_f32_0_to_1(value);
    store_f32_as_atomic_u32(&envelope_parameters.amount, amount);
}

fn set_envelope_release_time(envelope_parameters: &EnvelopeParameters, value: u8) {
    let milliseconds = midi_value_to_envelope_milliseconds(value);
    envelope_parameters.release_ms.store(milliseconds, Relaxed);
}

fn set_envelope_sustain_level(envelope_parameters: &EnvelopeParameters, value: u8) {
    let sustain_level = exponential_curve_level_adjustment_from_midi_value(value);
    store_f32_as_atomic_u32(&envelope_parameters.sustain_level, sustain_level);
}

fn set_envelope_decay_time(envelope_parameters: &EnvelopeParameters, value: u8) {
    let milliseconds = midi_value_to_envelope_milliseconds(value);
    envelope_parameters.decay_ms.store(milliseconds, Relaxed);
}

fn set_envelope_attack_time(envelope_parameters: &EnvelopeParameters, value: u8) {
    let milliseconds = midi_value_to_envelope_milliseconds(value);
    envelope_parameters.attack_ms.store(milliseconds, Relaxed);
}

fn set_envelope_inverted(envelope_parameters: &EnvelopeParameters, value: u8) {
    let is_inverted = midi_value_to_bool(value);
    envelope_parameters.is_inverted.store(is_inverted, Relaxed);
}

fn set_amp_eg_attack_time(envelope_parameters: &EnvelopeParameters, value: u8) {
    let milliseconds =
        midi_value_to_u32_range(value, ENVELOPE_MIN_MILLISECONDS, ENVELOPE_MAX_MILLISECONDS);
    envelope_parameters.attack_ms.store(milliseconds, Relaxed);
}

fn set_amp_eg_release_time(envelope_parameters: &EnvelopeParameters, value: u8) {
    let milliseconds =
        midi_value_to_u32_range(value, ENVELOPE_MIN_MILLISECONDS, ENVELOPE_MAX_MILLISECONDS);
    envelope_parameters.release_ms.store(milliseconds, Relaxed);
}

fn set_filter_resonance(filter_parameters: &FilterParameters, value: u8) {
    let resonance = midi_value_to_f32_range(value, MIN_FILTER_RESONANCE, MAX_FILTER_RESONANCE);
    store_f32_as_atomic_u32(&filter_parameters.resonance, resonance);
}

fn set_filter_poles(filter_parameters: &FilterParameters, value: u8) {
    let filter_slope = midi_value_to_filter_slope(value);
    filter_parameters.filter_slope.swap(filter_slope, Relaxed);
}

fn set_filter_cutoff(filter_parameters: &FilterParameters, value: u8) {
    let cutoff_frequency = exponential_curve_filter_cutoff_from_midi_value(value);
    store_f32_as_atomic_u32(&filter_parameters.cutoff_frequency, cutoff_frequency);
}

fn set_output_balance(parameters: &MixerParameters, value: u8) {
    let output_balance = midi_value_to_f32_negative_1_to_1(value);
    store_f32_as_atomic_u32(&parameters.output_balance, output_balance);
}

fn set_output_volume(parameters: &MixerParameters, value: u8) {
    let output_level = exponential_curve_level_adjustment_from_midi_value(value);
    store_f32_as_atomic_u32(&parameters.output_level, output_level);
}

fn set_velocity_curve(current_note: &mut Arc<CurrentNote>, value: u8) {
    current_note.velocity_curve.store(value, Relaxed);
}

fn set_mod_wheel(parameters: &KeyboardParameters, value: u8) {
    let mod_wheel_amount = midi_value_to_f32_0_to_1(value);
    store_f32_as_atomic_u32(&parameters.mod_wheel_amount, mod_wheel_amount);
}

fn set_oscillator_shape_parameter1(parameters: &OscillatorParameters, value: u8) {
    let parameter1 = midi_value_to_f32_0_to_1(value);
    store_f32_as_atomic_u32(&parameters.shape_parameter1, parameter1);
}

fn set_oscillator_shape_parameter2(parameters: &OscillatorParameters, value: u8) {
    let parameter2 = midi_value_to_f32_0_to_1(value);
    store_f32_as_atomic_u32(&parameters.shape_parameter2, parameter2);
}

fn set_oscillator_key_sync(current_note: &mut Arc<CurrentNote>, value: u8) {
    current_note
        .oscillator_key_sync_enabled
        .store(midi_value_to_bool(value), Relaxed);
}

fn set_oscillator_balance(parameters: &MixerParameters, oscillator: OscillatorIndex, value: u8) {
    let balance = midi_value_to_f32_negative_1_to_1(value);
    store_f32_as_atomic_u32(
        &parameters.quad_mixer_inputs[oscillator as usize].mixer_balance,
        balance,
    );
}

fn set_oscillator_mute(parameters: &MixerParameters, oscillator: OscillatorIndex, value: u8) {
    let mute = midi_value_to_bool(value);
    parameters.quad_mixer_inputs[oscillator as usize]
        .mixer_mute
        .swap(mute, Relaxed);
}

fn set_oscillator_level(parameters: &MixerParameters, oscillator: OscillatorIndex, value: u8) {
    let level = exponential_curve_level_adjustment_from_midi_value(value);
    store_f32_as_atomic_u32(
        &parameters.quad_mixer_inputs[oscillator as usize].mixer_level,
        level,
    );
}

fn set_oscillator_fine_tune(parameters: &OscillatorParameters, value: u8) {
    let cents = midi_value_to_fine_tune_cents(value);
    parameters.fine_tune.store(cents, Relaxed);
}

fn set_oscillator_course_tune(parameters: &OscillatorParameters, value: u8) {
    let interval = midi_value_to_course_tune_intervals(value);
    parameters.course_tune.store(interval, Relaxed);
}

fn set_oscillator_wave_shape(parameters: &OscillatorParameters, value: u8) {
    let wave_shape_index = midi_value_to_wave_shape_index(value);
    parameters.wave_shape_index.store(wave_shape_index, Relaxed);
}

pub fn midi_value_to_f32_range(midi_value: u8, minimum: f32, maximum: f32) -> f32 {
    let range = maximum - minimum;
    let increment = range / MAX_MIDI_VALUE as f32;
    minimum + (midi_value as f32 * increment)
}

pub fn midi_value_to_u32_range(midi_value: u8, minimum: u32, maximum: u32) -> u32 {
    let range = maximum - minimum;
    let increment = range as f32 / MAX_MIDI_VALUE as f32;
    minimum + (midi_value as f32 * increment).ceil() as u32
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

fn midi_value_to_envelope_milliseconds(midi_value: u8) -> u32 {
    midi_value_to_u32_range(
        midi_value,
        ENVELOPE_MIN_MILLISECONDS,
        ENVELOPE_MAX_MILLISECONDS,
    )
}

fn midi_value_to_filter_slope(midi_value: u8) -> u8 {
    if midi_value < 32 {
        //FilterSlope::Db6
        1
    } else if midi_value < 64 {
        //FilterSlope::Db12
        2
    } else if midi_value < 96 {
        //FilterSlope::Db18
        3
    } else {
        //FilterSlope::Db24
        4
    }
}

fn midi_value_to_wave_shape_index(midi_value: u8) -> u8 {
    midi_value_to_u32_range(midi_value, 1, NUMBER_OF_WAVE_SHAPES as u32) as u8
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
    oscillators: &[OscillatorParameters; 4],
) {
    for oscillator in oscillators.iter() {
        if pitch_bend_amount == PITCH_BEND_AMOUNT_ZERO_POINT {
            oscillator.pitch_bend.store(0, Relaxed);
        } else if pitch_bend_amount == PITCH_BEND_AMOUNT_MAX_VALUE {
            oscillator
                .pitch_bend
                .store(PITCH_BEND_AMOUNT_CENTS, Relaxed);
        } else {
            let pitch_bend_in_cents = (pitch_bend_amount - PITCH_BEND_AMOUNT_ZERO_POINT) as f32
                / PITCH_BEND_AMOUNT_ZERO_POINT as f32
                * PITCH_BEND_AMOUNT_CENTS as f32;
            oscillator
                .pitch_bend
                .store(pitch_bend_in_cents as i16, Relaxed);
        }
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
