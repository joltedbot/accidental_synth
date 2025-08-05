use crate::audio::default_audio_output_device;
use crate::midi::MidiMessage;
use crate::synthesizer::sine::Sine;
use anyhow::Result;
use cpal::traits::DeviceTrait;
use cpal::{Device, Stream};
use crossbeam_channel::Receiver;
use std::thread;

mod sine;

pub struct Synthesizer {
    output_stream: Option<Stream>,
}

impl Synthesizer {
    pub fn new() -> Self {
        log::info!("Constructing Synthesizer Module");

        Self {
            output_stream: None,
        }
    }

    pub fn run(&mut self, midi_message_receiver: Receiver<MidiMessage>) -> Result<()> {
        log::info!("Creating synthesizer audio stream");
        let output_audio_device = default_audio_output_device()?;
        let stream = self.create_synthesizer(output_audio_device)?;
        self.output_stream = Some(stream);
        log::debug!("run(): The synthesizer audio stream has been created");

        thread::spawn(move || {
            log::debug!("run(): spawned thread to receive MIDI events");

            while let Ok(event) = midi_message_receiver.recv() {
                // TODO
                log::debug!("run(): Received event: {event:?}");
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
        let tone_frequency = 440.0;

        log::info!(
            "Creating the synthesizer audio output stream for device {} with sample rate: {}",
            output_device.name().unwrap_or("Unknown".to_string()),
            sample_rate
        );

        let stream = output_device.build_output_stream(
            &default_device_stream_config,
            move |buffer: &mut [f32], _: &cpal::OutputCallbackInfo| {
                for frame in buffer.chunks_mut(number_of_channels) {
                    let sine_sample = sine_wave_generator.next_sample(tone_frequency, None);
                    frame[0] = sine_sample;
                    frame[1] = sine_sample;
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
