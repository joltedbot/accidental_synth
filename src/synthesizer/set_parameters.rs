use crate::math::store_f32_as_atomic_u32;
use crate::modules::effects::{AudioEffectParameters, EffectIndex};
use crate::modules::envelope::{
    EnvelopeParameters, MAX_ATTACK_MILLISECONDS, MAX_DECAY_MILLISECONDS, MAX_RELEASE_MILLISECONDS,
    MIN_ATTACK_MILLISECONDS, MIN_DECAY_MILLISECONDS, MIN_RELEASE_MILLISECONDS,
};
use crate::modules::filter::FilterParameters;
use crate::modules::lfo::{
    DEFAULT_LFO_PHASE, LfoParameters, MAX_LFO_CENTER_VALUE, MIN_LFO_CENTER_VALUE,
};
use crate::modules::oscillator::OscillatorParameters;
use crate::modules::oscillator::constants::{MAX_CLIP_BOOST, MIN_CLIP_BOOST};
use crate::synthesizer::constants::{
    EXPONENTIAL_ENVELOPE_CURVE_ATTACK_VALUES, EXPONENTIAL_ENVELOPE_CURVE_DECAY_VALUES,
    EXPONENTIAL_ENVELOPE_CURVE_RELEASE_VALUES, EXPONENTIAL_PORTAMENTO_COEFFICIENT,
    MAX_FILTER_RESONANCE, MAX_PITCH_BEND_RANGE, MIN_FILTER_RESONANCE, MIN_PITCH_BEND_RANGE,
    OSCILLATOR_COURSE_TUNE_MAX_INTERVAL, OSCILLATOR_COURSE_TUNE_MIN_INTERVAL,
    OSCILLATOR_FINE_TUNE_MAX_CENTS, OSCILLATOR_FINE_TUNE_MIN_CENTS,
};
use crate::synthesizer::midi_value_converters::{
    exponential_curve_envelope_time_from_normal_value,
    exponential_curve_filter_cutoff_from_midi_value,
    exponential_curve_from_normal_value_and_coefficient,
    exponential_curve_level_adjustment_from_normal_value,
    exponential_curve_lfo_frequency_from_normal_value, normal_value_to_bool,
    normal_value_to_f32_range, normal_value_to_integer_range,
    normal_value_to_number_of_filter_poles, normal_value_to_unsigned_integer_range,
    normal_value_to_wave_shape_index, velocity_curve_from_normal_value,
};
use crate::synthesizer::{CurrentNote, KeyboardParameters, MixerParameters, OscillatorIndex};
use std::sync::Arc;
use std::sync::atomic::Ordering::Relaxed;

pub fn set_lfo_frequency(parameters: &LfoParameters, normal_value: f32) -> f32 {
    let frequency = exponential_curve_lfo_frequency_from_normal_value(normal_value);
    store_f32_as_atomic_u32(&parameters.frequency, frequency);
    frequency
}

pub fn set_lfo_center_value(parameters: &LfoParameters, normal_value: f32) {
    let center_value =
        normal_value_to_f32_range(normal_value, MIN_LFO_CENTER_VALUE, MAX_LFO_CENTER_VALUE);
    store_f32_as_atomic_u32(&parameters.center_value, center_value);
}

pub fn set_lfo_range(parameters: &LfoParameters, normal_value: f32) {
    store_f32_as_atomic_u32(&parameters.range, normal_value);
}

pub fn set_lfo_phase(parameters: &LfoParameters, normal_value: f32) {
    store_f32_as_atomic_u32(&parameters.phase, normal_value);
}

pub fn set_lfo_wave_shape(parameters: &LfoParameters, normal_value: f32) {
    let wave_shape_index = normal_value_to_wave_shape_index(normal_value);
    parameters.wave_shape.store(wave_shape_index, Relaxed);
}

pub fn set_lfo_phase_reset(parameters: &LfoParameters) {
    parameters.phase.store(DEFAULT_LFO_PHASE as u32, Relaxed);
    parameters.reset.store(true, Relaxed);
}

pub fn set_key_tracking_amount(filter_parameters: &FilterParameters, normal_value: f32) {
    store_f32_as_atomic_u32(&filter_parameters.key_tracking_amount, normal_value);
}

pub fn set_envelope_amount(envelope_parameters: &EnvelopeParameters, normal_value: f32) {
    store_f32_as_atomic_u32(&envelope_parameters.amount, normal_value);
}

pub fn set_envelope_release_time(envelope_parameters: &EnvelopeParameters, normal_value: f32) {
    let milliseconds = exponential_curve_envelope_time_from_normal_value(
        normal_value,
        EXPONENTIAL_ENVELOPE_CURVE_RELEASE_VALUES,
        MIN_RELEASE_MILLISECONDS,
        MAX_RELEASE_MILLISECONDS,
    );
    envelope_parameters.release_ms.store(milliseconds, Relaxed);
}

pub fn set_envelope_sustain_level(envelope_parameters: &EnvelopeParameters, normal_value: f32) {
    store_f32_as_atomic_u32(&envelope_parameters.sustain_level, normal_value);
}

pub fn set_envelope_sustain_pedal(envelope_parameters: &[EnvelopeParameters], normal_value: f32) {
    for envelope in envelope_parameters {
        envelope
            .sustain_pedal
            .store(normal_value_to_bool(normal_value), Relaxed);
    }
}

pub fn set_envelope_decay_time(envelope_parameters: &EnvelopeParameters, normal_value: f32) {
    let milliseconds = exponential_curve_envelope_time_from_normal_value(
        normal_value,
        EXPONENTIAL_ENVELOPE_CURVE_DECAY_VALUES,
        MIN_DECAY_MILLISECONDS,
        MAX_DECAY_MILLISECONDS,
    );
    envelope_parameters.decay_ms.store(milliseconds, Relaxed);
}

pub fn set_envelope_attack_time(envelope_parameters: &EnvelopeParameters, normal_value: f32) {
    let milliseconds = exponential_curve_envelope_time_from_normal_value(
        normal_value,
        EXPONENTIAL_ENVELOPE_CURVE_ATTACK_VALUES,
        MIN_ATTACK_MILLISECONDS,
        MAX_ATTACK_MILLISECONDS,
    );
    envelope_parameters.attack_ms.store(milliseconds, Relaxed);
}

pub fn set_envelope_inverted(envelope_parameters: &EnvelopeParameters, normal_value: f32) {
    let is_inverted = normal_value_to_bool(normal_value);
    envelope_parameters.is_inverted.store(is_inverted, Relaxed);
}

pub fn set_filter_resonance(filter_parameters: &FilterParameters, normal_value: f32) {
    let resonance =
        normal_value_to_f32_range(normal_value, MIN_FILTER_RESONANCE, MAX_FILTER_RESONANCE);

    store_f32_as_atomic_u32(&filter_parameters.resonance, resonance);
}

pub fn set_filter_poles(filter_parameters: &FilterParameters, normal_value: f32) {
    let filter_poles = normal_value_to_number_of_filter_poles(normal_value);
    filter_parameters.filter_poles.swap(filter_poles, Relaxed);
}

pub fn set_filter_cutoff(filter_parameters: &FilterParameters, normal_value: f32) {
    let cutoff_frequency = exponential_curve_filter_cutoff_from_midi_value(normal_value);
    store_f32_as_atomic_u32(&filter_parameters.cutoff_frequency, cutoff_frequency);
}

pub fn set_output_balance(parameters: &MixerParameters, normal_value: f32) {
    let output_balance = normal_value_to_f32_range(normal_value, -1.0, 1.0);
    store_f32_as_atomic_u32(&parameters.output_balance, output_balance);
}

pub fn set_output_level(parameters: &MixerParameters, normal_value: f32) {
    let output_level = exponential_curve_level_adjustment_from_normal_value(normal_value);
    store_f32_as_atomic_u32(&parameters.output_level, output_level);
}

pub fn set_output_mute(parameters: &MixerParameters, normal_value: f32) {
    let is_muted = normal_value_to_bool(normal_value);
    parameters.output_is_muted.store(is_muted, Relaxed);
}

pub fn set_velocity_curve(current_note: &mut Arc<CurrentNote>, normal_value: f32) {
    let velocity_curve = velocity_curve_from_normal_value(normal_value);
    store_f32_as_atomic_u32(&current_note.velocity_curve, velocity_curve);
}

pub fn set_pitch_bend_range(parameters: &KeyboardParameters, normal_value: f32) {
    let range = normal_value_to_integer_range(
        normal_value,
        u32::from(MIN_PITCH_BEND_RANGE),
        u32::from(MAX_PITCH_BEND_RANGE),
    ) as u8;
    parameters.pitch_bend_range.store(range, Relaxed);
}

pub fn set_mod_wheel(parameters: &KeyboardParameters, normal_value: f32) {
    store_f32_as_atomic_u32(&parameters.mod_wheel_amount, normal_value);
}

pub fn set_oscillator_shape_parameter1(parameters: &OscillatorParameters, normal_value: f32) {
    store_f32_as_atomic_u32(&parameters.shape_parameter1, normal_value);
}

pub fn set_oscillator_shape_parameter2(parameters: &OscillatorParameters, normal_value: f32) {
    store_f32_as_atomic_u32(&parameters.shape_parameter2, normal_value);
}

pub fn set_oscillator_key_sync(parameters: &[OscillatorParameters; 4], normal_value: f32) {
    for parameters in parameters {
        parameters
            .key_sync_enabled
            .store(normal_value_to_bool(normal_value), Relaxed);
    }
}

pub fn set_oscillator_hard_sync(parameters: &[OscillatorParameters; 4], normal_value: f32) {
    for parameters in parameters {
        parameters
            .hard_sync_enabled
            .store(normal_value_to_bool(normal_value), Relaxed);
    }
}

pub fn set_portamento_time(parameters: &[OscillatorParameters; 4], normal_value: f32) {
    let speed = exponential_curve_from_normal_value_and_coefficient(
        normal_value,
        EXPONENTIAL_PORTAMENTO_COEFFICIENT,
    )
    .round() as u16;

    for parameters in parameters {
        parameters.portamento_time.store(speed, Relaxed);
    }
}

pub fn set_oscillator_clip_boost(parameters: &OscillatorParameters, normal_value: f32) {
    let boost = normal_value_to_integer_range(
        normal_value,
        u32::from(MIN_CLIP_BOOST),
        u32::from(MAX_CLIP_BOOST),
    ) as u8;
    parameters.clipper_boost.store(boost, Relaxed);
}

pub fn set_portamento_enabled(parameters: &[OscillatorParameters; 4], normal_value: f32) {
    for parameters in parameters {
        parameters
            .portamento_enabled
            .store(normal_value_to_bool(normal_value), Relaxed);
    }
}

pub fn set_oscillator_balance(
    parameters: &MixerParameters,
    oscillator: OscillatorIndex,
    normal_value: f32,
) {
    let balance = normal_value_to_f32_range(normal_value, -1.0, 1.0);
    store_f32_as_atomic_u32(
        &parameters.quad_mixer_inputs[oscillator as usize].balance,
        balance,
    );
}

pub fn set_oscillator_mute(
    parameters: &MixerParameters,
    oscillator: OscillatorIndex,
    normal_value: f32,
) {
    let mute = normal_value_to_bool(normal_value);
    parameters.quad_mixer_inputs[oscillator as usize]
        .mute
        .swap(mute, Relaxed);
}

pub fn set_oscillator_level(
    parameters: &MixerParameters,
    oscillator: OscillatorIndex,
    normal_value: f32,
) {
    let level = exponential_curve_level_adjustment_from_normal_value(normal_value);
    store_f32_as_atomic_u32(
        &parameters.quad_mixer_inputs[oscillator as usize].level,
        level,
    );
}

pub fn set_oscillator_fine_tune(parameters: &OscillatorParameters, normal_value: f32) -> i8 {
    let cents = normal_value_to_unsigned_integer_range(
        normal_value,
        i32::from(OSCILLATOR_FINE_TUNE_MIN_CENTS),
        i32::from(OSCILLATOR_FINE_TUNE_MAX_CENTS),
    ) as i8;

    parameters.fine_tune.store(cents, Relaxed);
    cents
}

pub fn set_oscillator_course_tune(parameters: &OscillatorParameters, normal_value: f32) -> i8 {
    let interval = normal_value_to_unsigned_integer_range(
        normal_value,
        i32::from(OSCILLATOR_COURSE_TUNE_MIN_INTERVAL),
        i32::from(OSCILLATOR_COURSE_TUNE_MAX_INTERVAL),
    ) as i8;

    parameters.course_tune.store(interval, Relaxed);
    interval
}

pub fn set_oscillator_wave_shape(parameters: &OscillatorParameters, normal_value: f32) -> u8 {
    let wave_shape_index = normal_value_to_wave_shape_index(normal_value);
    parameters.wave_shape_index.store(wave_shape_index, Relaxed);
    wave_shape_index
}

pub fn set_effect_is_enabled(
    parameters: &Vec<AudioEffectParameters>,
    effect: EffectIndex,
    normal_value: f32,
) {
    parameters[effect as usize].is_enabled.store(normal_value_to_bool(normal_value), Relaxed);
}


pub fn set_effect_parameter(
    parameters: &Vec<AudioEffectParameters>,
    effect: EffectIndex,
    parameter_index: i32,
    parameter: f32,
) {
    store_f32_as_atomic_u32(&parameters[effect as usize].parameters[parameter_index as usize], parameter);
}