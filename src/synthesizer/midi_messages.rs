use crate::midi::{CC, MidiMessage};
use crate::modules::envelope::{ENVELOPE_MAX_MILLISECONDS, ENVELOPE_MIN_MILLISECONDS, Envelope};
use crate::modules::filter::{Filter, FilterSlope};
use crate::modules::mixer::{Mixer, MixerInput};
use crate::modules::oscillator::Oscillator;
use crate::modules::tuner::tune;
use crate::synthesizer::constants::*;
use crate::synthesizer::{MidiNoteEvent, Parameters};
use crossbeam_channel::Receiver;
use std::sync::{Arc, Mutex, MutexGuard};
use std::thread;

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

pub fn start_midi_event_listener(
    midi_message_receiver: Receiver<MidiMessage>,
    mut parameters_arc: Arc<Mutex<Parameters>>,
    mut midi_event_arc: Arc<Mutex<Option<MidiNoteEvent>>>,
    mut amp_envelope_arc: Arc<Mutex<Envelope>>,
    mut oscillators_arc: Arc<Mutex<[Oscillator; 4]>>,
    mut filter_arc: Arc<Mutex<Filter>>,
    mut mixer_arc: Arc<Mutex<Mixer>>,
) {
    thread::spawn(move || {
        log::debug!("run(): spawned thread to receive MIDI events");

        while let Ok(event) = midi_message_receiver.recv() {
            match event {
                MidiMessage::NoteOn(midi_note, velocity) => {
                    let mut parameters = parameters_arc
                        .lock()
                        .unwrap_or_else(|poisoned| poisoned.into_inner());

                    parameters.current_note.velocity = match parameters.is_fixed_velocity {
                        false => Some(velocity as f32 * MIDI_VELOCITY_TO_SAMPLE_FACTOR),
                        true => None,
                    };

                    parameters.current_note.midi_note = midi_note;
                    update_current_note_from_stored_parameters(&mut parameters);

                    let mut midi_events = midi_event_arc
                        .lock()
                        .unwrap_or_else(|poisoned| poisoned.into_inner());

                    *midi_events = Some(MidiNoteEvent::NoteOn);
                }
                MidiMessage::NoteOff => {
                    let mut midi_events = midi_event_arc
                        .lock()
                        .unwrap_or_else(|poisoned| poisoned.into_inner());

                    *midi_events = Some(MidiNoteEvent::NoteOff);
                }
                MidiMessage::PitchBend(bend_amount) => {
                    let mut parameters = parameters_arc
                        .lock()
                        .unwrap_or_else(|poisoned| poisoned.into_inner());

                    update_current_note_from_midi_pitch_bend(bend_amount, &mut parameters);
                    update_current_note_from_stored_parameters(&mut parameters);
                }
                MidiMessage::ChannelPressure(pressure_value) => {
                    let mut parameters = parameters_arc
                        .lock()
                        .unwrap_or_else(|poisoned| poisoned.into_inner());
                    parameters.aftertouch_amount = midi_value_to_f32_0_to_1(pressure_value);
                }
                MidiMessage::ControlChange(cc_value) => {
                    process_midi_cc_values(
                        cc_value,
                        &mut parameters_arc,
                        &mut amp_envelope_arc,
                        &mut oscillators_arc,
                        &mut midi_event_arc,
                        &mut filter_arc,
                        &mut mixer_arc,
                    );
                }
            }
        }

        log::debug!("run(): MIDI event receiver thread has exited");
    });
}

pub fn process_midi_cc_values(
    cc_value: CC,
    parameters_arc: &mut Arc<Mutex<Parameters>>,
    amp_envelope_arc: &mut Arc<Mutex<Envelope>>,
    oscillators_arc: &mut Arc<Mutex<[Oscillator; 4]>>,
    midi_event_arc: &mut Arc<Mutex<Option<MidiNoteEvent>>>,
    filter_arc: &mut Arc<Mutex<Filter>>,
    mixer_arc: &mut Arc<Mutex<Mixer>>,
) {
    log::debug!("process_midi_cc_values(): CC received: {:?}", cc_value);

    match cc_value {
        CC::ModWheel(value) => {
            let mod_wheel_amount = midi_value_to_f32_0_to_1(value);
            let mut parameters = parameters_arc
                .lock()
                .unwrap_or_else(|poisoned| poisoned.into_inner());

            parameters.mod_wheel_amount = mod_wheel_amount;
        }
        CC::Volume(value) => {
            let output_level = exponential_curve_level_adjustment_from_midi_value(value);

            let mut mixer = mixer_arc
                .lock()
                .unwrap_or_else(|poisoned| poisoned.into_inner());

            mixer.set_output_level(output_level);
        }
        CC::ConstantLevel(value) => {
            let is_constant_level = midi_value_to_bool(value);
            let mut mixer = mixer_arc
                .lock()
                .unwrap_or_else(|poisoned| poisoned.into_inner());

            mixer.set_constant_level(is_constant_level)
        }
        CC::Pan(value) => {
            let output_pan = midi_value_to_f32_negative_1_to_1(value);

            let mut mixer = mixer_arc
                .lock()
                .unwrap_or_else(|poisoned| poisoned.into_inner());

            mixer.set_output_pan(output_pan);
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
            //TODO
        }
        CC::Oscillator1Shape(value) => {
            //TODO
        }
        CC::Oscillator2Shape(value) => {
            //TODO
        }
        CC::Oscillator3Shape(value) => {
            //TODO
        }
        CC::SubOscillatorCourseTune(value) => {
            let mut parameters = parameters_arc
                .lock()
                .unwrap_or_else(|poisoned| poisoned.into_inner());

            parameters.oscillators[0].course_tune =
                Some(midi_value_to_course_tune_intervals(value));
            update_current_note_from_stored_parameters(&mut parameters);
        }
        CC::Oscillator1CourseTune(value) => {
            let mut parameters = parameters_arc
                .lock()
                .unwrap_or_else(|poisoned| poisoned.into_inner());

            parameters.oscillators[1].course_tune =
                Some(midi_value_to_course_tune_intervals(value));
            update_current_note_from_stored_parameters(&mut parameters);
        }
        CC::Oscillator2CourseTune(value) => {
            let mut parameters = parameters_arc
                .lock()
                .unwrap_or_else(|poisoned| poisoned.into_inner());

            parameters.oscillators[2].course_tune =
                Some(midi_value_to_course_tune_intervals(value));
            update_current_note_from_stored_parameters(&mut parameters);
        }
        CC::Oscillator3CourseTune(value) => {
            let mut parameters = parameters_arc
                .lock()
                .unwrap_or_else(|poisoned| poisoned.into_inner());

            parameters.oscillators[3].course_tune =
                Some(midi_value_to_course_tune_intervals(value));
            update_current_note_from_stored_parameters(&mut parameters);
        }
        CC::SubOscillatorFineTune(value) => {
            let mut parameters = parameters_arc
                .lock()
                .unwrap_or_else(|poisoned| poisoned.into_inner());

            parameters.oscillators[0].fine_tune = Some(midi_value_to_fine_tune_cents(value));
            update_current_note_from_stored_parameters(&mut parameters);
        }
        CC::Oscillator1FineTune(value) => {
            let mut parameters = parameters_arc
                .lock()
                .unwrap_or_else(|poisoned| poisoned.into_inner());

            parameters.oscillators[1].fine_tune = Some(midi_value_to_fine_tune_cents(value));
            update_current_note_from_stored_parameters(&mut parameters);
        }
        CC::Oscillator2FineTune(value) => {
            let mut parameters = parameters_arc
                .lock()
                .unwrap_or_else(|poisoned| poisoned.into_inner());

            parameters.oscillators[2].fine_tune = Some(midi_value_to_fine_tune_cents(value));
            update_current_note_from_stored_parameters(&mut parameters);
        }
        CC::Oscillator3FineTune(value) => {
            let mut parameters = parameters_arc
                .lock()
                .unwrap_or_else(|poisoned| poisoned.into_inner());

            parameters.oscillators[3].fine_tune = Some(midi_value_to_fine_tune_cents(value));
            update_current_note_from_stored_parameters(&mut parameters);
        }
        CC::SubOscillatorLevel(value) => {
            let level = exponential_curve_level_adjustment_from_midi_value(value);

            let mut mixer = mixer_arc
                .lock()
                .unwrap_or_else(|poisoned| poisoned.into_inner());

            mixer.set_quad_level(level, MixerInput::One);

            let mut parameters = parameters_arc
                .lock()
                .unwrap_or_else(|poisoned| poisoned.into_inner());

            parameters.oscillators[0].level = level;
        }
        CC::Oscillator1Level(value) => {
            let level = exponential_curve_level_adjustment_from_midi_value(value);

            let mut mixer = mixer_arc
                .lock()
                .unwrap_or_else(|poisoned| poisoned.into_inner());

            mixer.set_quad_level(level, MixerInput::Two);

            let mut parameters = parameters_arc
                .lock()
                .unwrap_or_else(|poisoned| poisoned.into_inner());

            parameters.oscillators[1].level = level;
        }
        CC::Oscillator2Level(value) => {
            let level = exponential_curve_level_adjustment_from_midi_value(value);

            let mut mixer = mixer_arc
                .lock()
                .unwrap_or_else(|poisoned| poisoned.into_inner());

            mixer.set_quad_level(level, MixerInput::Three);

            let mut parameters = parameters_arc
                .lock()
                .unwrap_or_else(|poisoned| poisoned.into_inner());

            parameters.oscillators[2].level = level;
        }
        CC::Oscillator3Level(value) => {
            let level = exponential_curve_level_adjustment_from_midi_value(value);

            let mut mixer = mixer_arc
                .lock()
                .unwrap_or_else(|poisoned| poisoned.into_inner());

            mixer.set_quad_level(level, MixerInput::Four);

            let mut parameters = parameters_arc
                .lock()
                .unwrap_or_else(|poisoned| poisoned.into_inner());

            parameters.oscillators[3].level = level;
        }
        CC::SubOscillatorMute(value) => {
            let mute = midi_value_to_bool(value);

            let mut mixer = mixer_arc
                .lock()
                .unwrap_or_else(|poisoned| poisoned.into_inner());

            mixer.set_quad_mute(mute, MixerInput::One);

            let mut parameters = parameters_arc
                .lock()
                .unwrap_or_else(|poisoned| poisoned.into_inner());

            parameters.oscillators[0].mute = mute;
        }
        CC::Oscillator1Mute(value) => {
            let mute = midi_value_to_bool(value);

            let mut mixer = mixer_arc
                .lock()
                .unwrap_or_else(|poisoned| poisoned.into_inner());

            mixer.set_quad_mute(mute, MixerInput::Two);

            let mut parameters = parameters_arc
                .lock()
                .unwrap_or_else(|poisoned| poisoned.into_inner());

            parameters.oscillators[1].mute = mute;
        }
        CC::Oscillator2Mute(value) => {
            let mute = midi_value_to_bool(value);

            let mut mixer = mixer_arc
                .lock()
                .unwrap_or_else(|poisoned| poisoned.into_inner());

            mixer.set_quad_mute(mute, MixerInput::Three);

            let mut parameters = parameters_arc
                .lock()
                .unwrap_or_else(|poisoned| poisoned.into_inner());

            parameters.oscillators[2].mute = mute;
        }
        CC::Oscillator3Mute(value) => {
            let mute = midi_value_to_bool(value);

            let mut mixer = mixer_arc
                .lock()
                .unwrap_or_else(|poisoned| poisoned.into_inner());

            mixer.set_quad_mute(mute, MixerInput::Four);

            let mut parameters = parameters_arc
                .lock()
                .unwrap_or_else(|poisoned| poisoned.into_inner());

            parameters.oscillators[3].mute = mute;
        }
        CC::SubOscillatorPan(value) => {
            let pan = midi_value_to_f32_negative_1_to_1(value);

            let mut mixer = mixer_arc
                .lock()
                .unwrap_or_else(|poisoned| poisoned.into_inner());

            mixer.set_quad_pan(pan, MixerInput::One);

            let mut parameters = parameters_arc
                .lock()
                .unwrap_or_else(|poisoned| poisoned.into_inner());

            parameters.oscillators[0].pan = pan;
        }
        CC::Oscillator1Pan(value) => {
            let pan = midi_value_to_f32_negative_1_to_1(value);

            let mut mixer = mixer_arc
                .lock()
                .unwrap_or_else(|poisoned| poisoned.into_inner());

            mixer.set_quad_pan(pan, MixerInput::Two);

            let mut parameters = parameters_arc
                .lock()
                .unwrap_or_else(|poisoned| poisoned.into_inner());

            parameters.oscillators[1].pan = pan;
        }
        CC::Oscillator2Pan(value) => {
            let pan = midi_value_to_f32_negative_1_to_1(value);

            let mut mixer = mixer_arc
                .lock()
                .unwrap_or_else(|poisoned| poisoned.into_inner());

            mixer.set_quad_pan(pan, MixerInput::Three);

            let mut parameters = parameters_arc
                .lock()
                .unwrap_or_else(|poisoned| poisoned.into_inner());

            parameters.oscillators[2].pan = pan;
        }
        CC::Oscillator3Pan(value) => {
            let pan = midi_value_to_f32_negative_1_to_1(value);

            let mut mixer = mixer_arc
                .lock()
                .unwrap_or_else(|poisoned| poisoned.into_inner());

            mixer.set_quad_pan(pan, MixerInput::Four);

            let mut parameters = parameters_arc
                .lock()
                .unwrap_or_else(|poisoned| poisoned.into_inner());

            parameters.oscillators[3].pan = pan;
        }
        CC::FilterPoleSwitch(value) => {
            let mut filter = filter_arc
                .lock()
                .unwrap_or_else(|poisoned| poisoned.into_inner());

            filter.set_filter_slope(midi_value_to_filter_slope(value));
        }
        CC::FilterResonance(value) => {
            let mut filter = filter_arc
                .lock()
                .unwrap_or_else(|poisoned| poisoned.into_inner());

            filter.set_resonance(midi_value_to_f32_range(value, 0.0, 1.0));
        }
        CC::AmpEGReleaseTime(value) => {
            let time = midi_value_to_f32_range(
                value,
                ENVELOPE_MIN_MILLISECONDS,
                ENVELOPE_MAX_MILLISECONDS,
            );
            let mut envelope = amp_envelope_arc
                .lock()
                .unwrap_or_else(|poisoned| poisoned.into_inner());

            envelope.set_release_milliseconds(time);
        }
        CC::AmpEGAttackTime(value) => {
            let time = midi_value_to_f32_range(
                value,
                ENVELOPE_MIN_MILLISECONDS,
                ENVELOPE_MAX_MILLISECONDS,
            );

            let mut envelope = amp_envelope_arc
                .lock()
                .unwrap_or_else(|poisoned| poisoned.into_inner());

            envelope.set_attack_milliseconds(time);
        }
        CC::FilterCutoff(value) => {
            let mut filter = filter_arc
                .lock()
                .unwrap_or_else(|poisoned| poisoned.into_inner());

            filter.set_cutoff_frequency(exponential_curve_filter_cutoff_from_midi_value(value));
        }
        CC::AmpEGDecayTime(value) => {
            let time = midi_value_to_f32_range(
                value,
                ENVELOPE_MIN_MILLISECONDS,
                ENVELOPE_MAX_MILLISECONDS,
            );

            let mut envelope = amp_envelope_arc
                .lock()
                .unwrap_or_else(|poisoned| poisoned.into_inner());

            envelope.set_decay_milliseconds(time);
        }
        CC::AmpEGSustainLevel(value) => {
            let mut envelope = amp_envelope_arc
                .lock()
                .unwrap_or_else(|poisoned| poisoned.into_inner());

            envelope.set_sustain_level(midi_value_to_f32_0_to_1(value));
        }
        CC::AmpEGInverted(value) => {
            //TODO
        }
        CC::FilterEGAttackTime(value) => {
            //TODO
        }
        CC::FilterEGDecayTime(value) => {
            //TODO
        }
        CC::FilterEGSustainLevel(value) => {
            //TODO
        }
        CC::FilterEGReleaseTime(value) => {
            //TODO
        }
        CC::FilterEGInverted(value) => {
            //TODO
        }
        CC::FilterEGAmount(value) => {
            //TODO
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
            let mut midi_events = midi_event_arc
                .lock()
                .unwrap_or_else(|poisoned| poisoned.into_inner());

            *midi_events = Some(MidiNoteEvent::NoteOff);
        }
    }
}

pub fn midi_value_to_f32_range(midi_value: u8, minimum: f32, maximum: f32) -> f32 {
    let range = maximum - minimum;
    let increment = range / MAX_MIDI_NOTE as f32;
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
    (exponential_coefficient * (midi_value as f32 / MAX_MIDI_NOTE as f32)).exp()
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
        parameters.oscillators[0].course_tune,
        parameters.oscillators[0].fine_tune,
        parameters.oscillators[0].pitch_bend,
    );

    let osc1_frequency = tune(
        parameters.current_note.midi_note,
        parameters.oscillators[1].course_tune,
        parameters.oscillators[1].fine_tune,
        parameters.oscillators[1].pitch_bend,
    );
    let osc2_frequency = tune(
        parameters.current_note.midi_note,
        parameters.oscillators[2].course_tune,
        parameters.oscillators[2].fine_tune,
        parameters.oscillators[2].pitch_bend,
    );
    let osc3_frequency = tune(
        parameters.current_note.midi_note,
        parameters.oscillators[3].course_tune,
        parameters.oscillators[3].fine_tune,
        parameters.oscillators[3].pitch_bend,
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
