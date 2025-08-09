mod constants;

use crate::midi::{CC, MidiMessage};
use crate::modules::amplifier::vca;
use crate::modules::envelope::{ENVELOPE_MAX_MILLISECONDS, ENVELOPE_MIN_MILLISECONDS, Envelope};
use crate::modules::mixer::{Mixer, MixerInput};
use crate::modules::oscillator::{Oscillator, WaveShape};
use crate::synthesizer::constants::*;

use anyhow::Result;
use cpal::traits::DeviceTrait;
use cpal::{Device, Stream};
use crossbeam_channel::Receiver;
use std::sync::{Arc, Mutex, MutexGuard};
use std::thread;

#[derive(Debug, Clone, Copy, PartialEq)]
enum MidiEvent {
    NoteOn,
    NoteOff,
}

struct Oscillators {
    one: Oscillator,
    two: Oscillator,
    three: Oscillator,
    four: Oscillator,
}

#[derive(Default, Debug, Clone, Copy, PartialEq)]
struct Parameters {
    output_level: f32,
    output_pan: f32,
    current_note: (f32, f32),
    mixer: Mixer,
}

pub struct Synthesizer {
    audio_output_device: Device,
    output_stream: Option<Stream>,
    parameters: Arc<Mutex<Parameters>>,
    midi_events: Arc<Mutex<Option<MidiEvent>>>,
    oscillators: Arc<Mutex<Oscillators>>,
    amp_envelope: Arc<Mutex<Envelope>>,
}

impl Synthesizer {
    pub fn new(audio_output_device: Device, sample_rate: u32) -> Self {
        log::info!("Constructing Synthesizer Module");
        let parameters = Parameters {
            output_level: DEFAULT_OUTPUT_LEVEL,
            ..Default::default()
        };

        let oscillators = Oscillators {
            one: Oscillator::new(sample_rate, WaveShape::Sine),
            two: Oscillator::new(sample_rate, WaveShape::Triangle),
            three: Oscillator::new(sample_rate, WaveShape::Saw),
            four: Oscillator::new(sample_rate, WaveShape::Pulse),
        };

        Self {
            audio_output_device,
            output_stream: None,
            parameters: Arc::new(Mutex::new(parameters)),
            midi_events: Arc::new(Mutex::new(None)),
            oscillators: Arc::new(Mutex::new(oscillators)),
            amp_envelope: Arc::new(Mutex::new(Envelope::new(sample_rate))),
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
                let (note_frequency, velocity) = parameters.current_note;
                action_midi_events(midi_events.take(), &mut amp_envelope);

                // Split the buffer into frames
                for frame in buffer.chunks_mut(number_of_channels) {
                    // Begin generating and processing the samples in the frame

                    let osc1_sample = oscillators.one.generate(note_frequency, None);
                    let osc1_velocity_sample = vca(osc1_sample, None, Some(velocity));
                    let osc1_amp_envelope_sample =
                        vca(osc1_velocity_sample, None, Some(amp_envelope.generate()));

                    let osc2_sample = oscillators.two.generate(note_frequency, None);
                    let osc2_velocity_sample = vca(osc2_sample, None, Some(velocity));
                    let osc2_amp_envelope_sample =
                        vca(osc2_velocity_sample, None, Some(amp_envelope.generate()));

                    let osc3_sample = oscillators.three.generate(note_frequency, None);
                    let osc3_velocity_sample = vca(osc3_sample, None, Some(velocity));
                    let osc3_adsr_sample =
                        vca(osc3_velocity_sample, None, Some(amp_envelope.generate()));

                    let osc4_sample = oscillators.four.generate(note_frequency, None);
                    let osc4_velocity_sample = vca(osc4_sample, None, Some(velocity));
                    let osc4_amp_envelope_sample =
                        vca(osc4_velocity_sample, None, Some(amp_envelope.generate()));

                    let (oscillator_mix_left, oscillator_mix_right) = parameters.mixer.quad_mix(
                        osc1_amp_envelope_sample,
                        osc2_amp_envelope_sample,
                        osc3_adsr_sample,
                        osc4_amp_envelope_sample,
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
                MidiMessage::NoteOn(note_number, velocity) => {
                    let mut parameters = parameters_arc
                        .lock()
                        .unwrap_or_else(|poisoned| poisoned.into_inner());

                    parameters.current_note = (
                        MIDI_NOTE_FREQUENCIES[note_number as usize].0,
                        velocity as f32 * MIDI_VELOCITY_TO_SAMPLE_FACTOR,
                    );

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
        CC::Oscillator1Level(value) => parameters
            .mixer
            .set_quad_level(midi_value_to_f32_0_to_1(value), MixerInput::One),
        CC::Oscillator2Level(value) => parameters
            .mixer
            .set_quad_level(midi_value_to_f32_0_to_1(value), MixerInput::Two),
        CC::Oscillator3Level(value) => parameters
            .mixer
            .set_quad_level(midi_value_to_f32_0_to_1(value), MixerInput::Three),
        CC::Oscillator4Level(value) => parameters
            .mixer
            .set_quad_level(midi_value_to_f32_0_to_1(value), MixerInput::Four),
        CC::Oscillator1Mute(value) => parameters
            .mixer
            .set_quad_mute(midi_value_to_bool(value), MixerInput::One),
        CC::Oscillator2Mute(value) => parameters
            .mixer
            .set_quad_mute(midi_value_to_bool(value), MixerInput::Two),
        CC::Oscillator3Mute(value) => parameters
            .mixer
            .set_quad_mute(midi_value_to_bool(value), MixerInput::Three),
        CC::Oscillator4Mute(value) => parameters
            .mixer
            .set_quad_mute(midi_value_to_bool(value), MixerInput::Four),
        CC::Oscillator1Pan(value) => parameters
            .mixer
            .set_quad_pan(midi_value_to_f32_negative_1_to_1(value), MixerInput::One),
        CC::Oscillator2Pan(value) => parameters
            .mixer
            .set_quad_pan(midi_value_to_f32_negative_1_to_1(value), MixerInput::Two),
        CC::Oscillator3Pan(value) => parameters
            .mixer
            .set_quad_pan(midi_value_to_f32_negative_1_to_1(value), MixerInput::Three),
        CC::Oscillator4Pan(value) => parameters
            .mixer
            .set_quad_pan(midi_value_to_f32_negative_1_to_1(value), MixerInput::Four),
        CC::AttackTime(value) => envelope.set_attack_milliseconds(midi_value_to_f32_range(
            value,
            ENVELOPE_MIN_MILLISECONDS,
            ENVELOPE_MAX_MILLISECONDS,
        )),
        CC::DecayTime(value) => envelope.set_decay_milliseconds(midi_value_to_f32_range(
            value,
            ENVELOPE_MIN_MILLISECONDS,
            ENVELOPE_MAX_MILLISECONDS,
        )),
        CC::SustainLevel(value) => envelope.set_sustain_level(midi_value_to_f32_0_to_1(value)),
        CC::ReleaseTime(value) => envelope.set_release_milliseconds(midi_value_to_f32_range(
            value,
            ENVELOPE_MIN_MILLISECONDS,
            ENVELOPE_MAX_MILLISECONDS,
        )),
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
