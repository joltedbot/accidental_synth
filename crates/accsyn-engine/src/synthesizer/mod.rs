mod constants;
mod event_listener;
mod midi_messages;
/// Functions for converting normalized MIDI values to synthesizer parameter ranges.
pub mod midi_value_converters;
/// Patch and preset file management for saving and loading synthesizer state.
pub mod patches;
mod sample_generator;
mod set_parameters;

use self::constants::MAX_MIDI_KEY_VELOCITY;

use accsyn_types::audio_events::OutputStreamParameters;
use accsyn_types::defaults::Defaults;
use accsyn_types::midi_events::MidiEvent;
use accsyn_types::synth_events::{OscillatorIndex, SynthesizerUpdateEvents};
use accsyn_types::ui_events::UIUpdates;

use crate::modules::effects::AudioEffectParameters;
use crate::modules::envelope::EnvelopeParameters;
use crate::modules::filter::FilterParameters;
use crate::modules::lfo::LfoParameters;
use crate::modules::mixer::MixerInput;
use crate::modules::oscillator::OscillatorParameters;
use crate::synthesizer::constants::SYNTHESIZER_MESSAGE_SENDER_CAPACITY;
use crate::synthesizer::event_listener::start_update_event_listener;
use crate::synthesizer::midi_messages::{
    process_midi_cc_values, process_midi_channel_pressure_message, process_midi_note_off_message,
    process_midi_note_on_message, process_midi_pitch_bend_message,
};
use crate::synthesizer::sample_generator::sample_generator;

use crate::synthesizer::patches::Patches;
use accsyn_types::parameter_types::{Balance, NormalizedValue};
use anyhow::Result;
use crossbeam_channel::{Receiver, Sender};
use rtrb::Producer;
use serde::{Deserialize, Serialize};
use std::default::Default;
use std::sync::atomic::Ordering::Relaxed;
use std::sync::atomic::{AtomicBool, AtomicU8, AtomicU32};
use std::sync::{Arc, Mutex};
use std::thread;
use strum::EnumCount;

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

#[derive(Debug)]
struct CurrentNote {
    midi_note: AtomicU8,
    velocity: AtomicU32,
}

impl Default for CurrentNote {
    fn default() -> Self {
        Self {
            midi_note: AtomicU8::new(0),
            velocity: AtomicU32::new(MAX_MIDI_KEY_VELOCITY.to_bits()),
        }
    }
}

/// Parameters controlled by the MIDI keyboard input.
#[derive(Default, Debug, Serialize, Deserialize)]
pub struct KeyboardParameters {
    mod_wheel_amount: NormalizedValue,
    aftertouch_amount: NormalizedValue,
    /// Exponent applied to incoming velocity values.
    pub velocity_curve: NormalizedValue,
    /// Whether the output signal polarity is inverted.
    pub polarity_flipped: AtomicBool,
    /// Maximum pitch bend range in semitones.
    pub pitch_bend_range: AtomicU8,
}

impl KeyboardParameters {
    /// Replace all the values in this `KeyboardParameters` with the values from the provided `KeyboardParameters`.
    pub fn assign_from(&self, parameters: &KeyboardParameters) {
        self.mod_wheel_amount
            .store(parameters.mod_wheel_amount.load());
        self.aftertouch_amount
            .store(parameters.aftertouch_amount.load());
        self.velocity_curve.store(parameters.velocity_curve.load());
        self.polarity_flipped
            .store(parameters.polarity_flipped.load(Relaxed), Relaxed);
        self.pitch_bend_range
            .store(parameters.pitch_bend_range.load(Relaxed), Relaxed);
    }
}

/// Level, balance, and mute state for a single oscillator in the quad mixer.
#[derive(Debug, Serialize, Deserialize)]
pub struct QuadMixerInput {
    /// Output level for this mixer input.
    pub level: NormalizedValue,
    /// Stereo balance position for this mixer input.
    pub balance: Balance,
    /// Whether this mixer input is muted.
    pub mute: AtomicBool,
}

impl QuadMixerInput {
    /// Replace all the values in this QuadMixerInput with the values from the provided QuadMixerInput.
    pub fn assign_from(&self, parameters: &QuadMixerInput) {
        self.level.store(parameters.level.load());
        self.balance.store(parameters.balance.load());
        self.mute.store(parameters.mute.load(Relaxed), Relaxed);
    }
}

impl Default for QuadMixerInput {
    fn default() -> Self {
        Self {
            level: NormalizedValue::new(Defaults::QUAD_MIXER_LEVEL),
            balance: Balance::new(Defaults::QUAD_MIXER_BALANCE),
            mute: AtomicBool::new(false),
        }
    }
}

/// Parameters for the output mixer including per-oscillator sub-mixer inputs.
#[derive(Debug, Serialize, Deserialize)]
pub struct MixerParameters {
    /// Master output level.
    pub level: NormalizedValue,
    /// Master stereo balance position.
    pub balance: Balance,
    /// Whether the master output is muted.
    pub is_muted: AtomicBool,
    /// Per-oscillator mixer input controls.
    pub quad_mixer_inputs: [QuadMixerInput; 4],
}

impl MixerParameters {
    /// Replace all the values in this `MixerParameters` with the values from the provided `MixerParameters`.
    pub fn assign_from(&self, parameters: &MixerParameters) {
        self.level.store(parameters.level.load());
        self.balance.store(parameters.balance.load());
        self.is_muted
            .store(parameters.is_muted.load(Relaxed), Relaxed);

        parameters
            .quad_mixer_inputs
            .iter()
            .enumerate()
            .for_each(|(index, input)| {
                self.quad_mixer_inputs[index].assign_from(input);
            });
    }
}

impl Default for MixerParameters {
    fn default() -> Self {
        Self {
            level: NormalizedValue::new(Defaults::OUTPUT_MIXER_LEVEL),
            balance: Balance::new(Defaults::OUTPUT_MIXER_BALANCE),
            is_muted: AtomicBool::new(false),
            quad_mixer_inputs: Default::default(),
        }
    }
}

/// Shared parameter state for all synthesizer modules, accessed atomically from multiple threads.
#[derive(Default, Debug, Serialize, Deserialize)]
pub struct ModuleParameters {
    /// Filter module parameters.
    pub filter: FilterParameters,
    /// Output mixer parameters including per-oscillator levels.
    pub mixer: MixerParameters,
    /// Keyboard-related parameters such as velocity curve and pitch bend range.
    pub keyboard: KeyboardParameters,
    /// Parameters for the two LFOs (mod wheel and filter).
    pub lfos: [LfoParameters; 2],
    /// Parameters for the two envelopes (amplitude and filter).
    pub envelopes: [EnvelopeParameters; 2],
    /// Parameters for each of the four oscillators.
    pub oscillators: [OscillatorParameters; OscillatorIndex::COUNT],
    /// Parameters for the audio effects chain.
    pub effects: Vec<AudioEffectParameters>,
}

/// Top-level synthesizer coordinating MIDI input, DSP processing, and audio output.
pub struct Synthesizer {
    output_stream_parameters: OutputStreamParameters,
    current_note: Arc<CurrentNote>,
    module_parameters: Arc<ModuleParameters>,
    ui_update_sender: Sender<SynthesizerUpdateEvents>,
    ui_update_receiver: Receiver<SynthesizerUpdateEvents>,
    patches: Arc<Mutex<Patches>>,
}

impl Synthesizer {
    /// Creates a new synthesizer with the given audio output stream parameters.
    pub fn new(output_stream_parameters: OutputStreamParameters) -> Result<Self> {
        log::info!(target: "synthesizer", "Constructing Synthesizer Module");

        let (ui_update_sender, ui_update_receiver) =
            crossbeam_channel::bounded(SYNTHESIZER_MESSAGE_SENDER_CAPACITY);

        let module_parameters = patches::init_module_parameters()?;
        let patches = Patches::new()?;

        Ok(Self {
            output_stream_parameters,
            current_note: Arc::new(CurrentNote::default()),
            module_parameters: Arc::new(module_parameters),
            ui_update_sender,
            ui_update_receiver,
            patches: Arc::new(Mutex::new(patches)),
        })
    }

    /// Returns a clone of the channel sender for dispatching synthesizer update events.
    pub fn get_ui_update_sender(&self) -> Sender<SynthesizerUpdateEvents> {
        self.ui_update_sender.clone()
    }

    /// Returns a shared reference to the synthesizer module parameters.
    pub fn get_module_parameters(&self) -> Arc<ModuleParameters> {
        self.module_parameters.clone()
    }

    /// Starts the synthesizer's MIDI listener, event handler, and sample generation threads.
    pub fn run(
        &mut self,
        midi_message_receiver: Receiver<MidiEvent>,
        sample_buffer_receiver: Receiver<Producer<f32>>,
        ui_update_sender: Sender<UIUpdates>,
    ) -> Result<()> {
        log::debug!(target: "synthesizer", "Start the midi event listener thread");
        self.start_midi_event_listener(midi_message_receiver, ui_update_sender.clone());

        log::debug!(target: "synthesizer", "Start the update event listener thread");
        start_update_event_listener(
            self.ui_update_receiver.clone(),
            self.module_parameters.clone(),
            self.patches.clone(),
            ui_update_sender,
        );

        log::debug!(target: "synthesizer", "Start the synthesizer thread");
        sample_generator(
            sample_buffer_receiver,
            self.output_stream_parameters.clone(),
            &self.current_note,
            &self.module_parameters,
        )?;

        Ok(())
    }

    fn start_midi_event_listener(
        &mut self,
        midi_message_receiver: Receiver<MidiEvent>,
        ui_update_sender: Sender<UIUpdates>,
    ) {
        let mut current_note = self.current_note.clone();
        let mut module_parameters = self.module_parameters.clone();

        thread::spawn(move || {
            log::debug!("start_midi_event_listener(): spawned thread to receive MIDI events");

            while let Ok(event) = midi_message_receiver.recv() {
                match event {
                    MidiEvent::NoteOn(midi_note, velocity) => {
                        process_midi_note_on_message(
                            &mut module_parameters,
                            &mut current_note,
                            midi_note,
                            velocity,
                            &ui_update_sender,
                        );
                    }
                    MidiEvent::NoteOff => {
                        process_midi_note_off_message(&mut module_parameters);
                    }
                    MidiEvent::PitchBend(bend_amount) => {
                        process_midi_pitch_bend_message(
                            &module_parameters.oscillators,
                            module_parameters.keyboard.pitch_bend_range.load(Relaxed),
                            bend_amount,
                        );
                    }
                    MidiEvent::ChannelPressure(pressure_value) => {
                        process_midi_channel_pressure_message(
                            &module_parameters.keyboard,
                            pressure_value,
                        );
                    }
                    MidiEvent::ControlChange(cc_value) => {
                        process_midi_cc_values(cc_value, &mut module_parameters, &ui_update_sender);
                    }
                }
            }

            log::debug!("run(): MIDI event receiver thread has exited");
        });
    }

    /// Returns a clone of the Arc<Mutex<Patches>> for this synthesizer.
    pub fn patches(&self) -> Arc<Mutex<Patches>> {
        self.patches.clone()
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
        level: parameters.quad_mixer_inputs[oscillator as usize]
            .level
            .load(),
        balance: parameters.quad_mixer_inputs[oscillator as usize]
            .balance
            .load(),
        mute: parameters.quad_mixer_inputs[oscillator as usize]
            .mute
            .load(Relaxed),
    }
}
