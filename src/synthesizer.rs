#![allow(unused_variables)]
mod constants;
mod midi_messages;

use self::constants::*;
use crate::midi::{CC, MidiMessage};
use crate::modules::amplifier::amplify_stereo;
use crate::modules::envelope::{ENVELOPE_MAX_MILLISECONDS, ENVELOPE_MIN_MILLISECONDS, Envelope};
use crate::modules::filter::Filter;
use crate::modules::mixer::{Mixer, MixerInput};
use crate::modules::oscillator::{Oscillator, WaveShape};
use crate::modules::tuner::tune;

use anyhow::Result;
use cpal::traits::DeviceTrait;
use cpal::{Device, Stream};
use crossbeam_channel::Receiver;
use std::sync::{Arc, Mutex, MutexGuard};
use std::thread;

#[derive(Debug, Clone, Copy, PartialEq)]
enum MidiNoteEvent {
    NoteOn,
    NoteOff,
}

#[derive(Debug, Clone, Copy, PartialEq)]
struct OscillatorParameters {
    level: f32,
    pan: f32,
    mute: bool,
    frequency: f32,
    course_tune: Option<i8>,
    fine_tune: Option<i8>,
}

impl Default for OscillatorParameters {
    fn default() -> Self {
        Self {
            level: DEFAULT_OSCILLATOR_OUTPUT_LEVEL,
            pan: DEFAULT_OSCILLATOR_OUTPUT_PAN,
            mute: false,
            frequency: 0.0,
            course_tune: None,
            fine_tune: None,
        }
    }
}

#[derive(Default, Debug, Clone, Copy, PartialEq)]
struct CurrentNote {
    midi_note: u8,
    velocity: Option<f32>,
}

#[derive(Default, Debug, Clone, Copy, PartialEq)]
struct Parameters {
    is_fixed_velocity: bool,
    filter_envelope_is_enabled: bool,
    current_note: CurrentNote,
    oscillators: [OscillatorParameters; 4],
}

pub struct Synthesizer {
    audio_output_device: Device,
    output_stream: Option<Stream>,
    parameters: Arc<Mutex<Parameters>>,
    midi_note_events: Arc<Mutex<Option<MidiNoteEvent>>>,
    oscillators: Arc<Mutex<[Oscillator; 4]>>,
    amp_envelope: Arc<Mutex<Envelope>>,
    filter_envelope: Arc<Mutex<Envelope>>,
    mixer: Arc<Mutex<Mixer>>,
    filter: Arc<Mutex<Filter>>,
}

impl Synthesizer {
    pub fn new(audio_output_device: Device, sample_rate: u32) -> Self {
        log::info!("Constructing Synthesizer Module");

        let sub_oscillator = OscillatorParameters {
            level: DEFAULT_SUB_OSCILLATOR_LEVEL,
            course_tune: Some(DEFAULT_SUB_OSCILLATOR_INTERVAL),
            ..Default::default()
        };

        let oscillator_parameters = [
            sub_oscillator,
            Default::default(),
            Default::default(),
            Default::default(),
        ];

        let mut mixer = Mixer::new();
        mixer.set_quad_level(DEFAULT_SUB_OSCILLATOR_LEVEL, MixerInput::One);
        mixer.set_output_level(DEFAULT_OUTPUT_LEVEL);
        mixer.set_output_pan(DEFAULT_OUTPUT_PAN);

        let parameters = Parameters {
            is_fixed_velocity: DEFAULT_FIXED_VELOCITY_STATE,
            filter_envelope_is_enabled: DEFAULT_FILTER_ENVELOPE_STATE,
            current_note: Default::default(),
            oscillators: oscillator_parameters,
        };

        let oscillators = [
            Oscillator::new(sample_rate, WaveShape::Square),
            Oscillator::new(sample_rate, WaveShape::Square),
            Oscillator::new(sample_rate, WaveShape::Square),
            Oscillator::new(sample_rate, WaveShape::Square),
        ];

        let mut amp_envelope = Envelope::new(sample_rate);
        amp_envelope.set_attack_milliseconds(DEFAULT_AMP_ENVELOPE_ATTACK_TIME);
        amp_envelope.set_decay_milliseconds(DEFAULT_AMP_ENVELOPE_DECAY_TIME);
        amp_envelope.set_sustain_level(DEFAULT_AMP_ENVELOPE_SUSTAIN_LEVEL);
        amp_envelope.set_release_milliseconds(DEFAULT_AMP_ENVELOPE_RELEASE_TIME);

        let mut filter_envelope = Envelope::new(sample_rate);
        filter_envelope.set_is_inverted(true);
        filter_envelope.set_attack_milliseconds(DEFAULT_AMP_ENVELOPE_ATTACK_TIME);
        filter_envelope.set_decay_milliseconds(DEFAULT_AMP_ENVELOPE_DECAY_TIME);
        filter_envelope.set_sustain_level(DEFAULT_AMP_ENVELOPE_SUSTAIN_LEVEL);
        filter_envelope.set_release_milliseconds(DEFAULT_AMP_ENVELOPE_RELEASE_TIME);

        let mixer = Mixer::new();
        let filter = Filter::new(sample_rate);

        Self {
            audio_output_device,
            output_stream: None,
            parameters: Arc::new(Mutex::new(parameters)),
            midi_note_events: Arc::new(Mutex::new(None)),
            oscillators: Arc::new(Mutex::new(oscillators)),
            amp_envelope: Arc::new(Mutex::new(amp_envelope)),
            filter_envelope: Arc::new(Mutex::new(filter_envelope)),
            mixer: Arc::new(Mutex::new(mixer)),
            filter: Arc::new(Mutex::new(filter)),
        }
    }

    pub fn run(&mut self, midi_message_receiver: Receiver<MidiMessage>) -> Result<()> {
        log::info!("Creating the synthesizer audio stream");
        self.output_stream = Some(self.create_synthesizer(self.audio_output_device.clone())?);
        log::debug!("run(): The synthesizer audio stream has been created");

        let parameters_arc = self.parameters.clone();
        let midi_event_arc = self.midi_note_events.clone();
        let envelope_arc = self.amp_envelope.clone();
        let filter_arc = self.filter.clone();
        let mixer_arc = self.mixer.clone();

        log::debug!("run(): Start the midi event listener thread");
        start_midi_event_listener(
            midi_message_receiver,
            parameters_arc,
            midi_event_arc,
            envelope_arc,
            filter_arc,
            mixer_arc,
        );

        Ok(())
    }

    fn create_synthesizer(&mut self, output_device: Device) -> Result<Stream> {
        let parameters_arc = self.parameters.clone();
        let midi_note_events_arc = self.midi_note_events.clone();
        let oscillators_arc = self.oscillators.clone();
        let amp_envelope_arc = self.amp_envelope.clone();
        let filter_envelope_arc = self.filter_envelope.clone();
        let filter_arc = self.filter.clone();
        let mixer_arc = self.mixer.clone();

        let default_device_stream_config = output_device.default_output_config()?.config();
        let sample_rate = default_device_stream_config.sample_rate.0;
        let number_of_channels = default_device_stream_config.channels as usize;

        log::info!(
            "Creating the synthesizer audio output stream for the device {} with {} channels at sample rate: {}",
            output_device.name().unwrap_or("Unknown".to_string()),
            number_of_channels,
            sample_rate
        );

        let stream = output_device.build_output_stream(
            &default_device_stream_config,
            move |buffer: &mut [f32], _: &cpal::OutputCallbackInfo| {
                let mut oscillators = oscillators_arc
                    .lock()
                    .unwrap_or_else(|poisoned| poisoned.into_inner());
                let mixer = mixer_arc
                    .lock()
                    .unwrap_or_else(|poisoned| poisoned.into_inner());
                let mut filter = filter_arc
                    .lock()
                    .unwrap_or_else(|poisoned| poisoned.into_inner());
                let mut amp_envelope = amp_envelope_arc
                    .lock()
                    .unwrap_or_else(|poisoned| poisoned.into_inner());
                let mut filter_envelope = filter_envelope_arc
                    .lock()
                    .unwrap_or_else(|poisoned| poisoned.into_inner());

                let note_event = {
                    let mut midi_note_events = midi_note_events_arc
                        .lock()
                        .unwrap_or_else(|poisoned| poisoned.into_inner());

                    midi_note_events.take()
                };

                midi_messages::process_midi_note_events(
                    note_event,
                    &mut amp_envelope,
                    &mut filter_envelope,
                );

                let parameters = {
                    let parameter_mutex = parameters_arc
                        .lock()
                        .unwrap_or_else(|poisoned| poisoned.into_inner());
                    parameter_mutex.to_owned()
                };

                // Begin processing the audio buffer

                // Split the buffer into frames
                for frame in buffer.chunks_mut(number_of_channels) {
                    // Begin generating and processing the samples for the frame

                    let sub_oscillator_sample =
                        oscillators[0].generate(parameters.oscillators[0].frequency, None);

                    let oscillator1_sample =
                        oscillators[1].generate(parameters.oscillators[1].frequency, None);

                    let oscillator2_sample =
                        oscillators[2].generate(parameters.oscillators[2].frequency, None);

                    let oscillator3_sample =
                        oscillators[3].generate(parameters.oscillators[3].frequency, None);

                    // Any per-oscillator processing should happen before this stereo mix down
                    let (oscillator_mix_left, oscillator_mix_right) = mixer.quad_mix(
                        sub_oscillator_sample,
                        oscillator1_sample,
                        oscillator2_sample,
                        oscillator3_sample,
                    );

                    let amp_envelope_value = Some(amp_envelope.generate());
                    let (left_envelope_sample, right_envelope_sample) = amplify_stereo(
                        oscillator_mix_left,
                        oscillator_mix_right,
                        parameters.current_note.velocity,
                        amp_envelope_value,
                    );

                    if parameters.filter_envelope_is_enabled {
                        let filter_envelope_value = filter_envelope.generate();
                        filter.modulate_cutoff_frequency(filter_envelope_value);
                    }

                    let (filtered_left, filtered_right) =
                        filter.filter(left_envelope_sample, right_envelope_sample);

                    // Final output level control
                    let (output_left, output_right) =
                        mixer.output_mix(filtered_left, filtered_right);

                    // Hand back the processed samples to the frame to be sent to the audio device
                    frame[0] = output_left;
                    frame[1] = output_right;
                }
            },
            |err| {
                log::error!("create_synthesizer(): Error in audio output stream: {err}");
            },
            None,
        )?;

        log::info!("Synthesizer audio output stream was successfully created.");

        Ok(stream)
    }
}

fn start_midi_event_listener(
    midi_message_receiver: Receiver<MidiMessage>,
    mut parameters_arc: Arc<Mutex<Parameters>>,
    mut midi_event_arc: Arc<Mutex<Option<MidiNoteEvent>>>,
    mut amp_envelope_arc: Arc<Mutex<Envelope>>,
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

                    update_current_note_from_midi_note(midi_note, velocity, &mut parameters);

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
                MidiMessage::ControlChange(cc_value) => {
                    process_midi_cc_values(
                        cc_value,
                        &mut parameters_arc,
                        &mut amp_envelope_arc,
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

fn update_current_note_from_midi_note(
    midi_note: u8,
    velocity: u8,
    parameters: &mut MutexGuard<Parameters>,
) {
    let sub_osc_frequency = tune(
        midi_note,
        parameters.oscillators[0].course_tune,
        parameters.oscillators[0].fine_tune,
    );

    let osc1_frequency = tune(
        midi_note,
        parameters.oscillators[1].course_tune,
        parameters.oscillators[1].fine_tune,
    );
    let osc2_frequency = tune(
        midi_note,
        parameters.oscillators[2].course_tune,
        parameters.oscillators[2].fine_tune,
    );
    let osc3_frequency = tune(
        midi_note,
        parameters.oscillators[3].course_tune,
        parameters.oscillators[3].fine_tune,
    );

    parameters.current_note.midi_note = midi_note;
    parameters.current_note.velocity = match parameters.is_fixed_velocity {
        false => Some(velocity as f32 * MIDI_VELOCITY_TO_SAMPLE_FACTOR),
        true => None,
    };
    parameters.oscillators[0].frequency = sub_osc_frequency;
    parameters.oscillators[1].frequency = osc1_frequency;
    parameters.oscillators[2].frequency = osc2_frequency;
    parameters.oscillators[3].frequency = osc3_frequency;
}

fn process_midi_cc_values(
    cc_value: CC,
    parameters_arc: &mut Arc<Mutex<Parameters>>,
    amp_envelope_arc: &mut Arc<Mutex<Envelope>>,
    midi_event_arc: &mut Arc<Mutex<Option<MidiNoteEvent>>>,
    filter_arc: &mut Arc<Mutex<Filter>>,
    mixer_arc: &mut Arc<Mutex<Mixer>>,
) {
    log::debug!("process_midi_cc_values(): CC received: {:?}", cc_value);

    match cc_value {
        CC::Volume(value) => {
            let output_level = midi_messages::midi_value_to_f32_0_to_1(value);

            let mut mixer = mixer_arc
                .lock()
                .unwrap_or_else(|poisoned| poisoned.into_inner());

            mixer.set_output_level(output_level);
        }
        CC::Pan(value) => {
            let output_pan = midi_messages::midi_value_to_f32_negative_1_to_1(value);

            let mut mixer = mixer_arc
                .lock()
                .unwrap_or_else(|poisoned| poisoned.into_inner());

            mixer.set_output_pan(output_pan);
        }
        CC::SubOscillatorLevel(value) => {
            let level = midi_messages::midi_value_to_f32_0_to_1(value);

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
            let level = midi_messages::midi_value_to_f32_0_to_1(value);

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
            let level = midi_messages::midi_value_to_f32_0_to_1(value);

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
            let level = midi_messages::midi_value_to_f32_0_to_1(value);

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
            let mute = midi_messages::midi_value_to_bool(value);

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
            let mute = midi_messages::midi_value_to_bool(value);

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
            let mute = midi_messages::midi_value_to_bool(value);

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
            let mute = midi_messages::midi_value_to_bool(value);

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
            let pan = midi_messages::midi_value_to_f32_negative_1_to_1(value);

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
            let pan = midi_messages::midi_value_to_f32_negative_1_to_1(value);

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
            let pan = midi_messages::midi_value_to_f32_negative_1_to_1(value);

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
            let pan = midi_messages::midi_value_to_f32_negative_1_to_1(value);

            let mut mixer = mixer_arc
                .lock()
                .unwrap_or_else(|poisoned| poisoned.into_inner());

            mixer.set_quad_pan(pan, MixerInput::Four);

            let mut parameters = parameters_arc
                .lock()
                .unwrap_or_else(|poisoned| poisoned.into_inner());

            parameters.oscillators[3].pan = pan;
        }
        CC::AttackTime(value) => {
            let time = midi_messages::midi_value_to_f32_range(
                value,
                ENVELOPE_MIN_MILLISECONDS,
                ENVELOPE_MAX_MILLISECONDS,
            );

            let mut envelope = amp_envelope_arc
                .lock()
                .unwrap_or_else(|poisoned| poisoned.into_inner());

            envelope.set_attack_milliseconds(time);
        }
        CC::DecayTime(value) => {
            let time = midi_messages::midi_value_to_f32_range(
                value,
                ENVELOPE_MIN_MILLISECONDS,
                ENVELOPE_MAX_MILLISECONDS,
            );

            let mut envelope = amp_envelope_arc
                .lock()
                .unwrap_or_else(|poisoned| poisoned.into_inner());

            envelope.set_decay_milliseconds(time);
        }
        CC::SustainLevel(value) => {
            let mut envelope = amp_envelope_arc
                .lock()
                .unwrap_or_else(|poisoned| poisoned.into_inner());

            envelope.set_sustain_level(midi_messages::midi_value_to_f32_0_to_1(value));
        }
        CC::ReleaseTime(value) => {
            let time = midi_messages::midi_value_to_f32_range(
                value,
                ENVELOPE_MIN_MILLISECONDS,
                ENVELOPE_MAX_MILLISECONDS,
            );
            let mut envelope = amp_envelope_arc
                .lock()
                .unwrap_or_else(|poisoned| poisoned.into_inner());

            envelope.set_release_milliseconds(time);
        }
        CC::FilterResonance(value) => {
            let mut filter = filter_arc
                .lock()
                .unwrap_or_else(|poisoned| poisoned.into_inner());

            filter.set_resonance(midi_messages::midi_value_to_f32_range(value, 0.0, 1.0));
        }
        CC::FilterCutoff(value) => {
            let mut filter = filter_arc
                .lock()
                .unwrap_or_else(|poisoned| poisoned.into_inner());

            filter.set_cutoff_frequency(midi_messages::midi_value_to_filter_cutoff(value));
        }
        CC::FilterPoleSwitch(value) => {
            let mut filter = filter_arc
                .lock()
                .unwrap_or_else(|poisoned| poisoned.into_inner());

            filter.set_filter_slope(midi_messages::midi_value_to_filter_slope(value));
        }
        CC::AllNotesOff => {
            let mut midi_events = midi_event_arc
                .lock()
                .unwrap_or_else(|poisoned| poisoned.into_inner());

            *midi_events = Some(MidiNoteEvent::NoteOff);
        }
    }
}
