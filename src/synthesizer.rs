mod constants;
mod event_listener;
mod midi_messages;
pub mod midi_value_converters;
mod sample_generator;
mod set_parameters;

use self::constants::{
    DEFAULT_FILTER_ENVELOPE_AMOUNT, DEFAULT_VELOCITY_CURVE, DEFAULT_VIBRATO_LFO_CENTER_FREQUENCY,
    DEFAULT_VIBRATO_LFO_DEPTH, DEFAULT_VIBRATO_LFO_RATE, MAX_MIDI_KEY_VELOCITY,
};

use crate::math::{load_f32_from_atomic_u32, store_f32_as_atomic_u32};
use crate::midi::Event;
use crate::modules::envelope::EnvelopeParameters;
use crate::modules::filter::FilterParameters;
use crate::modules::filter::max_frequency_from_sample_rate;
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

use crate::audio::OutputStreamParameters;
use crate::defaults::Defaults;
use crate::modules::effects::{AudioEffectParameters, default_audio_effect_parameters};
use crate::ui::UIUpdates;
use anyhow::Result;
use crossbeam_channel::{Receiver, Sender};
use rtrb::Producer;
use std::default::Default;
use std::sync::Arc;
use std::sync::atomic::Ordering::Relaxed;
use std::sync::atomic::{AtomicBool, AtomicU8, AtomicU32};
use std::thread;
use strum_macros::{EnumCount, EnumIter, FromRepr};

pub enum SynthesizerUpdateEvents {
    WaveShapeIndex(i32, i32),
    CourseTune(i32, f32),
    FineTune(i32, f32),
    ClipperBoost(i32, f32),
    Parameter1(i32, f32),
    Parameter2(i32, f32),
    FilterCutoffFrequency(f32),
    FilterResonance(f32),
    FilterPoleCount(f32),
    FilterKeyTrackingAmount(f32),
    FilterEnvelopeAmount(f32),
    FilterLfoAmount(f32),
    FilterEnvelopeAttack(i32, f32),
    FilterEnvelopeDecay(i32, f32),
    FilterEnvelopeSustain(i32, f32),
    FilterEnvelopeRelease(i32, f32),
    FilterEnvelopeInvert(i32, bool),
    LfoFrequency(i32, f32),
    LfoShapeIndex(i32, i32),
    LfoPhase(i32, f32),
    LfoPhaseReset(i32),
    PortamentoEnabled(bool),
    PortamentoTime(f32),
    PitchBendRange(f32),
    VelocityCurve(f32),
    HardSyncEnabled(bool),
    KeySyncEnabled(bool),
    OutputBalance(f32),
    OutputLevel(f32),
    OutputMute(bool),
    OscillatorMixerBalance(i32, f32),
    OscillatorMixerLevel(i32, f32),
    OscillatorMixerMute(i32, bool),
    EffectEnabled(i32, bool),
    EffectParameters(i32, i32, f32),
}

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

#[derive(Debug, Clone, Copy, EnumCount, EnumIter, FromRepr)]
#[repr(i32)]
pub enum OscillatorIndex {
    Sub = 0,
    One = 1,
    Two = 2,
    Three = 3,
}

impl OscillatorIndex {
    pub fn from_i32(index: i32) -> Option<Self> {
        Self::from_repr(index)
    }
}

#[derive(Debug, Clone, Copy, EnumCount, EnumIter, FromRepr)]
#[repr(i32)]
pub enum LFOIndex {
    ModWheel = 0,
    Filter = 1,
}

impl LFOIndex {
    pub fn from_i32(index: i32) -> Option<Self> {
        Self::from_repr(index)
    }
}

#[derive(Debug, Clone, Copy, EnumCount, EnumIter, FromRepr)]
#[repr(i32)]
pub enum EnvelopeIndex {
    Amp = 0,
    Filter = 1,
}

impl EnvelopeIndex {
    pub fn from_i32(index: i32) -> Option<Self> {
        Self::from_repr(index)
    }
}

#[derive(Debug)]
struct CurrentNote {
    midi_note: AtomicU8,
    velocity: AtomicU32,
    velocity_curve: AtomicU32,
}

impl Default for CurrentNote {
    fn default() -> Self {
        Self {
            midi_note: AtomicU8::new(0),
            velocity: AtomicU32::new(MAX_MIDI_KEY_VELOCITY.to_bits()),
            velocity_curve: AtomicU32::new(DEFAULT_VELOCITY_CURVE.to_bits()),
        }
    }
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
            level: AtomicU32::new(Defaults::QUAD_MIXER_LEVEL.to_bits()),
            balance: AtomicU32::new(Defaults::QUAD_MIXER_BALANCE.to_bits()),
            mute: AtomicBool::new(false),
        }
    }
}

#[derive(Debug)]
struct MixerParameters {
    output_level: AtomicU32,
    output_balance: AtomicU32,
    output_is_muted: AtomicBool,
    quad_mixer_inputs: [QuadMixerInput; 4],
}

impl Default for MixerParameters {
    fn default() -> Self {
        Self {
            output_level: AtomicU32::new(Defaults::OUTPUT_MIXER_LEVEL.to_bits()),
            output_balance: AtomicU32::new(Defaults::OUTPUT_MIXER_BALANCE.to_bits()),
            output_is_muted: AtomicBool::new(false),
            quad_mixer_inputs: Default::default(),
        }
    }
}

#[derive(Default, Debug)]
pub struct ModuleParameters {
    filter: FilterParameters,
    mixer: MixerParameters,
    keyboard: KeyboardParameters,
    lfos: [LfoParameters; 2],
    envelopes: [EnvelopeParameters; 2],
    oscillators: [OscillatorParameters; 4],
    effects: Vec<AudioEffectParameters>,
}

pub struct Synthesizer {
    output_stream_parameters: OutputStreamParameters,
    current_note: Arc<CurrentNote>,
    module_parameters: Arc<ModuleParameters>,
    ui_update_sender: Sender<SynthesizerUpdateEvents>,
    ui_update_receiver: Receiver<SynthesizerUpdateEvents>,
}

impl Synthesizer {
    pub fn new(output_stream_parameters: OutputStreamParameters) -> Self {
        log::info!(target: "synthesizer", "Constructing Synthesizer Module");

        let (ui_update_sender, ui_update_receiver) =
            crossbeam_channel::bounded(SYNTHESIZER_MESSAGE_SENDER_CAPACITY);

        let mixer_parameters: MixerParameters = MixerParameters::default();
        store_f32_as_atomic_u32(
            &mixer_parameters.quad_mixer_inputs[0].level,
            Defaults::QUAD_MIXER_SUB_LEVEL,
        );

        let max_filter_frequency =
            max_frequency_from_sample_rate(output_stream_parameters.sample_rate.load(Relaxed));
        let filter_parameters = FilterParameters {
            cutoff_frequency: AtomicU32::new(max_filter_frequency.to_bits()),
            ..Default::default()
        };

        let filter_envelope_parameters: EnvelopeParameters = EnvelopeParameters::default();
        filter_envelope_parameters
            .amount
            .store(DEFAULT_FILTER_ENVELOPE_AMOUNT.to_bits(), Relaxed);

        let filter_lfo_parameters: LfoParameters = LfoParameters::default();
        store_f32_as_atomic_u32(&filter_lfo_parameters.range, 0.0);

        let mod_wheel_lfo_parameters = LfoParameters::default();
        mod_wheel_lfo_parameters
            .frequency
            .store(DEFAULT_VIBRATO_LFO_RATE.to_bits(), Relaxed);
        mod_wheel_lfo_parameters
            .center_value
            .store(DEFAULT_VIBRATO_LFO_CENTER_FREQUENCY.to_bits(), Relaxed);
        mod_wheel_lfo_parameters
            .range
            .store(DEFAULT_VIBRATO_LFO_DEPTH.to_bits(), Relaxed);

        let keyboard_parameters = KeyboardParameters {
            pitch_bend_range: AtomicU8::new(Defaults::PITCH_BEND_RANGE),
            ..KeyboardParameters::default()
        };

        let envelopes = [EnvelopeParameters::default(), filter_envelope_parameters];
        let lfos = [mod_wheel_lfo_parameters, filter_lfo_parameters];

        let effects = default_audio_effect_parameters();

        let module_parameters = ModuleParameters {
            filter: filter_parameters,
            mixer: mixer_parameters,
            keyboard: keyboard_parameters,
            oscillators: Default::default(),
            lfos,
            envelopes,
            effects,
        };

        Self {
            output_stream_parameters,
            current_note: Arc::new(CurrentNote::default()),
            module_parameters: Arc::new(module_parameters),
            ui_update_sender,
            ui_update_receiver,
        }
    }

    pub fn get_ui_update_sender(&self) -> Sender<SynthesizerUpdateEvents> {
        self.ui_update_sender.clone()
    }

    pub fn run(
        &mut self,
        midi_message_receiver: Receiver<Event>,
        sample_buffer_receiver: Receiver<Producer<f32>>,
        ui_update_sender: Sender<UIUpdates>,
    ) -> Result<()> {
        log::debug!(target: "synthesizer", "Start the midi event listener thread");
        self.start_midi_event_listener(midi_message_receiver, ui_update_sender.clone());

        log::debug!(target: "synthesizer", "Start the update event listener thread");
        start_update_event_listener(
            self.ui_update_receiver.clone(),
            self.module_parameters.clone(),
            self.current_note.clone(),
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
        midi_message_receiver: Receiver<Event>,
        ui_update_sender: Sender<UIUpdates>,
    ) {
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
                            &ui_update_sender,
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
                        process_midi_cc_values(
                            cc_value,
                            &mut current_note,
                            &mut module_parameters,
                            &ui_update_sender,
                        );
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
