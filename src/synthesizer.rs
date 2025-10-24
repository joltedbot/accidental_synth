mod constants;
mod create_synthesizer;
mod midi_messages;
mod midi_value_converters;
mod set_parameters;

use self::constants::{
    DEFAULT_FILTER_ENVELOPE_AMOUNT, DEFAULT_OUTPUT_BALANCE, DEFAULT_OUTPUT_LEVEL,
    DEFAULT_VELOCITY_CURVE, DEFAULT_VIBRATO_LFO_CENTER_FREQUENCY,
    DEFAULT_VIBRATO_LFO_DEPTH, DEFAULT_VIBRATO_LFO_RATE, MAX_MIDI_KEY_VELOCITY,
    QUAD_MIX_DEFAULT_BALANCE, QUAD_MIX_DEFAULT_INPUT_LEVEL, QUAD_MIX_DEFAULT_SUB_INPUT_LEVEL,
};

use crate::math::{load_f32_from_atomic_u32, store_f32_as_atomic_u32};
use crate::midi::Event;
use crate::modules::envelope::EnvelopeParameters;
use crate::modules::filter::{DEFAULT_KEY_TRACKING_AMOUNT, FilterParameters};
use crate::modules::lfo::LfoParameters;
use crate::modules::mixer::MixerInput;
use crate::modules::oscillator::OscillatorParameters;
use crate::synthesizer::create_synthesizer::create_synthesizer;
use crate::synthesizer::midi_messages::{
    process_midi_cc_values, process_midi_channel_pressure_message, process_midi_note_off_message,
    process_midi_note_on_message, process_midi_pitch_bend_message,
};
use anyhow::Result;
use crossbeam_channel::Receiver;
use rtrb::Producer;
use std::default::Default;
use std::sync::Arc;
use std::sync::atomic::Ordering::Relaxed;
use std::sync::atomic::{AtomicBool, AtomicU8, AtomicU32};
use std::thread;

#[derive(Debug, Clone, Copy)]
enum MidiNoteEvent {
    NoteOn = 1,
    NoteOff = 2,
}

#[derive(Default, Debug, Clone, Copy)]
enum MidiGateEvent {
    #[default]
    Wait = 0,
    GateOn = 1,
    GateOff = 2,
}

#[derive(Debug, Clone, Copy)]
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
    velocity_curve: AtomicU32,
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

        let velocity_curve =  AtomicU32::new(0);
        store_f32_as_atomic_u32(&velocity_curve, DEFAULT_VELOCITY_CURVE);
        let current_note = CurrentNote {
            midi_note: AtomicU8::new(60),
            velocity,
            velocity_curve,
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
        sample_buffer_receiver: Receiver<Producer<f32>>,
    ) -> Result<()> {
        log::debug!("run(): Start the midi event listener thread");
        self.start_midi_event_listener(midi_message_receiver);

        log::info!("Creating the synthesizer audio stream");
        create_synthesizer(
            sample_buffer_receiver,
            self.sample_rate,
            &self.current_note,
            &self.module_parameters,
        )?;

        Ok(())
    }

    fn start_midi_event_listener(&mut self, midi_message_receiver: Receiver<Event>) {
        let mut current_note = self.current_note.clone();
        let mut module_parameters = self.module_parameters.clone();

        thread::spawn(move || {
            log::debug!("start_midi_event_listener(): spawned thread to receive MIDI events");

            while let Ok(event) = midi_message_receiver.recv() {
                match event {
                    Event::NoteOn(midi_note, velocity) => {
                        process_midi_note_on_message(
                            &mut module_parameters,
                            &mut current_note,
                            midi_note,
                            velocity,
                        );
                    }
                    Event::NoteOff => {
                        process_midi_note_off_message(&mut module_parameters);
                    }
                    Event::PitchBend(bend_amount) => {
                        process_midi_pitch_bend_message(
                            &module_parameters.oscillators,
                            module_parameters.keyboard.pitch_bend_range.load(Relaxed),
                            bend_amount,
                        );
                    }
                    Event::ChannelPressure(pressure_value) => {
                        process_midi_channel_pressure_message(
                            &module_parameters.keyboard,
                            pressure_value,
                        );
                    }
                    Event::ControlChange(cc_value) => {
                        process_midi_cc_values(cc_value, &mut current_note, &mut module_parameters);
                    }
                }
            }

            log::debug!("run(): MIDI event receiver thread has exited");
        });
    }
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
