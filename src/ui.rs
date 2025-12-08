mod callbacks;
mod constants;
mod structs;
mod update_listener;

use super::{AccidentalSynth, AudioDevice, EnvelopeValues, LFOValues, MidiPort, Oscillator};
use crate::audio::AudioDeviceUpdateEvents;
use crate::midi::MidiDeviceUpdateEvents;
use crate::modules::lfo::DEFAULT_LFO_FREQUENCY;
use crate::synthesizer::midi_value_converters::{
    exponential_curve_lfo_frequency_from_normal_value, normal_value_to_bool,
    normal_value_to_wave_shape_index,
};
use crate::synthesizer::{OscillatorIndex, SynthesizerUpdateEvents};
use crate::ui::callbacks::register_callbacks;
use crate::ui::constants::{
    AUDIO_DEVICE_CHANNEL_INDEX_TO_NAME_OFFSET, AUDIO_DEVICE_CHANNEL_NULL_VALUE, MAX_PHASE_VALUE,
    MIDI_CHANNEL_LIST, MONO_CHANNEL_COUNT,
};
use crate::ui::structs::{UIAudioDevice, UIEnvelope, UILfo, UIMidiPort, UIOscillator};
use crate::ui::update_listener::start_ui_update_listener;
use anyhow::Result;
use crossbeam_channel::{Receiver, Sender, bounded};
use slint::{ModelRc, SharedString, VecModel, Weak};
use std::rc::Rc;
use std::sync::{Arc, Mutex};
use std::thread;

const UI_UPDATE_CHANNEL_CAPACITY: usize = 10;

#[derive(Debug, Clone, PartialEq)]
pub enum UIUpdates {
    MidiPortList(Vec<String>),
    MidiPortIndex(i32),
    MidiChannelIndex(i32),
    AudioDeviceList(Vec<String>),
    AudioDeviceIndex(i32),
    AudioDeviceChannelCount(u16),
    AudioDeviceChannelIndexes { left: i32, right: i32 },
    OscillatorWaveShape(i32, i32),
    OscillatorFineTune(i32, f32, i32),
    OscillatorFineTuneCents(i32, i32),
    OscillatorCourseTune(i32, i32),
    OscillatorClipperBoost(i32, f32),
    OscillatorParameter1(i32, f32),
    OscillatorParameter2(i32, f32),
    LFOFrequency(i32, f32),
    LFOFrequencyDisplay(i32, f32),
    LFOPhase(i32, f32),
    LFOWaveShape(i32, f32),
    EnvelopeAttackTime(i32, f32),
    EnvelopeDecayTime(i32, f32),
    EnvelopeSustainLevel(i32, f32),
    EnvelopeReleaseTime(i32, f32),
    EnvelopeInverted(i32, f32),
}

#[derive(Debug, Clone, Copy)]
pub enum LFOIndex {
    ModWheel = 0,
    Filter = 1,
}

impl LFOIndex {
    pub fn from_i32(index: i32) -> Option<Self> {
        match index {
            0 => Some(Self::ModWheel),
            1 => Some(Self::Filter),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub enum EnvelopeIndex {
    Amp = 0,
    Filter = 1,
}

impl EnvelopeIndex {
    pub fn from_i32(index: i32) -> Option<Self> {
        match index {
            0 => Some(Self::Amp),
            1 => Some(Self::Filter),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub enum EnvelopeStage {
    Attack,
    Decay,
    Sustain,
    Release,
}

struct ParameterValues {
    audio_device: Arc<Mutex<UIAudioDevice>>,
    midi_port: Arc<Mutex<UIMidiPort>>,
    amp_envelope: Arc<Mutex<UIEnvelope>>,
    filter_envelope: Arc<Mutex<UIEnvelope>>,
    mod_wheel_lfo: Arc<Mutex<UILfo>>,
    filter_lfo: Arc<Mutex<UILfo>>,
    oscillator_fine_tune: Arc<Mutex<Vec<i32>>>,
}

pub struct UI {
    ui_update_sender: Sender<UIUpdates>,
    ui_update_receiver: Receiver<UIUpdates>,
    oscillators: Arc<Mutex<Vec<UIOscillator>>>,
    parameter_values: ParameterValues,
}

impl UI {
    pub fn new() -> Self {
        log::info!("Constructing UI Module");

        let (ui_update_sender, ui_update_receiver) = bounded(UI_UPDATE_CHANNEL_CAPACITY);

        let midi_port_values = UIMidiPort {
            channels: MIDI_CHANNEL_LIST.iter().map(ToString::to_string).collect(),
            ..Default::default()
        };

        let audio_device_values = UIAudioDevice {
            left_channel_index: 0,
            right_channel_index: 1,
            ..Default::default()
        };

        let oscillator_values = vec![UIOscillator::default(); OscillatorIndex::count()];

        let parameter_values = ParameterValues {
            audio_device: Arc::new(Mutex::new(audio_device_values)),
            midi_port: Arc::new(Mutex::new(midi_port_values)),
            amp_envelope: Arc::new(Mutex::new(UIEnvelope::default())),
            filter_envelope: Arc::new(Mutex::new(UIEnvelope::default())),
            mod_wheel_lfo: Arc::new(Mutex::new(UILfo::default())),
            filter_lfo: Arc::new(Mutex::new(UILfo::default())),
            oscillator_fine_tune: Arc::new(Mutex::new(vec![0; OscillatorIndex::count()])),
        };

        Self {
            ui_update_sender,
            ui_update_receiver,
            parameter_values,
            oscillators: Arc::new(Mutex::new(oscillator_values)),
        }
    }

    pub fn get_ui_update_sender(&self) -> Sender<UIUpdates> {
        self.ui_update_sender.clone()
    }

    pub fn run(
        &mut self,
        ui_weak: &Weak<AccidentalSynth>,
        midi_update_sender: Sender<MidiDeviceUpdateEvents>,
        audio_output_device_sender: &Sender<AudioDeviceUpdateEvents>,
        synthesizer_update_sender: &Sender<SynthesizerUpdateEvents>,
    ) -> Result<()> {
        let ui_update_receiver = self.ui_update_receiver.clone();
        register_callbacks(
            &ui_weak.clone(),
            midi_update_sender,
            audio_output_device_sender,
            synthesizer_update_sender,
        );
        self.set_ui_default_values(ui_weak)?;
        start_ui_update_listener(
            ui_update_receiver,
            ui_weak,
            self.oscillators.clone(),
            &self.parameter_values,
        );

        Ok(())
    }

    fn set_ui_default_values(&self, ui_weak: &Weak<AccidentalSynth>) -> Result<()> {
        let midi_port_values = self.parameter_values.midi_port.clone();
        let audio_device_values = self.parameter_values.audio_device.clone();
        let amp_envelope_values = self.parameter_values.amp_envelope.clone();
        let filter_envelope_values = self.parameter_values.filter_envelope.clone();
        let mod_wheel_lfo_values = self.parameter_values.mod_wheel_lfo.clone();
        let filter_lfo_values = self.parameter_values.filter_lfo.clone();

        ui_weak.upgrade_in_event_loop(move |ui| {
            ui.set_version(SharedString::from(env!("CARGO_PKG_VERSION")));

            let midi_ports = midi_port_values
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner);
            let audio_devices = audio_device_values
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner);
            let amp_envelope = amp_envelope_values
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner);
            let filter_envelope = filter_envelope_values
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner);
            let mod_wheel_lfo = mod_wheel_lfo_values
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner);
            let filter_lfo = filter_lfo_values
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner);

            ui.set_midi_port_values(slint_midi_port_from_ui_midi_port(&midi_ports));
            ui.set_audio_device_values(slint_audio_device_from_ui_audio_device(&audio_devices));
            ui.set_amp_eg_values(slint_envelope_from_ui_envelope(&amp_envelope));
            ui.set_filter_eg_values(slint_envelope_from_ui_envelope(&filter_envelope));
            ui.set_mod_wheel_lfo_values(slint_lfo_from_ui_lfo(&mod_wheel_lfo));
            ui.set_mod_wheel_lfo_frequency_display(DEFAULT_LFO_FREQUENCY);
            ui.set_filter_lfo_values(slint_lfo_from_ui_lfo(&filter_lfo));
            ui.set_filter_lfo_frequency_display(DEFAULT_LFO_FREQUENCY);
        })?;

        Ok(())
    }
}

fn set_midi_port_list(
    ui_weak_thread: &Weak<AccidentalSynth>,
    midi_port_values: &Arc<Mutex<UIMidiPort>>,
    port_list: Vec<String>,
) {
    let mut midi_ports = midi_port_values
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    midi_ports.input_ports = port_list;
    let ui_midi_ports = midi_ports.clone();
    let _ = ui_weak_thread.upgrade_in_event_loop(move |ui| {
        ui.set_midi_port_values(slint_midi_port_from_ui_midi_port(&ui_midi_ports));
    });
}

fn set_midi_port_index(
    ui_weak_thread: &Weak<AccidentalSynth>,
    midi_port_values: &Arc<Mutex<UIMidiPort>>,
    port_index: i32,
) {
    let mut midi_ports = midi_port_values
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    midi_ports.input_port_index = port_index;
    let ui_midi_ports = midi_ports.clone();
    let _ = ui_weak_thread.upgrade_in_event_loop(move |ui| {
        ui.set_midi_port_values(slint_midi_port_from_ui_midi_port(&ui_midi_ports));
    });
}

fn set_midi_channel_index(
    ui_weak_thread: &Weak<AccidentalSynth>,
    midi_port_values: &Arc<Mutex<UIMidiPort>>,
    channel_index: i32,
) {
    let mut midi_ports = midi_port_values
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    midi_ports.channel_index = channel_index;
    let ui_midi_ports = midi_ports.clone();
    let _ = ui_weak_thread.upgrade_in_event_loop(move |ui| {
        ui.set_midi_port_values(slint_midi_port_from_ui_midi_port(&ui_midi_ports));
    });
}

fn set_audio_device_list(
    ui_weak_thread: &Weak<AccidentalSynth>,
    audio_device_values: &Arc<Mutex<UIAudioDevice>>,
    device_list: Vec<String>,
) {
    let mut audio_devices = audio_device_values
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    audio_devices.output_devices = device_list;
    let ui_audio_devices = audio_devices.clone();
    let _ = ui_weak_thread.upgrade_in_event_loop(move |ui| {
        ui.set_audio_device_values(slint_audio_device_from_ui_audio_device(&ui_audio_devices));
    });
}

fn set_audio_device_index(
    ui_weak: &Weak<AccidentalSynth>,
    audio_device_values: &Arc<Mutex<UIAudioDevice>>,
    audio_device_index: i32,
) {
    let mut audio_devices = audio_device_values
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    audio_devices.output_device_index = audio_device_index;
    let ui_audio_devices = audio_devices.clone();
    let _ = ui_weak.upgrade_in_event_loop(move |ui| {
        ui.set_audio_device_values(slint_audio_device_from_ui_audio_device(&ui_audio_devices));
    });
}

fn set_audio_device_channel_list(
    ui_weak_thread: &Weak<AccidentalSynth>,
    audio_device_values: &Arc<Mutex<UIAudioDevice>>,
    channel_count: u16,
) {
    let mut audio_devices = audio_device_values
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);

    let mut device_channels: Vec<String> = vec![];
    for channel in 0..channel_count {
        device_channels.push((channel + AUDIO_DEVICE_CHANNEL_INDEX_TO_NAME_OFFSET).to_string());
    }

    audio_devices.left_channels.clone_from(&device_channels);
    if channel_count > MONO_CHANNEL_COUNT {
        audio_devices.right_channels = device_channels;
    } else {
        audio_devices.right_channels = vec![];
    }

    let ui_audio_devices = audio_devices.clone();
    let _ = ui_weak_thread.upgrade_in_event_loop(move |ui| {
        ui.set_audio_device_values(slint_audio_device_from_ui_audio_device(&ui_audio_devices));
    });
}

fn set_audio_device_channel_indexes(
    ui_weak_thread: &Weak<AccidentalSynth>,
    audio_device_values: &Arc<Mutex<UIAudioDevice>>,
    left_chanel_index: i32,
    right_channel_index: i32,
) {
    let mut audio_devices = audio_device_values
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);

    audio_devices.left_channel_index = left_chanel_index;

    if right_channel_index == AUDIO_DEVICE_CHANNEL_NULL_VALUE {
        audio_devices.right_channel_index = AUDIO_DEVICE_CHANNEL_NULL_VALUE;
    } else {
        audio_devices.right_channel_index = right_channel_index;
    }

    let ui_audio_devices = audio_devices.clone();
    let _ = ui_weak_thread.upgrade_in_event_loop(move |ui| {
        ui.set_audio_device_values(slint_audio_device_from_ui_audio_device(&ui_audio_devices));
    });
}

fn set_oscillator_wave_shape(
    ui_weak_thread: &Weak<AccidentalSynth>,
    oscillator_values: &Arc<Mutex<Vec<UIOscillator>>>,
    oscillator_index: i32,
    wave_shape_index: i32,
) {
    let mut oscillators = oscillator_values
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    oscillators[oscillator_index as usize].wave_shape_index = wave_shape_index;

    let ui_oscillators = oscillators.clone();
    let _ = ui_weak_thread.upgrade_in_event_loop(move |ui| {
        ui.set_oscillators(slint_oscillators_from_oscillators(&ui_oscillators));
    });
}

fn set_oscillator_fine_tune(
    ui_weak_thread: &Weak<AccidentalSynth>,
    oscillator_values: &Arc<Mutex<Vec<UIOscillator>>>,
    oscillator_index: i32,
    normal_value: f32,
) {
    let mut oscillators = oscillator_values
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    oscillators[oscillator_index as usize].fine_tune = normal_value;

    let ui_oscillators = oscillators.clone();
    let _ = ui_weak_thread.upgrade_in_event_loop(move |ui| {
        ui.set_oscillators(slint_oscillators_from_oscillators(&ui_oscillators));
    });
}

fn set_oscillator_fine_tune_display(
    ui_weak_thread: &Weak<AccidentalSynth>,
    oscillator_fine_tune_values: &Arc<Mutex<Vec<i32>>>,
    oscillator_index: i32,
    cents: i32,
) {
    let mut fine_tune_values = oscillator_fine_tune_values
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    fine_tune_values[oscillator_index as usize] = cents;

    let ui_oscillator_fine_tune_values = fine_tune_values.clone();
    let _ = ui_weak_thread.upgrade_in_event_loop(move |ui| {
        ui.set_osc_fine_tune_cents(vec_to_model_rc_int(&ui_oscillator_fine_tune_values));
    });
}

fn set_oscillator_course_tune(
    ui_weak_thread: &Weak<AccidentalSynth>,
    oscillator_values: &Arc<Mutex<Vec<UIOscillator>>>,
    oscillator_index: i32,
    course_tune: i32,
) {
    let mut oscillators = oscillator_values
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    oscillators[oscillator_index as usize].course_tune = course_tune;

    let ui_oscillators = oscillators.clone();
    let _ = ui_weak_thread.upgrade_in_event_loop(move |ui| {
        ui.set_oscillators(slint_oscillators_from_oscillators(&ui_oscillators));
    });
}

fn set_oscillator_clipper_boost(
    ui_weak_thread: &Weak<AccidentalSynth>,
    oscillator_values: &Arc<Mutex<Vec<UIOscillator>>>,
    oscillator_index: i32,
    boost_level: f32,
) {
    let mut oscillators = oscillator_values
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    oscillators[oscillator_index as usize].clipper_boost = boost_level;

    let ui_oscillators = oscillators.clone();
    let _ = ui_weak_thread.upgrade_in_event_loop(move |ui| {
        ui.set_oscillators(slint_oscillators_from_oscillators(&ui_oscillators));
    });
}
fn set_oscillator_parameter1(
    ui_weak_thread: &Weak<AccidentalSynth>,
    oscillator_values: &Arc<Mutex<Vec<UIOscillator>>>,
    oscillator_index: i32,
    value: f32,
) {
    let mut oscillators = oscillator_values
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    oscillators[oscillator_index as usize].parameter1 = value;

    let ui_oscillators = oscillators.clone();
    let _ = ui_weak_thread.upgrade_in_event_loop(move |ui| {
        ui.set_oscillators(slint_oscillators_from_oscillators(&ui_oscillators));
    });
}

fn set_oscillator_parameter2(
    ui_weak_thread: &Weak<AccidentalSynth>,
    oscillator_values: &Arc<Mutex<Vec<UIOscillator>>>,
    oscillator_index: i32,
    value: f32,
) {
    let mut oscillators = oscillator_values
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    oscillators[oscillator_index as usize].parameter2 = value;

    let ui_oscillators = oscillators.clone();
    let _ = ui_weak_thread.upgrade_in_event_loop(move |ui| {
        ui.set_oscillators(slint_oscillators_from_oscillators(&ui_oscillators));
    });
}

fn set_lfo_frequency(
    ui_weak_thread: &Weak<AccidentalSynth>,
    lfo_index: LFOIndex,
    lfo_values: &Arc<Mutex<UILfo>>,
    normal_value: f32,
) {
    let mut lfo_values = lfo_values
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    lfo_values.frequency = normal_value;

    let ui_lfo_values = lfo_values.clone();
    let _ = ui_weak_thread.upgrade_in_event_loop(move |ui| match lfo_index {
        LFOIndex::ModWheel => ui.set_mod_wheel_lfo_values(slint_lfo_from_ui_lfo(&ui_lfo_values)),
        LFOIndex::Filter => ui.set_filter_lfo_values(slint_lfo_from_ui_lfo(&ui_lfo_values)),
    });
}

fn set_lfo_frequency_display(
    ui_weak_thread: &Weak<AccidentalSynth>,
    lfo_index: LFOIndex,
    frequency: f32,
) {
    let _ = ui_weak_thread.upgrade_in_event_loop(move |ui| match lfo_index {
        LFOIndex::ModWheel => ui.set_mod_wheel_lfo_frequency_display(frequency),
        LFOIndex::Filter => ui.set_filter_lfo_frequency_display(frequency),
    });
}

fn set_lfo_wave_shape(
    ui_weak_thread: &Weak<AccidentalSynth>,
    lfo_index: LFOIndex,
    lfo_values: &Arc<Mutex<UILfo>>,
    normal_value: f32,
) {
    let mut lfo_values = lfo_values
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    lfo_values.wave_shape_index = normal_value_to_wave_shape_index(normal_value) as i32;

    let ui_lfo_values = lfo_values.clone();
    let _ = ui_weak_thread.upgrade_in_event_loop(move |ui| match lfo_index {
        LFOIndex::ModWheel => ui.set_mod_wheel_lfo_values(slint_lfo_from_ui_lfo(&ui_lfo_values)),
        LFOIndex::Filter => ui.set_filter_lfo_values(slint_lfo_from_ui_lfo(&ui_lfo_values)),
    });
}

fn set_lfo_phase(
    ui_weak_thread: &Weak<AccidentalSynth>,
    lfo_index: LFOIndex,
    lfo_values: &Arc<Mutex<UILfo>>,
    normal_value: f32,
) {
    let mut lfo_values = lfo_values
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    lfo_values.phase = normal_value;

    let ui_lfo_values = lfo_values.clone();
    let _ = ui_weak_thread.upgrade_in_event_loop(move |ui| match lfo_index {
        LFOIndex::ModWheel => ui.set_mod_wheel_lfo_values(slint_lfo_from_ui_lfo(&ui_lfo_values)),
        LFOIndex::Filter => ui.set_filter_lfo_values(slint_lfo_from_ui_lfo(&ui_lfo_values)),
    });
}

fn set_lfo_phase_display(
    ui_weak_thread: &Weak<AccidentalSynth>,
    lfo_index: LFOIndex,
    degrees: i32,
) {
    let _ = ui_weak_thread.upgrade_in_event_loop(move |ui| match lfo_index {
        LFOIndex::ModWheel => ui.set_mod_wheel_lfo_phase_display(degrees),
        LFOIndex::Filter => ui.set_filter_lfo_phase_display(degrees),
    });
}

fn set_envelope_stage_value(
    ui_weak_thread: &Weak<AccidentalSynth>,
    envelope_index: EnvelopeIndex,
    stage: EnvelopeStage,
    envelope_values: &Arc<Mutex<UIEnvelope>>,
    normal_value: f32,
) {
    let mut envelope_values = envelope_values
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);

    match stage {
        EnvelopeStage::Attack => {
            envelope_values.attack = normal_value;
        }
        EnvelopeStage::Decay => {
            envelope_values.decay = normal_value;
        }
        EnvelopeStage::Sustain => {
            envelope_values.sustain = normal_value;
        }
        EnvelopeStage::Release => {
            envelope_values.release = normal_value;
        }
    }

    let ui_envelope_values = envelope_values.clone();
    let _ = ui_weak_thread.upgrade_in_event_loop(move |ui| match envelope_index {
        EnvelopeIndex::Amp => {
            ui.set_amp_eg_values(slint_envelope_from_ui_envelope(&ui_envelope_values))
        }
        EnvelopeIndex::Filter => {
            ui.set_filter_eg_values(slint_envelope_from_ui_envelope(&ui_envelope_values))
        }
    });
}

fn set_envelope_inverted(
    ui_weak_thread: &Weak<AccidentalSynth>,
    envelope_index: EnvelopeIndex,
    envelope_values: &Arc<Mutex<UIEnvelope>>,
    normal_value: f32,
) {
    let mut envelope_values = envelope_values
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);

    envelope_values.inverted = normal_value_to_bool(normal_value);

    let ui_envelope_values = envelope_values.clone();
    let _ = ui_weak_thread.upgrade_in_event_loop(move |ui| match envelope_index {
        EnvelopeIndex::Amp => {
            ui.set_amp_eg_values(slint_envelope_from_ui_envelope(&ui_envelope_values))
        }
        EnvelopeIndex::Filter => {
            ui.set_filter_eg_values(slint_envelope_from_ui_envelope(&ui_envelope_values))
        }
    });
}

fn slint_oscillators_from_oscillators(oscillator_values: &[UIOscillator]) -> ModelRc<Oscillator> {
    let oscillators: VecModel<Oscillator> = oscillator_values
        .iter()
        .map(|osc| Oscillator {
            wave_shape_index: osc.wave_shape_index,
            fine_tune: osc.fine_tune,
            course_tune: osc.course_tune,
            clipper_boost: osc.clipper_boost,
            parameter1: osc.parameter1,
            parameter2: osc.parameter2,
        })
        .collect();

    ModelRc::from(Rc::new(oscillators))
}

fn slint_audio_device_from_ui_audio_device(audio_device_values: &UIAudioDevice) -> AudioDevice {
    AudioDevice {
        output_device_index: audio_device_values.output_device_index,
        left_channel_index: audio_device_values.left_channel_index,
        right_channel_index: audio_device_values.right_channel_index,
        sample_rate_index: audio_device_values.sample_rate_index,
        output_devices: vec_to_model_rc_shared_string(&audio_device_values.output_devices),
        left_channels: vec_to_model_rc_shared_string(&audio_device_values.left_channels),
        right_channels: vec_to_model_rc_shared_string(&audio_device_values.right_channels),
        sample_rates: vec_to_model_rc_shared_string(&audio_device_values.sample_rates),
    }
}

fn slint_envelope_from_ui_envelope(envelope_values: &UIEnvelope) -> EnvelopeValues {
    EnvelopeValues {
        attack: envelope_values.attack,
        decay: envelope_values.decay,
        sustain: envelope_values.sustain,
        release: envelope_values.release,
        inverted: envelope_values.inverted,
    }
}
fn slint_lfo_from_ui_lfo(lfo_values: &UILfo) -> LFOValues {
    LFOValues {
        frequency: lfo_values.frequency,
        phase: lfo_values.phase,
        wave_shape_index: lfo_values.wave_shape_index,
    }
}

fn slint_midi_port_from_ui_midi_port(midi_port_values: &UIMidiPort) -> MidiPort {
    MidiPort {
        input_ports: vec_to_model_rc_shared_string(&midi_port_values.input_ports),
        channels: vec_to_model_rc_shared_string(&midi_port_values.channels),
        input_port_index: midi_port_values.input_port_index,
        channel_index: midi_port_values.channel_index,
    }
}

fn vec_to_model_rc_shared_string(input_values: &[String]) -> ModelRc<SharedString> {
    ModelRc::new(VecModel::from(
        input_values
            .iter()
            .map(SharedString::from)
            .collect::<Vec<SharedString>>(),
    ))
}

fn vec_to_model_rc_int(input_values: &Vec<i32>) -> ModelRc<i32> {
    ModelRc::new(VecModel::from(input_values.to_owned()))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crossbeam_channel::internal::SelectHandle;

    #[test]
    fn new_returns_correct_object_contents() {
        let ui = UI::new();
        let ui_update_sender = ui.get_ui_update_sender();
        assert!(ui_update_sender.is_ready());
    }
}
