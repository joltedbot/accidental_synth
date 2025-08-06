mod constants;
mod sine;

use crate::audio::default_audio_output_device;
use crate::midi::MidiMessage;
use crate::modules::amplifier::*;
use crate::modules::envelope::*;
use crate::synthesizer::constants::*;
use crate::synthesizer::sine::Sine;
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

#[derive(Default, Debug, Clone, Copy, PartialEq)]
struct Parameters {
    current_note: (f32, u8),
}

pub struct Synthesizer {
    output_stream: Option<Stream>,
    parameters: Arc<Mutex<Parameters>>,
    midi_events: Arc<Mutex<Option<MidiEvent>>>,
}

impl Synthesizer {
    pub fn new() -> Self {
        log::info!("Constructing Synthesizer Module");
        let parameters = Parameters::default();

        Self {
            parameters: Arc::new(Mutex::new(parameters)),
            output_stream: None,
            midi_events: Arc::new(Mutex::new(None)),
        }
    }

    pub fn run(&mut self, midi_message_receiver: Receiver<MidiMessage>) -> Result<()> {
        log::info!("Creating the synthesizer audio stream");
        let output_audio_device = default_audio_output_device()?;
        self.output_stream = Some(self.create_synthesizer(output_audio_device)?);
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

        let default_device_stream_config = output_device.default_output_config()?.config();
        let sample_rate = default_device_stream_config.sample_rate.0;
        let number_of_channels = default_device_stream_config.channels as usize;

        let mut envelope = Envelope::new(sample_rate);
        envelope.set_attack_milliseconds(500);
        envelope.set_decay_milliseconds(400);
        envelope.set_sustain_level(0.8);
        envelope.set_release_milliseconds(500);
        let amp_envelope_arc = Arc::new(Mutex::new(envelope));
        log::debug!("create_synthesizer(): Amp envelope created");

        // TODO - Implement a proper oscillator module with the required controls
        let mut sine_wave_generator = Sine::new(sample_rate);

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
                let mut amp_envelope = amp_envelope_arc
                    .lock()
                    .unwrap_or_else(|poisoned| poisoned.into_inner());

                let (note_frequency, velocity) = parameters.current_note;
                action_midi_events(midi_events.take(), &mut amp_envelope);

                for frame in buffer.chunks_mut(number_of_channels) {
                    let sine_sample = sine_wave_generator.next_sample(note_frequency, None);
                    let output_sample =
                        controllable_amplifier(sine_sample, Some(amp_envelope.next()), None);

                    frame[0] = output_sample;
                    frame[1] = output_sample;
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

                    parameters.current_note =
                        (MIDI_NOTE_FREQUENCIES[note_number as usize].0, velocity);

                    let mut midi_events = midi_event_arc
                        .lock()
                        .unwrap_or_else(|poisoned| poisoned.into_inner());

                    *midi_events = Some(MidiEvent::NoteOn);
                }
                MidiMessage::NoteOff => {
                    let mut parameters = parameters_arc
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
            amp_envelope.start();
        }
        Some(MidiEvent::NoteOff) => {
            amp_envelope.stop();
        }
        None => {}
    }
}
