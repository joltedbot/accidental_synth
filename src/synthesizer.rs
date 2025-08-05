mod constants;
mod sine;

use crate::audio::default_audio_output_device;
use crate::midi::MidiMessage;
use crate::synthesizer::constants::*;
use crate::synthesizer::sine::Sine;
use anyhow::Result;
use cpal::traits::DeviceTrait;
use cpal::{Device, Stream};
use crossbeam_channel::Receiver;
use std::sync::{Arc, Mutex};
use std::thread;

#[derive(Default, Clone, Debug, PartialEq)]
pub enum MidiState {
    NoteOn(f32, u8),
    NoteHold(f32, u8),
    NoteOff,
    #[default]
    Rest,
}

#[derive(Default)]
struct Parameters {
    midi_state: MidiState,
}

pub struct Synthesizer {
    output_stream: Option<Stream>,
    parameters: Arc<Mutex<Parameters>>,
}

impl Synthesizer {
    pub fn new() -> Self {
        log::info!("Constructing Synthesizer Module");

        let parameters = Parameters::default();

        Self {
            parameters: Arc::new(Mutex::new(parameters)),
            output_stream: None,
        }
    }

    pub fn run(&mut self, midi_message_receiver: Receiver<MidiMessage>) -> Result<()> {
        log::info!("Creating synthesizer audio stream");
        let output_audio_device = default_audio_output_device()?;
        let stream = self.create_synthesizer(output_audio_device)?;
        self.output_stream = Some(stream);
        log::debug!("run(): The synthesizer audio stream has been created");

        let parameters_arc = self.parameters.clone();

        thread::spawn(move || {
            log::debug!("run(): spawned thread to receive MIDI events");

            while let Ok(event) = midi_message_receiver.recv() {
                match event {
                    MidiMessage::NoteOn(note_number, velocity) => {
                        let mut parameters = parameters_arc
                            .lock()
                            .unwrap_or_else(|poisoned| poisoned.into_inner());

                        parameters.midi_state = MidiState::NoteOn(
                            MIDI_NOTE_FREQUENCIES[note_number as usize].0,
                            velocity,
                        );
                    }
                    MidiMessage::NoteOff => {
                        let mut parameters = parameters_arc
                            .lock()
                            .unwrap_or_else(|poisoned| poisoned.into_inner());

                        parameters.midi_state = MidiState::NoteOff;
                    }
                    MidiMessage::ControlChange(control_number, value) => {
                        //TODO
                    }
                }
            }

            log::debug!("run(): MIDI event receiver thread has exited");
        });

        Ok(())
    }

    fn create_synthesizer(&mut self, output_device: Device) -> Result<Stream> {
        let default_device_stream_config = output_device.default_output_config()?.config();
        let sample_rate = default_device_stream_config.sample_rate.0;
        let number_of_channels = default_device_stream_config.channels as usize;

        // Temporary testing parameters
        let mut sine_wave_generator = Sine::new(sample_rate);

        let parameters_arc = self.parameters.clone();

        log::info!(
            "Creating the synthesizer audio output stream for device {} with sample rate: {}",
            output_device.name().unwrap_or("Unknown".to_string()),
            sample_rate
        );

        let stream = output_device.build_output_stream(
            &default_device_stream_config,
            move |buffer: &mut [f32], _: &cpal::OutputCallbackInfo| {
                let mut parameters = parameters_arc
                    .lock()
                    .unwrap_or_else(|poisoned| poisoned.into_inner());

                let (note_frequency, note_velocity) =
                    get_note_data_from_midi_state(&mut parameters.midi_state);
                let velocity_level_adjustment = note_velocity as f32 / 127.0;

                for frame in buffer.chunks_mut(number_of_channels) {
                    let sine_sample = sine_wave_generator.next_sample(note_frequency, None);

                    frame[0] = sine_sample * velocity_level_adjustment;
                    frame[1] = sine_sample * velocity_level_adjustment;
                }
            },
            |err| {
                log::error!("create_synthesizer(): Error in audio output stream: {err}");
            },
            None,
        )?;

        Ok(stream)
    }
}

fn get_note_data_from_midi_state(midi_state: &mut MidiState) -> (f32, u8) {
    match *midi_state {
        MidiState::NoteOn(note_frequency, velocity) => {
            *midi_state = MidiState::NoteHold(note_frequency, velocity);
            (note_frequency, velocity)
        }
        MidiState::NoteHold(note_frequency, velocity) => (note_frequency, velocity),
        MidiState::NoteOff => {
            *midi_state = MidiState::Rest;
            (NOTE_REST_FREQUENCY, NOTE_REST_VELOCITY)
        }
        MidiState::Rest => (NOTE_REST_FREQUENCY, NOTE_REST_VELOCITY),
    }
}
