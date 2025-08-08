mod constants;

use crate::midi::MidiMessage;
use crate::modules::amplifier::controllable_amplifier;
use crate::modules::envelope::Envelope;
use crate::modules::mixer::{Mixer, MixerInput};
use crate::modules::oscillator::{Oscillator, WaveShape};
use crate::synthesizer::constants::*;

use crate::modules::mixer;
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
    current_note: (f32, f32),
}

pub struct Synthesizer {
    sample_rate: u32,
    audio_output_device: Device,
    output_stream: Option<Stream>,
    parameters: Arc<Mutex<Parameters>>,
    midi_events: Arc<Mutex<Option<MidiEvent>>>,
    oscillators: Arc<Mutex<Oscillators>>,
    amp_envelope: Arc<Mutex<Envelope>>,
    oscillator_mixer: Arc<Mutex<Mixer>>,
}

impl Synthesizer {
    pub fn new(audio_output_device: Device, sample_rate: u32) -> Self {
        log::info!("Constructing Synthesizer Module");
        let parameters = Parameters::default();
        let oscillators = Oscillators {
            one: Oscillator::new(sample_rate, WaveShape::Sine),
            two: Oscillator::new(sample_rate, WaveShape::Sine),
            three: Oscillator::new(sample_rate, WaveShape::Sine),
            four: Oscillator::new(sample_rate, WaveShape::Sine),
        };

        Self {
            sample_rate,
            audio_output_device,
            output_stream: None,
            parameters: Arc::new(Mutex::new(parameters)),
            midi_events: Arc::new(Mutex::new(None)),
            oscillators: Arc::new(Mutex::new(oscillators)),
            amp_envelope: Arc::new(Mutex::new(Envelope::new(sample_rate))),
            oscillator_mixer: Arc::new(Mutex::new(Mixer::new())),
        }
    }

    pub fn run(&mut self, midi_message_receiver: Receiver<MidiMessage>) -> Result<()> {
        log::info!("Creating the synthesizer audio stream");
        self.output_stream = Some(self.create_synthesizer(self.audio_output_device.clone())?);
        log::debug!("run(): The synthesizer audio stream has been created");

        let parameters_arc = self.parameters.clone();
        let midi_event_arc = self.midi_events.clone();

        log::debug!("run(): Start the midi event listener thread");
        start_midi_event_listener(midi_message_receiver, parameters_arc, midi_event_arc);

        Ok(())
    }

    fn create_synthesizer(&mut self, output_device: Device) -> Result<Stream> {
        let parameters_arc = self.parameters.clone();
        let midi_events_arc = self.midi_events.clone();
        let oscillators_arc = self.oscillators.clone();
        let oscillator_mixer_arc = self.oscillator_mixer.clone();
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
                let mut oscillator_mixer = oscillator_mixer_arc
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

                    let oscillator_sample = oscillators.one.next_sample(note_frequency, None);
                    let velocity_sample =
                        controllable_amplifier(oscillator_sample, None, Some(velocity));
                    let oscillator1_adsr_sample =
                        controllable_amplifier(velocity_sample, None, Some(amp_envelope.next()));

                    let oscillator_sample = oscillators.two.next_sample(note_frequency, None);
                    let velocity_sample =
                        controllable_amplifier(oscillator_sample, None, Some(velocity));
                    let oscillator2_adsr_sample =
                        controllable_amplifier(velocity_sample, None, Some(amp_envelope.next()));

                    let oscillator_sample = oscillators.three.next_sample(note_frequency, None);
                    let velocity_sample =
                        controllable_amplifier(oscillator_sample, None, Some(velocity));
                    let oscillator3_adsr_sample =
                        controllable_amplifier(velocity_sample, None, Some(amp_envelope.next()));

                    let oscillator_sample = oscillators.four.next_sample(note_frequency, None);
                    let velocity_sample =
                        controllable_amplifier(oscillator_sample, None, Some(velocity));
                    let oscillator4_adsr_sample =
                        controllable_amplifier(velocity_sample, None, Some(amp_envelope.next()));

                    let (output_left, output_right) = oscillator_mixer.mix(
                        oscillator1_adsr_sample,
                        oscillator2_adsr_sample,
                        oscillator3_adsr_sample,
                        oscillator4_adsr_sample,
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
    parameters_arc: Arc<Mutex<Parameters>>,
    midi_event_arc: Arc<Mutex<Option<MidiEvent>>>,
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
                    let parameters = parameters_arc
                        .lock()
                        .unwrap_or_else(|poisoned| poisoned.into_inner());

                    let mut midi_events = midi_event_arc
                        .lock()
                        .unwrap_or_else(|poisoned| poisoned.into_inner());

                    *midi_events = Some(MidiEvent::NoteOff);
                }
                MidiMessage::ControlChange(control_number, value) => {
                    //TODO
                }
            }
        }

        log::debug!("run(): MIDI event receiver thread has exited");
    });
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
