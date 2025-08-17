use crate::midi::CC;
use crate::modules::envelope::{ENVELOPE_MAX_MILLISECONDS, ENVELOPE_MIN_MILLISECONDS, Envelope};
use crate::modules::filter::{Filter, FilterSlope};
use crate::modules::mixer::{Mixer, MixerInput};
use crate::modules::oscillator::{Oscillator, WaveShape};
use crate::synthesizer::constants::*;
use crate::synthesizer::tuner::tune;
use crate::synthesizer::{MidiNoteEvent, Parameters};
use std::sync::{Arc, Mutex, MutexGuard};

pub fn process_midi_note_events(
    midi_events: Option<MidiNoteEvent>,
    amp_envelope: &mut MutexGuard<Envelope>,
    filter_envelope: &mut MutexGuard<Envelope>,
) {
    match midi_events {
        Some(MidiNoteEvent::NoteOn) => {
            amp_envelope.gate_on();
            filter_envelope.gate_on();
        }
        Some(MidiNoteEvent::NoteOff) => {
            amp_envelope.gate_off();
            filter_envelope.gate_off();
        }
        None => {}
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
    amp_envelope_arc: &mut Arc<Mutex<Envelope>>,
    oscillators_arc: &mut Arc<Mutex<[Oscillator; 4]>>,
    midi_event_arc: &mut Arc<Mutex<Option<MidiNoteEvent>>>,
    filter_arc: &mut Arc<Mutex<Filter>>,
    filter_envelope_arc: &mut Arc<Mutex<Envelope>>,
    mixer_arc: &mut Arc<Mutex<Mixer>>,
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
            set_output_volume(mixer_arc, value);
        }
        CC::ConstantLevel(value) => {
            set_output_constant_level(mixer_arc, value);
        }
        CC::Pan(value) => {
            set_output_pan(mixer_arc, value);
        }
        CC::SubOscillatorShapeParameter1(value) => {
            //TODO
        }
        CC::SubOscillatorShapeParameter2(value) => {
            //TODO
        }
        CC::Oscillator1ShapeParameter1(value) => {
            //TODO
        }
        CC::Oscillator1ShapeParameter2(value) => {
            //TODO
        }
        CC::Oscillator2ShapeParameter1(value) => {
            //TODO
        }
        CC::Oscillator2ShapeParameter2(value) => {
            //TODO
        }
        CC::Oscillator3ShapeParameter1(value) => {
            //TODO
        }
        CC::Oscillator3ShapeParameter2(value) => {
            //TODO
        }
        CC::SubOscillatorShape(value) => {
            set_oscillator_wave_shape(parameters_arc, oscillators_arc, 0, value);
        }
        CC::Oscillator1Shape(value) => {
            set_oscillator_wave_shape(parameters_arc, oscillators_arc, 1, value);
        }
        CC::Oscillator2Shape(value) => {
            set_oscillator_wave_shape(parameters_arc, oscillators_arc, 2, value);
        }
        CC::Oscillator3Shape(value) => {
            set_oscillator_wave_shape(parameters_arc, oscillators_arc, 3, value);
        }
        CC::SubOscillatorCourseTune(value) => {
            set_oscillator_course_tune(parameters_arc, 0, value);
        }
        CC::Oscillator1CourseTune(value) => {
            set_oscillator_course_tune(parameters_arc, 1, value);
        }
        CC::Oscillator2CourseTune(value) => {
            set_oscillator_course_tune(parameters_arc, 2, value);
        }
        CC::Oscillator3CourseTune(value) => {
            set_oscillator_course_tune(parameters_arc, 3, value);
        }
        CC::SubOscillatorFineTune(value) => {
            set_oscillator_fine_tune(parameters_arc, 0, value);
        }
        CC::Oscillator1FineTune(value) => {
            set_oscillator_fine_tune(parameters_arc, 1, value);
        }
        CC::Oscillator2FineTune(value) => {
            set_oscillator_fine_tune(parameters_arc, 2, value);
        }
        CC::Oscillator3FineTune(value) => {
            set_oscillator_fine_tune(parameters_arc, 3, value);
        }
        CC::SubOscillatorLevel(value) => {
            set_oscillator_level(parameters_arc, mixer_arc, 0, MixerInput::One, value);
        }
        CC::Oscillator1Level(value) => {
            set_oscillator_level(parameters_arc, mixer_arc, 1, MixerInput::Two, value);
        }
        CC::Oscillator2Level(value) => {
            set_oscillator_level(parameters_arc, mixer_arc, 2, MixerInput::Three, value);
        }
        CC::Oscillator3Level(value) => {
            set_oscillator_level(parameters_arc, mixer_arc, 4, MixerInput::Four, value);
        }
        CC::SubOscillatorMute(value) => {
            set_oscillator_mute(parameters_arc, mixer_arc, 0, MixerInput::One, value);
        }
        CC::Oscillator1Mute(value) => {
            set_oscillator_mute(parameters_arc, mixer_arc, 1, MixerInput::Two, value);
        }
        CC::Oscillator2Mute(value) => {
            set_oscillator_mute(parameters_arc, mixer_arc, 2, MixerInput::Three, value);
        }
        CC::Oscillator3Mute(value) => {
            set_oscillator_mute(parameters_arc, mixer_arc, 3, MixerInput::Four, value);
        }
        CC::SubOscillatorPan(value) => {
            set_oscillator_pan(parameters_arc, mixer_arc, 0, MixerInput::One, value);
        }
        CC::Oscillator1Pan(value) => {
            set_oscillator_pan(parameters_arc, mixer_arc, 1, MixerInput::Two, value);
        }
        CC::Oscillator2Pan(value) => {
            set_oscillator_pan(parameters_arc, mixer_arc, 2, MixerInput::Three, value);
        }
        CC::Oscillator3Pan(value) => {
            set_oscillator_pan(parameters_arc, mixer_arc, 3, MixerInput::Four, value);
        }
        CC::FilterPoles(value) => {
            set_filter_poles(filter_arc, value);
        }
        CC::FilterResonance(value) => {
            set_filter_resonance(filter_arc, value);
        }
        CC::AmpEGReleaseTime(value) => {
            set_amp_eg_release_time(amp_envelope_arc, value);
        }
        CC::AmpEGAttackTime(value) => {
            set_amp_eg_attack_time(amp_envelope_arc, value);
        }
        CC::FilterCutoff(value) => {
            set_filter_cutoff(filter_arc, value);
        }
        CC::AmpEGDecayTime(value) => {
            set_amp_eg_decay_time(amp_envelope_arc, value);
        }
        CC::AmpEGSustainLevel(value) => {
            set_amp_eq_sustain_level(amp_envelope_arc, value);
        }
        CC::AmpEGInverted(value) => {
            set_amp_eg_inverted(amp_envelope_arc, value);
        }
        CC::FilterEGAttackTime(value) => {
            set_filter_eg_attack_time(filter_envelope_arc, value);
        }
        CC::FilterEGDecayTime(value) => {
            set_filter_eq_decay_time(filter_envelope_arc, value);
        }
        CC::FilterEGSustainLevel(value) => {
            set_filter_eq_sustain_level(filter_envelope_arc, value);
        }
        CC::FilterEGReleaseTime(value) => {
            set_filter_eq_release_time(filter_envelope_arc, value);
        }
        CC::FilterEGInverted(value) => {
            set_filter_eq_inverted(filter_envelope_arc, value);
        }
        CC::FilterEGAmount(value) => {
            set_filter_eg_amount(filter_envelope_arc, value);
        }
        CC::LFO1Frequency(value) => {
            //TODO
        }
        CC::LFO1CenterValue(value) => {
            //TODO
        }
        CC::LFO1Range(value) => {
            //TODO
        }
        CC::LFO1WaveShape(value) => {
            //TODO
        }
        CC::LFO1Inverted(value) => {
            //TODO
        }
        CC::LFO1Reset(value) => {
            //TODO
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

fn set_filter_eg_amount(filter_envelope_arc: &mut Arc<Mutex<Envelope>>, value: u8) {
    let mut filter_envelope = filter_envelope_arc
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    filter_envelope.set_amount(midi_value_to_f32_0_to_1(value));
}

fn set_filter_eq_inverted(filter_envelope_arc: &mut Arc<Mutex<Envelope>>, value: u8) {
    let mut filter_envelope = filter_envelope_arc
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    filter_envelope.set_is_inverted(midi_value_to_bool(value));
}

fn set_filter_eq_release_time(filter_envelope_arc: &mut Arc<Mutex<Envelope>>, value: u8) {
    let time = midi_value_to_f32_range(value, ENVELOPE_MIN_MILLISECONDS, ENVELOPE_MAX_MILLISECONDS);

    let mut envelope = filter_envelope_arc
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());

    envelope.set_release_milliseconds(time);
}

fn set_filter_eq_sustain_level(filter_envelope_arc: &mut Arc<Mutex<Envelope>>, value: u8) {
    let mut envelope = filter_envelope_arc
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());

    envelope.set_sustain_level(midi_value_to_f32_0_to_1(value));
}

fn set_filter_eq_decay_time(filter_envelope_arc: &mut Arc<Mutex<Envelope>>, value: u8) {
    let time = midi_value_to_f32_range(value, ENVELOPE_MIN_MILLISECONDS, ENVELOPE_MAX_MILLISECONDS);

    let mut envelope = filter_envelope_arc
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());

    envelope.set_decay_milliseconds(time);
}

fn set_filter_eg_attack_time(filter_envelope_arc: &mut Arc<Mutex<Envelope>>, value: u8) {
    let time = midi_value_to_f32_range(value, ENVELOPE_MIN_MILLISECONDS, ENVELOPE_MAX_MILLISECONDS);

    let mut envelope = filter_envelope_arc
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());

    envelope.set_attack_milliseconds(time);
}

fn set_amp_eg_inverted(amp_envelope_arc: &mut Arc<Mutex<Envelope>>, value: u8) {
    let mut envelope = amp_envelope_arc
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    envelope.set_is_inverted(midi_value_to_bool(value));
}

fn set_amp_eq_sustain_level(amp_envelope_arc: &mut Arc<Mutex<Envelope>>, value: u8) {
    let mut envelope = amp_envelope_arc
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());

    envelope.set_sustain_level(midi_value_to_f32_0_to_1(value));
}

fn set_amp_eg_decay_time(amp_envelope_arc: &mut Arc<Mutex<Envelope>>, value: u8) {
    let time = midi_value_to_f32_range(value, ENVELOPE_MIN_MILLISECONDS, ENVELOPE_MAX_MILLISECONDS);

    let mut envelope = amp_envelope_arc
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());

    envelope.set_decay_milliseconds(time);
}

fn set_filter_cutoff(filter_arc: &mut Arc<Mutex<Filter>>, value: u8) {
    let mut filter = filter_arc
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());

    filter.set_cutoff_frequency(exponential_curve_filter_cutoff_from_midi_value(value));
}

fn set_amp_eg_attack_time(amp_envelope_arc: &mut Arc<Mutex<Envelope>>, value: u8) {
    let time = midi_value_to_f32_range(value, ENVELOPE_MIN_MILLISECONDS, ENVELOPE_MAX_MILLISECONDS);

    let mut envelope = amp_envelope_arc
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());

    envelope.set_attack_milliseconds(time);
}

fn set_amp_eg_release_time(amp_envelope_arc: &mut Arc<Mutex<Envelope>>, value: u8) {
    let time = midi_value_to_f32_range(value, ENVELOPE_MIN_MILLISECONDS, ENVELOPE_MAX_MILLISECONDS);
    let mut envelope = amp_envelope_arc
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());

    envelope.set_release_milliseconds(time);
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

fn set_output_pan(mixer_arc: &mut Arc<Mutex<Mixer>>, value: u8) {
    let output_pan = midi_value_to_f32_negative_1_to_1(value);

    let mut mixer = mixer_arc
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());

    mixer.set_output_pan(output_pan);
}

fn set_output_constant_level(mixer_arc: &mut Arc<Mutex<Mixer>>, value: u8) {
    let is_constant_level = midi_value_to_bool(value);
    let mut mixer = mixer_arc
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());

    mixer.set_constant_level(is_constant_level)
}

fn set_output_volume(mixer_arc: &mut Arc<Mutex<Mixer>>, value: u8) {
    let output_level = exponential_curve_level_adjustment_from_midi_value(value);

    let mut mixer = mixer_arc
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());

    mixer.set_output_level(output_level);
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

fn set_oscillator_pan(
    parameters_arc: &mut Arc<Mutex<Parameters>>,
    mixer_arc: &mut Arc<Mutex<Mixer>>,
    oscillator: usize,
    mixer_input: MixerInput,
    value: u8,
) {
    let pan = midi_value_to_f32_negative_1_to_1(value);

    let mut mixer = mixer_arc
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());

    mixer.set_quad_pan(pan, mixer_input);

    let mut parameters = parameters_arc
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());

    parameters.oscillators[oscillator].pan = pan;
}

fn set_oscillator_mute(
    parameters_arc: &mut Arc<Mutex<Parameters>>,
    mixer_arc: &mut Arc<Mutex<Mixer>>,
    oscillator: usize,
    mixer_input: MixerInput,
    value: u8,
) {
    let mute = midi_value_to_bool(value);

    let mut mixer = mixer_arc
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());

    mixer.set_quad_mute(mute, mixer_input);

    let mut parameters = parameters_arc
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());

    parameters.oscillators[oscillator].mute = mute;
}

fn set_oscillator_level(
    parameters_arc: &mut Arc<Mutex<Parameters>>,
    mixer_arc: &mut Arc<Mutex<Mixer>>,
    oscillator: usize,
    mixer_input: MixerInput,
    value: u8,
) {
    let level = exponential_curve_level_adjustment_from_midi_value(value);

    let mut mixer = mixer_arc
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());

    mixer.set_quad_level(level, mixer_input);

    let mut parameters = parameters_arc
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());

    parameters.oscillators[oscillator].level = level;
}

fn set_oscillator_fine_tune(
    parameters_arc: &mut Arc<Mutex<Parameters>>,
    oscillator: usize,
    value: u8,
) {
    let mut parameters = parameters_arc
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());

    parameters.oscillators[oscillator].fine_tune = Some(midi_value_to_fine_tune_cents(value));
    update_current_note_from_stored_parameters(&mut parameters);
}

fn set_oscillator_course_tune(
    parameters_arc: &mut Arc<Mutex<Parameters>>,
    oscillator: usize,
    value: u8,
) {
    let mut parameters = parameters_arc
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());

    parameters.oscillators[oscillator].course_tune =
        Some(midi_value_to_course_tune_intervals(value));
    update_current_note_from_stored_parameters(&mut parameters);
}

fn set_oscillator_wave_shape(
    parameters_arc: &mut Arc<Mutex<Parameters>>,
    oscillators_arc: &mut Arc<Mutex<[Oscillator; 4]>>,
    oscillator: usize,
    value: u8,
) {
    let wave_shape = midi_value_to_oscillator_wave_shape(value);

    let mut parameters = parameters_arc
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());

    parameters.oscillators[oscillator].wave_shape = wave_shape;

    let mut oscillators = oscillators_arc
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());

    oscillators[oscillator].set_wave_shape(wave_shape);
}

pub fn midi_value_to_f32_range(midi_value: u8, minimum: f32, maximum: f32) -> f32 {
    let range = maximum - minimum;
    let increment = range / MAX_MIDI_NAME as f32;
    minimum + (midi_value as f32 * increment)
}

fn midi_value_to_f32_0_to_1(midi_value: u8) -> f32 {
    midi_value_to_f32_range(midi_value, 0.0, 1.0)
}

fn midi_value_to_f32_negative_1_to_1(midi_value: u8) -> f32 {
    midi_value_to_f32_range(midi_value, -1.0, 1.0)
}

fn midi_value_to_bool(midi_value: u8) -> bool {
    midi_value > 63
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
        WaveShape::AM
    } else if midi_value < 26 {
        WaveShape::FM
    } else if midi_value < 39 {
        WaveShape::Noise
    } else if midi_value < 52 {
        WaveShape::Pulse
    } else if midi_value < 65 {
        WaveShape::Ramp
    } else if midi_value < 78 {
        WaveShape::Saw
    } else if midi_value < 91 {
        WaveShape::Sine
    } else if midi_value < 104 {
        WaveShape::Square
    } else if midi_value < 117 {
        WaveShape::SuperSaw
    } else {
        WaveShape::Triangle
    }
}

fn exponential_curve_filter_cutoff_from_midi_value(midi_value: u8) -> f32 {
    if midi_value == 0 {
        return 0.0;
    }
    exponential_curve_from_midi_value_and_coefficient(midi_value, EXPONENTIAL_FILTER_COEFFICIENT)
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
    (exponential_coefficient * (midi_value as f32 / MAX_MIDI_NAME as f32)).exp()
}
fn continuously_variable_curve_mapping_from_midi_value(
    mut slope_midi_value: u8,
    input_midi_exponent: u8,
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

    (input_midi_exponent as f32).powf(curve_exponent) / (MAX_MIDI_NAME as f32).powf(curve_exponent)
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
