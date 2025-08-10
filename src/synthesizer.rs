use crate::midi::{CC, MidiMessage};
use crate::modules::amplifier::vca;
use crate::modules::envelope::{ENVELOPE_MAX_MILLISECONDS, ENVELOPE_MIN_MILLISECONDS, Envelope};
use crate::modules::mixer::{Mixer, MixerInput};
use crate::modules::oscillator::{Oscillator, WaveShape};
use crate::modules::tuner::tune;

use anyhow::Result;
use cpal::traits::DeviceTrait;
use cpal::{Device, Stream};
use crossbeam_channel::Receiver;
use std::sync::{Arc, Mutex, MutexGuard};
use std::thread;

pub const MIDI_VELOCITY_TO_SAMPLE_FACTOR: f32 = 1.0 / 127.0;

pub const DEFAULT_OSCILLATOR_OUTPUT_LEVEL: f32 = 1.0;
pub const DEFAULT_OSCILLATOR_OUTPUT_PAN: f32 = 0.0;
pub const DEFAULT_SUB_OSCILLATOR_INTERVAL: i8 = -12;
pub const DEFAULT_SUB_OSCILLATOR_LEVEL: f32 = 0.0;
pub const DEFAULT_AMP_ENVELOPE_ATTACK_TIME: f32 = 200.0;
pub const DEFAULT_AMP_ENVELOPE_DECAY_TIME: f32 = 200.0;
pub const DEFAULT_AMP_ENVELOPE_SUSTAIN_LEVEL: f32 = 0.8;
pub const DEFAULT_AMP_ENVELOPE_RELEASE_TIME: f32 = 200.0;
pub const DEFAULT_OUTPUT_LEVEL: f32 = 0.5;
pub const DEFAULT_OUTPUT_PAN: f32 = 0.0;

#[derive(Debug, Clone, Copy, PartialEq)]
enum MidiEvent {
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
struct EnvelopeParameters {
    attack_time: f32,
    decay_time: f32,
    sustain_level: f32,
    release_time: f32,
}

#[derive(Default, Debug, Clone, Copy, PartialEq)]
struct CurrentNote {
    midi_note: u8,
    velocity: f32,
}

#[derive(Default, Debug, Clone, Copy, PartialEq)]
struct Parameters {
    output_level: f32,
    output_pan: f32,
    current_note: CurrentNote,
    mixer: Mixer,
    oscillators: [OscillatorParameters; 4],
    amp_envelope: EnvelopeParameters,
}

pub struct Synthesizer {
    audio_output_device: Device,
    output_stream: Option<Stream>,
    parameters: Arc<Mutex<Parameters>>,
    midi_events: Arc<Mutex<Option<MidiEvent>>>,
    oscillators: Arc<Mutex<[Oscillator; 4]>>,
    amp_envelope: Arc<Mutex<Envelope>>,
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

        let parameters = Parameters {
            output_level: DEFAULT_OUTPUT_LEVEL,
            output_pan: DEFAULT_OUTPUT_PAN,
            mixer,
            current_note: Default::default(),
            oscillators: oscillator_parameters,
            amp_envelope: EnvelopeParameters {
                attack_time: DEFAULT_AMP_ENVELOPE_ATTACK_TIME,
                decay_time: DEFAULT_AMP_ENVELOPE_DECAY_TIME,
                sustain_level: DEFAULT_AMP_ENVELOPE_SUSTAIN_LEVEL,
                release_time: DEFAULT_AMP_ENVELOPE_RELEASE_TIME,
            },
        };

        let oscillators = [
            Oscillator::new(sample_rate, WaveShape::Sine),
            Oscillator::new(sample_rate, WaveShape::Sine),
            Oscillator::new(sample_rate, WaveShape::Sine),
            Oscillator::new(sample_rate, WaveShape::Sine),
        ];

        let mut amp_envelope = Envelope::new(sample_rate);
        amp_envelope.set_attack_milliseconds(DEFAULT_AMP_ENVELOPE_ATTACK_TIME);
        amp_envelope.set_decay_milliseconds(DEFAULT_AMP_ENVELOPE_DECAY_TIME);
        amp_envelope.set_sustain_level(DEFAULT_AMP_ENVELOPE_SUSTAIN_LEVEL);
        amp_envelope.set_release_milliseconds(DEFAULT_AMP_ENVELOPE_RELEASE_TIME);

        Self {
            audio_output_device,
            output_stream: None,
            parameters: Arc::new(Mutex::new(parameters)),
            midi_events: Arc::new(Mutex::new(None)),
            oscillators: Arc::new(Mutex::new(oscillators)),
            amp_envelope: Arc::new(Mutex::new(amp_envelope)),
        }
    }

    pub fn run(&mut self, midi_message_receiver: Receiver<MidiMessage>) -> Result<()> {
        log::info!("Creating the synthesizer audio stream");
        self.output_stream = Some(self.create_synthesizer(self.audio_output_device.clone())?);
        log::debug!("run(): The synthesizer audio stream has been created");

        let parameters_arc = self.parameters.clone();
        let midi_event_arc = self.midi_events.clone();
        let envelope_arc = self.amp_envelope.clone();

        log::debug!("run(): Start the midi event listener thread");
        start_midi_event_listener(
            midi_message_receiver,
            parameters_arc,
            midi_event_arc,
            envelope_arc,
        );

        Ok(())
    }

    fn create_synthesizer(&mut self, output_device: Device) -> Result<Stream> {
        let parameters_arc = self.parameters.clone();
        let midi_events_arc = self.midi_events.clone();
        let oscillators_arc = self.oscillators.clone();
        let amp_envelope_arc = self.amp_envelope.clone();

        let default_device_stream_config = output_device.default_output_config()?.config();
        let sample_rate = default_device_stream_config.sample_rate.0;
        let number_of_channels = default_device_stream_config.channels as usize;

        log::debug!("create_synthesizer(): Amp envelope created");

        log::info!(
            "Creating the synthesizer audio output stream for the device {} with sample rate: {}",
            output_device.name().unwrap_or("Unknown".to_string()),
            sample_rate
        );

        let stream = output_device.build_output_stream(
            &default_device_stream_config,
            move |buffer: &mut [f32], _: &cpal::OutputCallbackInfo| {
                let parameters = parameters_arc
                    .lock()
                    .unwrap_or_else(|poisoned| poisoned.into_inner());
                let mut midi_events = midi_events_arc
                    .lock()
                    .unwrap_or_else(|poisoned| poisoned.into_inner());
                let mut oscillators = oscillators_arc
                    .lock()
                    .unwrap_or_else(|poisoned| poisoned.into_inner());
                let mut amp_envelope = amp_envelope_arc
                    .lock()
                    .unwrap_or_else(|poisoned| poisoned.into_inner());

                // Begin processing the audio buffer
                action_midi_events(midi_events.take(), &mut amp_envelope);

                // Split the buffer into frames
                for frame in buffer.chunks_mut(number_of_channels) {
                    // Begin generating and processing the samples for the frame

                    let amp_envelope_value = amp_envelope.generate();

                    let sub_oscillator_raw_sample =
                        oscillators[0].generate(parameters.oscillators[0].frequency, None);
                    let sub_oscillator_sample = vca(
                        sub_oscillator_raw_sample,
                        Some(parameters.current_note.velocity),
                        Some(amp_envelope_value),
                    );

                    let oscillator1_raw_sample =
                        oscillators[1].generate(parameters.oscillators[1].frequency, None);
                    let oscillator1_sample = vca(
                        oscillator1_raw_sample,
                        Some(parameters.current_note.velocity),
                        Some(amp_envelope_value),
                    );

                    let oscillator2_raw_sample =
                        oscillators[2].generate(parameters.oscillators[2].frequency, None);
                    let oscillator2_sample = vca(
                        oscillator2_raw_sample,
                        Some(parameters.current_note.velocity),
                        Some(amp_envelope_value),
                    );

                    let oscillator3_raw_sample =
                        oscillators[3].generate(parameters.oscillators[3].frequency, None);
                    let oscillator3_sample = vca(
                        oscillator3_raw_sample,
                        Some(parameters.current_note.velocity),
                        Some(amp_envelope_value),
                    );

                    let (oscillator_mix_left, oscillator_mix_right) = parameters.mixer.quad_mix(
                        sub_oscillator_sample,
                        oscillator1_sample,
                        oscillator2_sample,
                        oscillator3_sample,
                    );

                    // Final output level control
                    let (output_left, output_right) = parameters.mixer.output_mix(
                        oscillator_mix_left,
                        oscillator_mix_right,
                        parameters.output_level,
                        parameters.output_pan,
                    );

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
    mut midi_event_arc: Arc<Mutex<Option<MidiEvent>>>,
    mut amp_envelope_arc: Arc<Mutex<Envelope>>,
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

                    *midi_events = Some(MidiEvent::NoteOn);
                }
                MidiMessage::NoteOff => {
                    let mut midi_events = midi_event_arc
                        .lock()
                        .unwrap_or_else(|poisoned| poisoned.into_inner());

                    *midi_events = Some(MidiEvent::NoteOff);
                }
                MidiMessage::ControlChange(cc_value) => {
                    process_midi_cc_values(
                        cc_value,
                        &mut parameters_arc,
                        &mut amp_envelope_arc,
                        &mut midi_event_arc,
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
    parameters.current_note.velocity = velocity as f32 * MIDI_VELOCITY_TO_SAMPLE_FACTOR;
    parameters.oscillators[0].frequency = sub_osc_frequency;
    parameters.oscillators[1].frequency = osc1_frequency;
    parameters.oscillators[2].frequency = osc2_frequency;
    parameters.oscillators[3].frequency = osc3_frequency;
}

fn process_midi_cc_values(
    cc_value: CC,
    parameters_arc: &mut Arc<Mutex<Parameters>>,
    amp_envelope_arc: &mut Arc<Mutex<Envelope>>,
    midi_event_arc: &mut Arc<Mutex<Option<MidiEvent>>>,
) {
    log::debug!("process_midi_cc_values(): CC received: {:?}", cc_value);

    let mut parameters = parameters_arc
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    let mut envelope = amp_envelope_arc
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());

    match cc_value {
        CC::Volume(value) => parameters.output_level = midi_value_to_f32_0_to_1(value),
        CC::Pan(value) => parameters.output_pan = midi_value_to_f32_negative_1_to_1(value),
        CC::SubOscillatorLevel(value) => {
            let level = midi_value_to_f32_0_to_1(value);
            parameters.mixer.set_quad_level(level, MixerInput::One);
            parameters.oscillators[0].level = level;
        }
        CC::Oscillator1Level(value) => {
            let level = midi_value_to_f32_0_to_1(value);
            parameters.mixer.set_quad_level(level, MixerInput::Two);
            parameters.oscillators[1].level = level;
        }
        CC::Oscillator2Level(value) => {
            let level = midi_value_to_f32_0_to_1(value);
            parameters.mixer.set_quad_level(level, MixerInput::Three);
            parameters.oscillators[2].level = level;
        }
        CC::Oscillator3Level(value) => {
            let level = midi_value_to_f32_0_to_1(value);
            parameters.mixer.set_quad_level(level, MixerInput::Four);
            parameters.oscillators[3].level = level;
        }
        CC::SubOscillatorMute(value) => {
            let mute = midi_value_to_bool(value);
            parameters.mixer.set_quad_mute(mute, MixerInput::One);
            parameters.oscillators[0].mute = mute;
        }
        CC::Oscillator1Mute(value) => {
            let mute = midi_value_to_bool(value);
            parameters.mixer.set_quad_mute(mute, MixerInput::Two);
            parameters.oscillators[1].mute = mute;
        }
        CC::Oscillator2Mute(value) => {
            let mute = midi_value_to_bool(value);
            parameters.mixer.set_quad_mute(mute, MixerInput::Three);
            parameters.oscillators[2].mute = mute;
        }
        CC::Oscillator3Mute(value) => {
            let mute = midi_value_to_bool(value);
            parameters.mixer.set_quad_mute(mute, MixerInput::Four);
            parameters.oscillators[3].mute = mute;
        }
        CC::SubOscillatorPan(value) => {
            let pan = midi_value_to_f32_negative_1_to_1(value);
            parameters.mixer.set_quad_pan(pan, MixerInput::One);
            parameters.oscillators[0].pan = pan;
        }
        CC::Oscillator1Pan(value) => {
            let pan = midi_value_to_f32_negative_1_to_1(value);
            parameters.mixer.set_quad_pan(pan, MixerInput::Two);
            parameters.oscillators[1].pan = pan;
        }
        CC::Oscillator2Pan(value) => {
            let pan = midi_value_to_f32_negative_1_to_1(value);
            parameters.mixer.set_quad_pan(pan, MixerInput::Three);
            parameters.oscillators[2].pan = pan;
        }
        CC::Oscillator3Pan(value) => {
            let pan = midi_value_to_f32_negative_1_to_1(value);
            parameters.mixer.set_quad_pan(pan, MixerInput::Four);
            parameters.oscillators[3].pan = pan;
        }
        CC::AttackTime(value) => {
            let time = midi_value_to_f32_range(
                value,
                ENVELOPE_MIN_MILLISECONDS,
                ENVELOPE_MAX_MILLISECONDS,
            );
            envelope.set_attack_milliseconds(time);
            parameters.amp_envelope.attack_time = time;
        }
        CC::DecayTime(value) => {
            let time = midi_value_to_f32_range(
                value,
                ENVELOPE_MIN_MILLISECONDS,
                ENVELOPE_MAX_MILLISECONDS,
            );
            envelope.set_decay_milliseconds(time);
            parameters.amp_envelope.decay_time = time;
        }
        CC::SustainLevel(value) => {
            envelope.set_sustain_level(midi_value_to_f32_0_to_1(value));
            parameters.amp_envelope.sustain_level = midi_value_to_f32_0_to_1(value);
        }
        CC::ReleaseTime(value) => {
            let time = midi_value_to_f32_range(
                value,
                ENVELOPE_MIN_MILLISECONDS,
                ENVELOPE_MAX_MILLISECONDS,
            );
            envelope.set_release_milliseconds(time);
            parameters.amp_envelope.release_time = time;
        }
        CC::AllNotesOff => {
            let mut midi_events = midi_event_arc
                .lock()
                .unwrap_or_else(|poisoned| poisoned.into_inner());

            *midi_events = Some(MidiEvent::NoteOff);
        }
    }
}

fn action_midi_events(midi_events: Option<MidiEvent>, amp_envelope: &mut MutexGuard<Envelope>) {
    match midi_events {
        Some(MidiEvent::NoteOn) => {
            amp_envelope.gate_on();
        }
        Some(MidiEvent::NoteOff) => {
            amp_envelope.gate_off();
        }
        None => {}
    }
}

fn midi_value_to_f32_range(midi_value: u8, minimum: f32, maximum: f32) -> f32 {
    let range = maximum - minimum;
    let increment = range / 127.0;
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
