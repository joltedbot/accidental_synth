mod constants;
mod midi_messages;

use self::constants::*;
use crate::midi::MidiMessage;
use crate::modules::amplifier::amplify_stereo;
use crate::modules::envelope::Envelope;
use crate::modules::filter::Filter;
use crate::modules::mixer::{Mixer, MixerInput};
use crate::modules::oscillator::{Oscillator, WaveShape};
use std::default::Default;

use crate::modules::lfo::Lfo;
use anyhow::Result;
use cpal::traits::DeviceTrait;
use cpal::{Device, Stream};
use crossbeam_channel::Receiver;
use std::sync::{Arc, Mutex};
use std::thread;

#[derive(Debug, Clone, Copy, PartialEq)]
enum MidiNoteEvent {
    NoteOn,
    NoteOff,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum OscillatorIndex {
    Sub = 0,
    One = 1,
    Two = 2,
    Three = 3,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum EnvelopeIndex {
    Amplifier = 0,
    Filter = 1,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum LfoIndex {
    Filter = 0,
    Lfo1 = 1,
}

#[derive(Default, Debug, Clone, Copy, PartialEq)]
pub struct OscillatorParameters {
    frequency: f32,
    pitch_bend: Option<i16>,
    course_tune: Option<i8>,
    fine_tune: Option<i16>,
    is_sub_oscillator: bool,
}

#[derive(Default, Debug, Clone, Copy, PartialEq)]
struct CurrentNote {
    midi_note: u8,
    velocity: f32,
    velocity_curve: u8,
}

#[derive(Default, Debug, Clone, Copy, PartialEq)]
struct Parameters {
    is_fixed_velocity: bool,
    current_note: CurrentNote,
    mod_wheel_amount: f32,
    aftertouch_amount: f32,
    oscillator_key_sync_enabled: bool,
    oscillators: [OscillatorParameters; 4],
}

pub struct Modules {
    envelopes: [Envelope; 2],
    filter: Filter,
    lfos: [Lfo; 2],
    mixer: Mixer,
    oscillators: [Oscillator; 4],
}

pub struct Synthesizer {
    audio_output_device: Device,
    output_stream: Option<Stream>,
    parameters: Arc<Mutex<Parameters>>,
    midi_note_events: Arc<Mutex<Option<MidiNoteEvent>>>,
    modules: Arc<Mutex<Modules>>,
}

impl Synthesizer {
    pub fn new(audio_output_device: Device, sample_rate: u32) -> Self {
        log::info!("Constructing Synthesizer Module");

        let current_note = CurrentNote {
            midi_note: 0,
            velocity: MAX_MIDI_KEY_VELOCITY,
            velocity_curve: DEFAULT_MIDI_KEY_VELOCITY_CURVE_MIDI_VALUE,
        };

        let parameters = Parameters {
            current_note,
            oscillator_key_sync_enabled: true,
            ..Default::default()
        };

        let amp_envelope = Envelope::new(sample_rate);
        let mut filter_envelope = Envelope::new(sample_rate);
        filter_envelope.set_amount(DEFAULT_FILTER_ENVELOPE_AMOUNT);
        let envelopes = [amp_envelope, filter_envelope];

        let mut filter_modulation_lfo = Lfo::new(sample_rate);
        filter_modulation_lfo.set_range(0.0);
        let shared_lfo1 = Lfo::new(sample_rate);
        let lfos = [filter_modulation_lfo, shared_lfo1];

        let mut mixer = Mixer::new();
        mixer.set_quad_level(MixerInput::One(0.0));

        let mut oscillators = [
            Oscillator::new(sample_rate, WaveShape::Saw),
            Oscillator::new(sample_rate, WaveShape::Saw),
            Oscillator::new(sample_rate, WaveShape::Saw),
            Oscillator::new(sample_rate, WaveShape::Saw),
        ];
        oscillators[OscillatorIndex::Sub as usize].set_is_sub_oscillator(true);

        let modules = Modules {
            envelopes,
            filter: Filter::new(sample_rate),
            lfos,
            mixer,
            oscillators,
        };

        Self {
            audio_output_device,
            output_stream: None,
            parameters: Arc::new(Mutex::new(parameters)),
            midi_note_events: Arc::new(Mutex::new(None)),
            modules: Arc::new(Mutex::new(modules)),
        }
    }

    pub fn run(&mut self, midi_message_receiver: Receiver<MidiMessage>) -> Result<()> {
        log::info!("Creating the synthesizer audio stream");
        self.output_stream = Some(self.create_synthesizer(self.audio_output_device.clone())?);
        log::debug!("run(): The synthesizer audio stream has been created");

        log::debug!("run(): Start the midi event listener thread");
        self.start_midi_event_listener(midi_message_receiver);
        log::debug!("run(): The midi event listener thread is running");
        Ok(())
    }

    fn start_midi_event_listener(&mut self, midi_message_receiver: Receiver<MidiMessage>) {
        let mut parameters_arc = self.parameters.clone();
        let mut midi_event_arc = self.midi_note_events.clone();
        let mut modules_arc = self.modules.clone();

        thread::spawn(move || {
            log::debug!("run(): spawned thread to receive MIDI events");

            while let Ok(event) = midi_message_receiver.recv() {
                match event {
                    MidiMessage::NoteOn(midi_note, velocity) => {
                        let mut modules = modules_arc
                            .lock()
                            .unwrap_or_else(|poisoned| poisoned.into_inner());
                        let oscillators = &mut modules.oscillators;

                        midi_messages::process_midi_note_on_message(
                            &mut parameters_arc,
                            &mut midi_event_arc,
                            oscillators,
                            midi_note,
                            velocity,
                        );
                    }
                    MidiMessage::NoteOff => {
                        midi_messages::process_midi_note_off_message(&mut midi_event_arc);
                    }
                    MidiMessage::PitchBend(bend_amount) => {
                        midi_messages::process_midi_pitch_bend_message(
                            &mut modules_arc,
                            &mut parameters_arc,
                            bend_amount,
                        );
                    }
                    MidiMessage::ChannelPressure(pressure_value) => {
                        midi_messages::process_midi_channel_pressure_message(
                            &mut parameters_arc,
                            pressure_value,
                        );
                    }
                    MidiMessage::ControlChange(cc_value) => {
                        midi_messages::process_midi_cc_values(
                            cc_value,
                            &mut parameters_arc,
                            &mut midi_event_arc,
                            &mut modules_arc,
                        );
                    }
                }
            }

            log::debug!("run(): MIDI event receiver thread has exited");
        });
    }

    fn create_synthesizer(&mut self, output_device: Device) -> Result<Stream> {
        let parameters_arc = self.parameters.clone();
        let midi_note_events_arc = self.midi_note_events.clone();
        let modules_arc = self.modules.clone();

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
                let parameters = {
                    let parameter_mutex = parameters_arc
                        .lock()
                        .unwrap_or_else(|poisoned| poisoned.into_inner());
                    parameter_mutex.to_owned()
                };

                let mut modules = modules_arc
                    .lock()
                    .unwrap_or_else(|poisoned| poisoned.into_inner());

                let note_event = {
                    let mut midi_note_events = midi_note_events_arc
                        .lock()
                        .unwrap_or_else(|poisoned| poisoned.into_inner());

                    midi_note_events.take()
                };

                if let Some(event) = note_event {
                    midi_messages::action_midi_note_events(
                        event,
                        &mut modules,
                        parameters.oscillator_key_sync_enabled,
                    );
                }

                // Begin processing the audio buffer

                // Split the buffer into frames
                for frame in buffer.chunks_mut(number_of_channels) {
                    // Begin generating and processing the samples for the frame

                    let sub_oscillator_sample = modules.oscillators[0].generate(None);

                    let oscillator1_sample = modules.oscillators[1].generate(None);

                    let oscillator2_sample = modules.oscillators[2].generate(None);

                    let oscillator3_sample = modules.oscillators[3].generate(None);

                    // Any per-oscillator processing should happen before this stereo mix down
                    let (oscillator_mix_left, oscillator_mix_right) = modules.mixer.quad_mix(
                        sub_oscillator_sample,
                        oscillator1_sample,
                        oscillator2_sample,
                        oscillator3_sample,
                    );

                    let amp_envelope_value =
                        Some(modules.envelopes[EnvelopeIndex::Amplifier as usize].generate());

                    let (left_envelope_sample, right_envelope_sample) = amplify_stereo(
                        oscillator_mix_left,
                        oscillator_mix_right,
                        Some(parameters.current_note.velocity),
                        amp_envelope_value,
                    );

                    let filter_envelope_value =
                        modules.envelopes[EnvelopeIndex::Filter as usize].generate();

                    let filter_lfo_value = modules.lfos[LfoIndex::Filter as usize].generate();
                    modules
                        .filter
                        .modulate_cutoff_frequency(filter_envelope_value + filter_lfo_value);

                    let (filtered_left, filtered_right) = modules
                        .filter
                        .filter(left_envelope_sample, right_envelope_sample);

                    // Final output level control
                    let (output_left, output_right) =
                        modules.mixer.output_mix(filtered_left, filtered_right);

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
