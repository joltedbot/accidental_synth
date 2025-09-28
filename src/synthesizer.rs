mod constants;
mod midi_messages;
mod midi_value_converters;

use self::constants::{
    DEFAULT_FILTER_ENVELOPE_AMOUNT, DEFAULT_OUTPUT_BALANCE, DEFAULT_OUTPUT_LEVEL,
    DEFAULT_VELOCITY_CURVE_MIDI_VALUE, DEFAULT_VIBRATO_LFO_CENTER_FREQUENCY,
    DEFAULT_VIBRATO_LFO_DEPTH, DEFAULT_VIBRATO_LFO_RATE, MAX_MIDI_KEY_VELOCITY,
    QUAD_MIX_DEFAULT_BALANCE, QUAD_MIX_DEFAULT_INPUT_LEVEL, QUAD_MIX_DEFAULT_SUB_INPUT_LEVEL,
};
use crate::audio::OutputDevice;
use crate::math::{load_f32_from_atomic_u32, store_f32_as_atomic_u32};
use crate::midi::Event;
use crate::modules::amplifier::amplify_stereo;
use crate::modules::envelope::{Envelope, EnvelopeParameters};
use crate::modules::filter::{DEFAULT_KEY_TRACKING_AMOUNT, Filter, FilterParameters};
use crate::modules::lfo::{Lfo, LfoParameters};
use crate::modules::mixer::{MixerInput, output_mix, quad_mix};
use crate::modules::oscillator::{HardSyncRole, Oscillator, OscillatorParameters, WaveShape};
use anyhow::Result;
use cpal::Stream;
use cpal::traits::DeviceTrait;
use crossbeam_channel::Receiver;
use std::default::Default;
use std::sync::Arc;
use std::sync::atomic::Ordering::Relaxed;
use std::sync::atomic::{AtomicBool, AtomicU8, AtomicU32};
use std::thread;

#[derive(Debug, Clone, Copy, PartialEq)]
enum MidiNoteEvent {
    NoteOn = 1,
    NoteOff = 2,
}

#[derive(Default, Debug, Clone, Copy, PartialEq)]
enum MidiGateEvent {
    #[default]
    Wait = 0,
    GateOn = 1,
    GateOff = 2,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum OscillatorIndex {
    Sub = 0,
    One = 1,
    Two = 2,
    Three = 3,
}

#[derive(Default, Debug)]
struct CurrentNote {
    midi_note: AtomicU8,
    velocity: AtomicU32,
    velocity_curve: AtomicU8,
}

#[derive(Default, Debug)]
struct KeyboardParameters {
    mod_wheel_amount: AtomicU32,
    aftertouch_amount: AtomicU32,
    pitch_bend_range: AtomicU8,
}
#[derive(Debug)]
struct QuadMixerInput {
    level: AtomicU32,
    balance: AtomicU32,
    mute: AtomicBool,
}

impl Default for QuadMixerInput {
    fn default() -> Self {
        Self {
            level: AtomicU32::new(QUAD_MIX_DEFAULT_INPUT_LEVEL.to_bits()),
            balance: AtomicU32::new(QUAD_MIX_DEFAULT_BALANCE.to_bits()),
            mute: AtomicBool::new(false),
        }
    }
}

#[derive(Default, Debug)]
struct MixerParameters {
    output_level: AtomicU32,
    output_balance: AtomicU32,
    quad_mixer_inputs: [QuadMixerInput; 4],
}

#[derive(Default, Debug)]
pub struct ModuleParameters {
    filter: FilterParameters,
    mixer: MixerParameters,
    amp_envelope: EnvelopeParameters,
    filter_envelope: EnvelopeParameters,
    filter_lfo: LfoParameters,
    lfo1: LfoParameters,
    keyboard: KeyboardParameters,
    oscillators: [OscillatorParameters; 4],
}

pub struct Synthesizer {
    sample_rate: u32,
    current_note: Arc<CurrentNote>,
    module_parameters: Arc<ModuleParameters>,
}

impl Synthesizer {
    pub fn new(sample_rate: u32) -> Self {
        log::info!("Constructing Synthesizer Module");

        let velocity = AtomicU32::new(0);
        store_f32_as_atomic_u32(&velocity, MAX_MIDI_KEY_VELOCITY);

        let current_note = CurrentNote {
            midi_note: AtomicU8::new(60),
            velocity,
            velocity_curve: AtomicU8::new(DEFAULT_VELOCITY_CURVE_MIDI_VALUE),
        };

        let oscillator_mixer_parameters: [QuadMixerInput; 4] = Default::default();
        store_f32_as_atomic_u32(
            &oscillator_mixer_parameters[0].level,
            QUAD_MIX_DEFAULT_SUB_INPUT_LEVEL,
        );

        let mixer_parameters = MixerParameters {
            output_level: AtomicU32::new(DEFAULT_OUTPUT_LEVEL.to_bits()),
            output_balance: AtomicU32::new(DEFAULT_OUTPUT_BALANCE.to_bits()),
            quad_mixer_inputs: oscillator_mixer_parameters,
        };

        let max_filter_frequency = (sample_rate as f32 * 0.35).min(20000.0);
        let filter_parameters = FilterParameters {
            cutoff_frequency: AtomicU32::new(max_filter_frequency.to_bits()),
            resonance: AtomicU32::new(0.0_f32.to_bits()),
            filter_poles: AtomicU8::new(4),
            key_tracking_amount: AtomicU32::new(DEFAULT_KEY_TRACKING_AMOUNT.to_bits()),
            current_note_number: AtomicU8::new(0),
        };

        let filter_envelope_parameters: EnvelopeParameters = EnvelopeParameters::default();
        filter_envelope_parameters
            .amount
            .store(DEFAULT_FILTER_ENVELOPE_AMOUNT.to_bits(), Relaxed);

        let filter_lfo_parameters: LfoParameters = LfoParameters::default();
        store_f32_as_atomic_u32(&filter_lfo_parameters.range, 0.0);

        let lfo1_parameters = LfoParameters::default();
        lfo1_parameters
            .frequency
            .store(DEFAULT_VIBRATO_LFO_RATE.to_bits(), Relaxed);
        lfo1_parameters
            .center_value
            .store(DEFAULT_VIBRATO_LFO_CENTER_FREQUENCY.to_bits(), Relaxed);
        lfo1_parameters
            .range
            .store(DEFAULT_VIBRATO_LFO_DEPTH.to_bits(), Relaxed);

        let keyboard_parameters = KeyboardParameters {
            pitch_bend_range: AtomicU8::new(12),
            ..KeyboardParameters::default()
        };

        let module_parameters = ModuleParameters {
            filter: filter_parameters,
            mixer: mixer_parameters,
            amp_envelope: EnvelopeParameters::default(),
            filter_envelope: filter_envelope_parameters,
            filter_lfo: filter_lfo_parameters,
            lfo1: lfo1_parameters,
            keyboard: keyboard_parameters,
            oscillators: Default::default(),
        };

        Self {
            sample_rate,
            current_note: Arc::new(current_note),
            module_parameters: Arc::new(module_parameters),
        }
    }

    pub fn run(
        &mut self,
        midi_message_receiver: Receiver<Event>,
        output_device_receiver: Receiver<Option<OutputDevice>>,
    ) {
        log::info!("Creating the synthesizer audio stream");
        self.start_audio_device_event_listener(output_device_receiver);
        log::debug!("run(): The synthesizer audio stream has been created");

        log::debug!("run(): Start the midi event listener thread");
        self.start_midi_event_listener(midi_message_receiver);
        log::debug!("run(): The midi event listener thread is running");
    }

    fn start_audio_device_event_listener(
        &mut self,
        output_device_receiver: Receiver<Option<OutputDevice>>,
    ) {
        let device_receiver = output_device_receiver;
        let sample_rate = self.sample_rate;
        let current_note = self.current_note.clone();
        let module_parameters = self.module_parameters.clone();

        thread::spawn(move || {
            log::debug!(
                "start_audio_device_event_listener(): spawned thread to receive audio device update events"
            );

            let mut output_stream = None;

            while let Ok(new_output_device) = device_receiver.recv() {
                output_stream = match new_output_device {
                    None => {
                        if output_stream.is_some() {
                            log::info!(
                                "start_audio_device_event_listener(): Audio output device removed. Stopping the\
                             current audio output stream"
                            );
                            drop(output_stream);
                        }

                        None
                    }
                    Some(output_device) => {
                        let stream = create_synthesizer(
                            &output_device,
                            sample_rate,
                            &current_note,
                            &module_parameters,
                        )
                        .expect(
                            "start_audio_device_event_listener(): failed to create synthesizer",
                        );

                        if output_stream.is_some() {
                            log::info!(
                                "start_audio_device_event_listener(): Audio output device added. Starting the \
                            new audio output stream. {}",
                                output_device.name
                            );
                            drop(output_stream);
                        }

                        Some(stream)
                    }
                }
            }
        });
    }

    fn start_midi_event_listener(&mut self, midi_message_receiver: Receiver<Event>) {
        let mut current_note = self.current_note.clone();
        let mut module_parameters = self.module_parameters.clone();

        thread::spawn(move || {
            log::debug!("start_midi_event_listener(): spawned thread to receive MIDI events");

            while let Ok(event) = midi_message_receiver.recv() {
                match event {
                    Event::NoteOn(midi_note, velocity) => {
                        midi_messages::process_midi_note_on_message(
                            &mut module_parameters,
                            &mut current_note,
                            midi_note,
                            velocity,
                        );
                    }
                    Event::NoteOff => {
                        midi_messages::process_midi_note_off_message(&mut module_parameters);
                    }
                    Event::PitchBend(bend_amount) => {
                        midi_messages::process_midi_pitch_bend_message(
                            &module_parameters.oscillators,
                            module_parameters.keyboard.pitch_bend_range.load(Relaxed),
                            bend_amount,
                        );
                    }
                    Event::ChannelPressure(pressure_value) => {
                        midi_messages::process_midi_channel_pressure_message(
                            &module_parameters.keyboard,
                            pressure_value,
                        );
                    }
                    Event::ControlChange(cc_value) => {
                        midi_messages::process_midi_cc_values(
                            cc_value,
                            &mut current_note,
                            &mut module_parameters,
                        );
                    }
                }
            }

            log::debug!("run(): MIDI event receiver thread has exited");
        });
    }
}

fn create_synthesizer(
    output_device: &OutputDevice,
    sample_rate: u32,
    current_note: &Arc<CurrentNote>,
    module_parameters: &Arc<ModuleParameters>,
) -> Result<Stream> {
    let current_note = current_note.clone();
    let module_parameters = module_parameters.clone();

    log::info!("Initializing the filter module");
    let mut filter = Filter::new(sample_rate);
    let mut amp_envelope = Envelope::new(sample_rate);
    let mut filter_envelope = Envelope::new(sample_rate);
    let mut filter_lfo = Lfo::new(sample_rate);
    let mut lfo1 = Lfo::new(sample_rate);

    let mut oscillators = [
        Oscillator::new(sample_rate, WaveShape::default()),
        Oscillator::new(sample_rate, WaveShape::default()),
        Oscillator::new(sample_rate, WaveShape::default()),
        Oscillator::new(sample_rate, WaveShape::default()),
    ];
    oscillators[OscillatorIndex::Sub as usize].set_is_sub_oscillator(true);

    let oscillator_hard_sync_buffer = Arc::new(AtomicBool::new(false));
    oscillators[OscillatorIndex::One as usize]
        .set_hard_sync_role(HardSyncRole::Source(oscillator_hard_sync_buffer.clone()));
    oscillators[OscillatorIndex::Two as usize]
        .set_hard_sync_role(HardSyncRole::Synced(oscillator_hard_sync_buffer.clone()));

    let default_device_stream_config = output_device.device.default_output_config()?.config();
    let number_of_channels = output_device.channels.total;
    let left_channel_index = output_device.channels.left;
    let right_channel_index = output_device.channels.right;

    log::info!(
        "Creating the synthesizer audio output stream for the device {} with {} channels at sample rate: {}",
        output_device.name,
        number_of_channels,
        sample_rate
    );

    let stream = output_device.device.build_output_stream(
        &default_device_stream_config,
        move |buffer: &mut [f32], _: &cpal::OutputCallbackInfo| {
            // Process the module parameters per buffer
            amp_envelope.set_parameters(&module_parameters.amp_envelope);
            filter_envelope.set_parameters(&module_parameters.filter_envelope);
            filter_lfo.set_parameters(&module_parameters.filter_lfo);
            filter.set_parameters(&module_parameters.filter);
            lfo1.set_parameters(&module_parameters.lfo1);

            for (index, oscillator) in oscillators.iter_mut().enumerate() {
                oscillator.set_parameters(&module_parameters.oscillators[index]);
                oscillator.tune(current_note.midi_note.load(Relaxed));
            }

            // Begin processing the audio buffer
            let mut quad_mixer_inputs: [MixerInput; 4] =
                create_quad_mixer_inputs(&module_parameters);

            let vibrato_amount =
                load_f32_from_atomic_u32(&module_parameters.keyboard.mod_wheel_amount);
            lfo1.set_range(vibrato_amount / 4.0);

            // Split the buffer into frames
            for frame in buffer.chunks_mut(number_of_channels as usize) {
                // Begin generating and processing the samples for the frame
                let vibrato_value = lfo1.generate(None);
                for (index, input) in quad_mixer_inputs.iter_mut().enumerate() {
                    input.sample = oscillators[index].generate(Some(vibrato_value));
                }

                // Any per-oscillator processing should happen before this stereo mix down
                let (oscillator_mix_left, oscillator_mix_right) = quad_mix(quad_mixer_inputs);

                let amp_envelope_value = Some(amp_envelope.generate());

                let (left_envelope_sample, right_envelope_sample) = amplify_stereo(
                    oscillator_mix_left,
                    oscillator_mix_right,
                    Some(load_f32_from_atomic_u32(&current_note.velocity)),
                    amp_envelope_value,
                );

                let filter_envelope_value = filter_envelope.generate();
                let filter_lfo_value = filter_lfo.generate(None);
                let filter_modulation = filter_envelope_value + filter_lfo_value;

                let (filtered_left, filtered_right) = filter.process(
                    left_envelope_sample,
                    right_envelope_sample,
                    Some(filter_modulation),
                );

                // Final output level control
                let output_level = load_f32_from_atomic_u32(&module_parameters.mixer.output_level);
                let output_balance =
                    load_f32_from_atomic_u32(&module_parameters.mixer.output_balance);

                let (output_left, output_right) =
                    output_mix(filtered_left, filtered_right, output_level, output_balance);

                // Hand back the processed samples to the frame to be sent to the audio device
                frame[left_channel_index] = output_left;

                // For mono devices just drop the right sample
                if let Some(index) = right_channel_index {
                    frame[index] = output_right;
                }
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

fn create_quad_mixer_inputs(module_parameters: &Arc<ModuleParameters>) -> [MixerInput; 4] {
    let sub_oscillator_mixer_input = create_mixer_input_from_oscillator_parameters(
        &module_parameters.mixer,
        OscillatorIndex::Sub,
    );
    let oscillator1_mixer_input = create_mixer_input_from_oscillator_parameters(
        &module_parameters.mixer,
        OscillatorIndex::One,
    );
    let oscillator2_mixer_input = create_mixer_input_from_oscillator_parameters(
        &module_parameters.mixer,
        OscillatorIndex::Two,
    );
    let oscillator3_mixer_input = create_mixer_input_from_oscillator_parameters(
        &module_parameters.mixer,
        OscillatorIndex::Three,
    );
    [
        sub_oscillator_mixer_input,
        oscillator1_mixer_input,
        oscillator2_mixer_input,
        oscillator3_mixer_input,
    ]
}

fn create_mixer_input_from_oscillator_parameters(
    parameters: &MixerParameters,
    oscillator: OscillatorIndex,
) -> MixerInput {
    MixerInput {
        sample: 0.0,
        level: load_f32_from_atomic_u32(&parameters.quad_mixer_inputs[oscillator as usize].level),
        balance: load_f32_from_atomic_u32(
            &parameters.quad_mixer_inputs[oscillator as usize].balance,
        ),
        mute: parameters.quad_mixer_inputs[oscillator as usize]
            .mute
            .load(Relaxed),
    }
}
