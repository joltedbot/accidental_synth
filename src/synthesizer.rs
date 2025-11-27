mod constants;
mod create_synthesizer;
mod midi_messages;
mod midi_value_converters;
mod set_parameters;

use self::constants::{
    DEFAULT_FILTER_ENVELOPE_AMOUNT, DEFAULT_OUTPUT_BALANCE, DEFAULT_OUTPUT_LEVEL,
    DEFAULT_VELOCITY_CURVE, DEFAULT_VIBRATO_LFO_CENTER_FREQUENCY, DEFAULT_VIBRATO_LFO_DEPTH,
    DEFAULT_VIBRATO_LFO_RATE, MAX_MIDI_KEY_VELOCITY, QUAD_MIX_DEFAULT_BALANCE,
    QUAD_MIX_DEFAULT_INPUT_LEVEL, QUAD_MIX_DEFAULT_SUB_INPUT_LEVEL,LFO_INDEX_FILTER, ENVELOPE_INDEX_FILTER
};

use crate::math::{load_f32_from_atomic_u32, store_f32_as_atomic_u32};
use crate::midi::Event;
use crate::modules::envelope::EnvelopeParameters;
use crate::modules::filter::{DEFAULT_KEY_TRACKING_AMOUNT, FilterParameters};
use crate::modules::lfo::LfoParameters;
use crate::modules::mixer::MixerInput;
use crate::modules::oscillator::OscillatorParameters;
use crate::synthesizer::constants::{
    SYNTHESIZER_MESSAGE_SENDER_CAPACITY, UI_TO_SYNTHESIZER_WAVESHAPE_INDEX_OFFSET,
};
use crate::synthesizer::create_synthesizer::create_synthesizer;
use crate::synthesizer::midi_messages::{
    process_midi_cc_values, process_midi_channel_pressure_message, process_midi_note_off_message,
    process_midi_note_on_message, process_midi_pitch_bend_message,
};
use crate::synthesizer::set_parameters::{set_envelope_amount, set_envelope_attack_time, set_envelope_decay_time, set_envelope_inverted, set_envelope_release_time, set_envelope_sustain_level, set_filter_cutoff, set_filter_poles, set_filter_resonance, set_key_tracking_amount, set_lfo_frequency, set_lfo_phase, set_lfo_phase_reset, set_lfo_range, set_lfo_wave_shape, set_oscillator_clip_boost, set_oscillator_course_tune, set_oscillator_fine_tune, set_oscillator_shape_parameter1, set_oscillator_shape_parameter2};
use anyhow::Result;
use crossbeam_channel::{Receiver, Sender};
use rtrb::Producer;
use std::default::Default;
use std::sync::Arc;
use std::sync::atomic::Ordering::Relaxed;
use std::sync::atomic::{AtomicBool, AtomicU8, AtomicU32};
use std::thread;

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
    ui_update_sender: Sender<SynthesizerUpdateEvents>,
    ui_update_receiver: Receiver<SynthesizerUpdateEvents>,
}

impl Synthesizer {
    pub fn new(sample_rate: u32) -> Self {
        log::info!("Constructing Synthesizer Module");

        let (ui_update_sender, ui_update_receiver) =
            crossbeam_channel::bounded(SYNTHESIZER_MESSAGE_SENDER_CAPACITY);

        let velocity = AtomicU32::new(0);
        store_f32_as_atomic_u32(&velocity, MAX_MIDI_KEY_VELOCITY);

        let velocity_curve = AtomicU32::new(0);
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
    ) -> Result<()> {
        log::debug!("run(): Start the midi event listener thread");
        self.start_midi_event_listener(midi_message_receiver);

        self.start_ui_event_listener();

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

    fn start_ui_event_listener(&self) {
        let ui_update_receiver = self.ui_update_receiver.clone();
        let module_parameters = self.module_parameters.clone();
        thread::spawn(move || {
            log::debug!("start_ui_event_listener(): spawned thread to receive UI events");

            while let Ok(event) = ui_update_receiver.recv() {
                match event {
                    SynthesizerUpdateEvents::WaveShapeIndex(oscillator_index, wave_shape_index) => {
                        if oscillator_index >= 0
                            && oscillator_index < module_parameters.oscillators.len() as i32
                        {
                            let reindexed_wave_shape =
                                (wave_shape_index + UI_TO_SYNTHESIZER_WAVESHAPE_INDEX_OFFSET) as u8;
                            module_parameters.oscillators[oscillator_index as usize]
                                .wave_shape_index
                                .store(reindexed_wave_shape, Relaxed);
                        } else {
                            log::error!(
                                "start_ui_event_listener(): Invalid oscillator index: {oscillator_index}"
                            );
                        }
                    }
                    SynthesizerUpdateEvents::CourseTune(oscillator_index, course_tune) => {
                        if oscillator_index >= 0
                            && oscillator_index < module_parameters.oscillators.len() as i32
                        {
                            set_oscillator_course_tune(
                                &module_parameters.oscillators[oscillator_index as usize],
                                course_tune,
                            );
                        } else {
                            log::error!(
                                "start_ui_event_listener(): Invalid oscillator index: {oscillator_index}"
                            );
                        }
                    }
                    SynthesizerUpdateEvents::FineTune(oscillator_index, fine_tune) => {
                        if oscillator_index >= 0
                            && oscillator_index < module_parameters.oscillators.len() as i32
                        {
                            set_oscillator_fine_tune(
                                &module_parameters.oscillators[oscillator_index as usize],
                                fine_tune,
                            );
                        } else {
                            log::error!(
                                "start_ui_event_listener(): Invalid oscillator index: {oscillator_index}"
                            );
                        }
                    }
                    SynthesizerUpdateEvents::ClipperBoost(oscillator_index, boost) => {
                        if oscillator_index >= 0
                            && oscillator_index < module_parameters.oscillators.len() as i32
                        {
                            set_oscillator_clip_boost(
                                &module_parameters.oscillators[oscillator_index as usize],
                                boost,
                            );
                        } else {
                            log::error!(
                                "start_ui_event_listener(): Invalid oscillator index: {oscillator_index}"
                            );
                        }
                    }
                    SynthesizerUpdateEvents::Parameter1(oscillator_index, boost) => {
                        if oscillator_index >= 0
                            && oscillator_index < module_parameters.oscillators.len() as i32
                        {
                            set_oscillator_shape_parameter1(
                                &module_parameters.oscillators[oscillator_index as usize],
                                boost,
                            );
                        } else {
                            log::error!(
                                "start_ui_event_listener(): Invalid oscillator index: {oscillator_index}"
                            );
                        }
                    }
                    SynthesizerUpdateEvents::Parameter2(oscillator_index, boost) => {
                        if oscillator_index >= 0
                            && oscillator_index < module_parameters.oscillators.len() as i32
                        {
                            set_oscillator_shape_parameter2(
                                &module_parameters.oscillators[oscillator_index as usize],
                                boost,
                            );
                        } else {
                            log::error!(
                                "start_ui_event_listener(): Invalid oscillator index: {oscillator_index}"
                            );
                        }
                    }
                    SynthesizerUpdateEvents::FilterCutoffFrequency(frequency) => {
                        set_filter_cutoff(&module_parameters.filter, frequency);
                    }
                    SynthesizerUpdateEvents::FilterResonance(resonance) => {
                        set_filter_resonance(&module_parameters.filter, resonance);
                    }
                    SynthesizerUpdateEvents::FilterPoleCount(poles) => {
                        set_filter_poles(&module_parameters.filter, poles);
                    }
                    SynthesizerUpdateEvents::FilterKeyTrackingAmount(amount) => {
                        set_key_tracking_amount(&module_parameters.filter, amount);
                    }
                    SynthesizerUpdateEvents::FilterEnvelopeAmount(amount) => {
                        set_envelope_amount(&module_parameters.filter_envelope, amount);
                    }
                    SynthesizerUpdateEvents::FilterLfoAmount(amount) => {
                        set_lfo_range(&module_parameters.filter_lfo, amount);
                    }
                    SynthesizerUpdateEvents::FilterEnvelopeAttack(envelope_index, milliseconds) => {
                        match envelope_index {
                            ENVELOPE_INDEX_FILTER => set_envelope_attack_time(&module_parameters.filter_envelope, milliseconds),
                            _=> {
                                log::error!("start_ui_event_listener():SynthesizerUpdateEvents::FilterEnvelopeAttack: Invalid \
                                Envelope index: {envelope_index}");
                            }
                        }
                    }
                    SynthesizerUpdateEvents::FilterEnvelopeDecay(envelope_index, milliseconds) => {
                        match envelope_index {
                            ENVELOPE_INDEX_FILTER => set_envelope_decay_time(&module_parameters.filter_envelope,
                                                                             milliseconds),
                            _=> {
                                log::error!("start_ui_event_listener():SynthesizerUpdateEvents::FilterEnvelopeDecay: Invalid \
                                Envelope index: {envelope_index}");
                            }
                        }
                    }
                    SynthesizerUpdateEvents::FilterEnvelopeSustain(envelope_index, level) => {
                        match envelope_index {
                            ENVELOPE_INDEX_FILTER => set_envelope_sustain_level(&module_parameters.filter_envelope,
                                                                                level),
                            _=> {
                                log::error!("start_ui_event_listener():SynthesizerUpdateEvents::FilterEnvelopeSustain: Invalid \
                                Envelope index: {envelope_index}");
                            }
                        }
                    }
                    SynthesizerUpdateEvents::FilterEnvelopeRelease(envelope_index, milliseconds) => {
                        match envelope_index {
                            ENVELOPE_INDEX_FILTER => set_envelope_release_time(&module_parameters.filter_envelope,
                                                                               milliseconds),
                            _=> {
                                log::error!("start_ui_event_listener():SynthesizerUpdateEvents::FilterEnvelopeRelease: Invalid \
                                Envelope index: {envelope_index}");
                            }
                        }
                    }
                    SynthesizerUpdateEvents::FilterEnvelopeInvert(envelope_index, is_inverted) => {
                        match envelope_index {
                            ENVELOPE_INDEX_FILTER => set_envelope_inverted(&module_parameters.filter_envelope,
                                                                           f32::from(is_inverted)),
                            _=> {
                                log::error!("start_ui_event_listener():SynthesizerUpdateEvents::FilterEnvelopeInvert: Invalid \
                                Envelope index: {envelope_index}");
                            }
                        }
                    }
                    SynthesizerUpdateEvents::LfoFrequency(lfo_index, frequency) => {
                        match lfo_index {
                            LFO_INDEX_FILTER => set_lfo_frequency(&module_parameters.filter_lfo, frequency),
                            _=> {
                                log::error!("start_ui_event_listener():SynthesizerUpdateEvents::LfoFrequency: Invalid LFO index: {lfo_index}");
                            }
                        }
                    }
                    SynthesizerUpdateEvents::LfoShapeIndex(lfo_index, wave_shape_index) => {
                        match lfo_index {
                            LFO_INDEX_FILTER => {
                                let reindexed_wave_shape = (wave_shape_index + UI_TO_SYNTHESIZER_WAVESHAPE_INDEX_OFFSET) as u8;
                                &module_parameters.filter_lfo.wave_shape.store(reindexed_wave_shape, Relaxed);
                            }
                            _=> {
                                log::error!("start_ui_event_listener():SynthesizerUpdateEvents::LfoShapeIndex: Invalid\
                                 LFO index: {lfo_index}");
                            }
                        }
                    }
                    SynthesizerUpdateEvents::LfoPhase(lfo_index, phase) => {
                        match lfo_index {
                            LFO_INDEX_FILTER => set_lfo_phase(&module_parameters.filter_lfo, phase),
                            _=> {
                                log::error!("start_ui_event_listener():SynthesizerUpdateEvents::LfoPhase: Invalid LFO index: {lfo_index}");
                            }
                        }
                    }
                    SynthesizerUpdateEvents::LfoPhaseReset(lfo_index) => {
                        match lfo_index {
                            LFO_INDEX_FILTER => set_lfo_phase_reset(&module_parameters.filter_lfo),
                            _=> {
                                log::error!("start_ui_event_listener():SynthesizerUpdateEvents::LfoPhaseReset: Invalid LFO index: {lfo_index}");
                            }
                        }
                    }
                }
            }
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