mod callbacks;
mod constants;
mod structs;
mod update_listener;

use super::{
    AccidentalSynth, AudioDevice, EffectsValues, EnvelopeValues, FilterCutoff, FilterOptions,
    GlobalOptions, LFOValues, MidiPort, Mixer, Oscillator,
};
use crate::ui::callbacks::register_callbacks;
use crate::ui::constants::{
    AUDIO_DEVICE_CHANNEL_INDEX_TO_NAME_OFFSET, AUDIO_DEVICE_CHANNEL_NULL_VALUE, MIDI_CHANNEL_LIST,
    MIDI_SCREEN_TOTAL_SLOTS, MONO_CHANNEL_COUNT,
};
use crate::ui::structs::{
    UIAudioDevice, UIEnvelope, UIFilterCutoff, UIFilterOptions, UILfo, UIMidiPort, UIMixer,
    UIOscillator,
};
use crate::ui::update_listener::start_ui_update_listener;
use accsyn_engine::modules::lfo::DEFAULT_LFO_FREQUENCY;
use accsyn_engine::synthesizer::midi_value_converters::normal_value_to_bool;
use accsyn_midi::MidiDeviceUpdateEvents;
use accsyn_types::audio_events::AudioDeviceUpdateEvents;
use accsyn_types::defaults::Defaults;
use accsyn_types::effects::EffectParameters;
use accsyn_types::synth_events::{
    EnvelopeIndex, LFOIndex, OscillatorIndex, SynthesizerUpdateEvents,
};
use accsyn_types::ui_events::{EnvelopeStage, UIUpdates};
use anyhow::Result;
use crossbeam_channel::{Receiver, Sender, bounded};
use slint::{ModelRc, SharedString, VecModel, Weak};
use std::rc::Rc;
use std::sync::{Arc, Mutex};
use structs::UIGlobalOptions;
use strum::EnumCount;

const UI_UPDATE_CHANNEL_CAPACITY: usize = 10;

#[derive(Clone)]
struct ParameterValues {
    audio_device: UIAudioDevice,
    midi_port: UIMidiPort,
    oscillators: Vec<UIOscillator>,
    oscillator_fine_tune: Vec<i32>,
    amp_envelope: UIEnvelope,
    filter_envelope: UIEnvelope,
    mod_wheel_lfo: UILfo,
    filter_lfo: UILfo,
    filter_cutoff: UIFilterCutoff,
    filter_options: UIFilterOptions,
    output_mixer: UIMixer,
    oscillator_mixer: Vec<UIMixer>,
    global_options: UIGlobalOptions,
    midi_screen: Vec<String>,
    effects: Vec<EffectParameters>,
}

pub struct UI {
    ui_update_sender: Sender<UIUpdates>,
    ui_update_receiver: Receiver<UIUpdates>,
    parameter_values: Arc<Mutex<ParameterValues>>,
}

impl UI {
    pub fn new() -> Self {
        log::info!("Constructing UI Module");

        let (ui_update_sender, ui_update_receiver) = bounded(UI_UPDATE_CHANNEL_CAPACITY);

        let midi_port = UIMidiPort {
            channels: MIDI_CHANNEL_LIST.iter().map(ToString::to_string).collect(),
            ..Default::default()
        };

        let output_mixer = UIMixer {
            balance: Defaults::OUTPUT_MIXER_BALANCE,
            level: Defaults::OUTPUT_MIXER_LEVEL,
            is_muted: Defaults::OUTPUT_MIXER_IS_MUTED,
        };
        let oscillator_mixer = UIMixer {
            balance: Defaults::QUAD_MIXER_BALANCE,
            level: Defaults::QUAD_MIXER_LEVEL,
            is_muted: Defaults::QUAD_MIXER_IS_MUTED,
        };
        let mut oscillator_mixer = vec![oscillator_mixer; OscillatorIndex::COUNT];
        oscillator_mixer[OscillatorIndex::Sub as usize].level = Defaults::QUAD_MIXER_SUB_LEVEL;

        let parameter_values = ParameterValues {
            audio_device: UIAudioDevice::default(),
            midi_port,
            oscillators: vec![UIOscillator::default(); OscillatorIndex::COUNT],
            amp_envelope: UIEnvelope::default(),
            filter_envelope: UIEnvelope::default(),
            mod_wheel_lfo: UILfo::default(),
            filter_lfo: UILfo::default(),
            oscillator_fine_tune: vec![0; OscillatorIndex::COUNT],
            filter_cutoff: UIFilterCutoff::default(),
            filter_options: UIFilterOptions::default(),
            output_mixer,
            oscillator_mixer,
            midi_screen: Vec::with_capacity(MIDI_SCREEN_TOTAL_SLOTS),
            global_options: UIGlobalOptions::default(),
            effects: EffectParameters::default_all(),
        };

        Self {
            ui_update_sender,
            ui_update_receiver,
            parameter_values: Arc::new(Mutex::new(parameter_values)),
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
        set_ui_default_values(ui_weak, &self.parameter_values)?;

        start_ui_update_listener(ui_update_receiver, ui_weak, &self.parameter_values);

        Ok(())
    }
}
fn set_ui_default_values(
    ui_weak: &Weak<AccidentalSynth>,
    parameter_values: &Arc<Mutex<ParameterValues>>,
) -> Result<()> {
    let values = parameter_values
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);

    let ui_default_values = values.clone();

    ui_weak.upgrade_in_event_loop(move |ui| {
        ui.set_version(SharedString::from(env!("CARGO_PKG_VERSION")));

        ui.set_midi_port_values(slint_midi_port_from_ui_midi_port(
            &ui_default_values.midi_port,
        ));
        ui.set_audio_device_values(slint_audio_device_from_ui_audio_device(
            &ui_default_values.audio_device,
        ));
        ui.set_amp_envelope_values(slint_envelope_from_ui_envelope(
            &ui_default_values.amp_envelope,
        ));
        ui.set_filter_envelope_values(slint_envelope_from_ui_envelope(
            &ui_default_values.filter_envelope,
        ));

        ui.set_effects_values(slint_effect_values_from_effect_parameters(
            &ui_default_values.effects,
        ));
        dbg!(&ui_default_values.effects);
        ui.set_mod_wheel_lfo_values(slint_lfo_from_ui_lfo(&ui_default_values.mod_wheel_lfo));
        ui.set_filter_lfo_values(slint_lfo_from_ui_lfo(&ui_default_values.filter_lfo));
        ui.set_mod_wheel_lfo_frequency_display(DEFAULT_LFO_FREQUENCY);
        ui.set_filter_lfo_frequency_display(DEFAULT_LFO_FREQUENCY);
    })?;

    Ok(())
}

fn set_midi_screen_values(
    ui_weak_thread: &Weak<AccidentalSynth>,
    midi_screen_values: &mut Vec<String>,
    new_message: String,
) {
    if midi_screen_values.len() > MIDI_SCREEN_TOTAL_SLOTS {
        let target_index = midi_screen_values.len() - MIDI_SCREEN_TOTAL_SLOTS;
        midi_screen_values.drain(..target_index);
    }

    if midi_screen_values.len() == MIDI_SCREEN_TOTAL_SLOTS {
        _ = midi_screen_values.drain(0..1);
    }

    midi_screen_values.push(new_message);

    let ui_midi_screen_values = midi_screen_values.clone();
    let _ = ui_weak_thread.upgrade_in_event_loop(move |ui| {
        ui.set_midi_display_values(vec_to_model_rc_shared_string(&ui_midi_screen_values));
    });
}

fn set_midi_port_values(ui_weak_thread: &Weak<AccidentalSynth>, midi_port_values: &mut UIMidiPort) {
    let ui_midi_ports = midi_port_values.clone();
    let _ = ui_weak_thread.upgrade_in_event_loop(move |ui| {
        ui.set_midi_port_values(slint_midi_port_from_ui_midi_port(&ui_midi_ports));
    });
}

fn set_audio_device_values(
    ui_weak_thread: &Weak<AccidentalSynth>,
    audio_device_values: &mut UIAudioDevice,
) {
    let ui_audio_devices = audio_device_values.clone();
    let _ = ui_weak_thread.upgrade_in_event_loop(move |ui| {
        ui.set_audio_device_values(slint_audio_device_from_ui_audio_device(&ui_audio_devices));
    });
}

fn set_audio_device_channel_list(
    ui_weak_thread: &Weak<AccidentalSynth>,
    audio_device_values: &mut UIAudioDevice,
    channel_count: u16,
) {
    let mut device_channels: Vec<String> = vec![];
    for channel in 0..channel_count {
        device_channels.push((channel + AUDIO_DEVICE_CHANNEL_INDEX_TO_NAME_OFFSET).to_string());
    }

    audio_device_values
        .left_channels
        .clone_from(&device_channels);
    if channel_count > MONO_CHANNEL_COUNT {
        audio_device_values.right_channels = device_channels;
    } else {
        audio_device_values.right_channels = vec![];
    }

    let ui_audio_devices = audio_device_values.clone();
    let _ = ui_weak_thread.upgrade_in_event_loop(move |ui| {
        ui.set_audio_device_values(slint_audio_device_from_ui_audio_device(&ui_audio_devices));
    });
}

fn set_audio_device_channel_indexes(
    ui_weak_thread: &Weak<AccidentalSynth>,
    audio_device_values: &mut UIAudioDevice,
    left_chanel_index: i32,
    right_channel_index: i32,
) {
    audio_device_values.left_channel_index = left_chanel_index;

    if right_channel_index == AUDIO_DEVICE_CHANNEL_NULL_VALUE {
        audio_device_values.right_channel_index = AUDIO_DEVICE_CHANNEL_NULL_VALUE;
    } else {
        audio_device_values.right_channel_index = right_channel_index;
    }

    let ui_audio_devices = audio_device_values.clone();
    let _ = ui_weak_thread.upgrade_in_event_loop(move |ui| {
        ui.set_audio_device_values(slint_audio_device_from_ui_audio_device(&ui_audio_devices));
    });
}

fn set_oscillator_fine_tune_display(
    ui_weak_thread: &Weak<AccidentalSynth>,
    oscillator_fine_tune_values: &mut [i32],
    oscillator_index: i32,
    cents: i32,
) {
    oscillator_fine_tune_values[oscillator_index as usize] = cents;
    let ui_oscillator_fine_tune_values = oscillator_fine_tune_values.to_vec();
    let _ = ui_weak_thread.upgrade_in_event_loop(move |ui| {
        ui.set_osc_fine_tune_cents(vec_to_model_rc_int(&ui_oscillator_fine_tune_values));
    });
}

fn set_oscillator_values(
    ui_weak_thread: &Weak<AccidentalSynth>,
    oscillator_values: &mut [UIOscillator],
) {
    let ui_oscillators = oscillator_values.to_vec();
    let _ = ui_weak_thread.upgrade_in_event_loop(move |ui| {
        ui.set_oscillators(slint_oscillators_from_oscillators(&ui_oscillators));
    });
}

fn set_lfo_values(
    ui_weak_thread: &Weak<AccidentalSynth>,
    lfo_index: LFOIndex,
    lfo_values: &mut UILfo,
) {
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
    envelope_values: &mut UIEnvelope,
    normal_value: f32,
) {
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
            ui.set_amp_envelope_values(slint_envelope_from_ui_envelope(&ui_envelope_values));
        }
        EnvelopeIndex::Filter => {
            ui.set_filter_envelope_values(slint_envelope_from_ui_envelope(&ui_envelope_values));
        }
    });
}

fn set_envelope_inverted(
    ui_weak_thread: &Weak<AccidentalSynth>,
    envelope_index: EnvelopeIndex,
    envelope_values: &mut UIEnvelope,
    normal_value: f32,
) {
    envelope_values.inverted = normal_value_to_bool(normal_value);

    let ui_envelope_values = envelope_values.clone();
    let _ = ui_weak_thread.upgrade_in_event_loop(move |ui| match envelope_index {
        EnvelopeIndex::Amp => {
            ui.set_amp_envelope_values(slint_envelope_from_ui_envelope(&ui_envelope_values));
        }
        EnvelopeIndex::Filter => {
            ui.set_filter_envelope_values(slint_envelope_from_ui_envelope(&ui_envelope_values));
        }
    });
}

fn set_filter_cutoff_values(
    ui_weak_thread: &Weak<AccidentalSynth>,
    filter_cutoff_values: &mut UIFilterCutoff,
) {
    let ui_filter_cutoff_values = filter_cutoff_values.clone();
    let _ = ui_weak_thread.upgrade_in_event_loop(move |ui| {
        ui.set_filter_cutoff_values(slint_filter_cutoff_from_ui_filter_cutoff(
            &ui_filter_cutoff_values,
        ));
    });
}

fn set_filter_options_values(
    ui_weak_thread: &Weak<AccidentalSynth>,
    filter_option_values: &mut UIFilterOptions,
) {
    let ui_filter_option_values = filter_option_values.clone();
    let _ = ui_weak_thread.upgrade_in_event_loop(move |ui| {
        ui.set_filter_options_values(slint_filter_options_from_ui_filter_options(
            &ui_filter_option_values,
        ));
    });
}

fn set_output_mixer_values(ui_weak_thread: &Weak<AccidentalSynth>, mixer_values: &UIMixer) {
    let ui_mixer_values = mixer_values.clone();
    let _ = ui_weak_thread.upgrade_in_event_loop(move |ui| {
        ui.set_output_mixer_values(slint_mixer_from_ui_mixer_options(&ui_mixer_values));
    });
}

fn set_oscillator_mixer_values(ui_weak_thread: &Weak<AccidentalSynth>, mixer_values: &[UIMixer]) {
    let oscillator_mixers: Vec<Mixer> = mixer_values
        .iter()
        .map(slint_mixer_from_ui_mixer_options)
        .collect();

    let _ = ui_weak_thread.upgrade_in_event_loop(move |ui| {
        ui.set_oscillator_mixer_values(ModelRc::from(Rc::new(VecModel::from(oscillator_mixers))));
    });
}

fn set_global_options_values(
    ui_weak_thread: &Weak<AccidentalSynth>,
    global_option_values: &UIGlobalOptions,
) {
    let ui_global_option_values = *global_option_values;
    let _ = ui_weak_thread.upgrade_in_event_loop(move |ui| {
        ui.set_global_options_values(slint_global_options_from_ui_global_options(
            &ui_global_option_values,
        ));
    });
}

fn set_effect_display(
    ui_weak_thread: &Weak<AccidentalSynth>,
    effect_values: &mut [EffectParameters],
    effect_index: i32,
    is_enabled: bool,
    parameters: Vec<f32>,
) {
    effect_values[effect_index as usize] = EffectParameters {
        is_enabled,
        parameters,
    };
    let ui_effect_values = effect_values.to_vec();
    let _ = ui_weak_thread.upgrade_in_event_loop(move |ui| {
        ui.set_effects_values(slint_effect_values_from_effect_parameters(
            &ui_effect_values,
        ));
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
        buffer_size_index: audio_device_values.buffer_size_index,
        output_devices: vec_to_model_rc_shared_string(&audio_device_values.output_devices),
        left_channels: vec_to_model_rc_shared_string(&audio_device_values.left_channels),
        right_channels: vec_to_model_rc_shared_string(&audio_device_values.right_channels),
        sample_rates: vec_to_model_rc_shared_string(&audio_device_values.sample_rates),
        buffer_sizes: vec_to_model_rc_shared_string(&audio_device_values.buffer_sizes),
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

fn slint_filter_cutoff_from_ui_filter_cutoff(
    filter_cutoff_values: &UIFilterCutoff,
) -> FilterCutoff {
    FilterCutoff {
        cutoff: filter_cutoff_values.cutoff,
        resonance: filter_cutoff_values.resonance,
    }
}

fn slint_filter_options_from_ui_filter_options(
    filter_option_values: &UIFilterOptions,
) -> FilterOptions {
    FilterOptions {
        poles: filter_option_values.poles,
        key_track: filter_option_values.key_track,
        envelope_amount: filter_option_values.envelope_amount,
        lfo_amount: filter_option_values.lfo_amount,
    }
}

fn slint_mixer_from_ui_mixer_options(mixer_values: &UIMixer) -> Mixer {
    Mixer {
        balance: mixer_values.balance,
        level: mixer_values.level,
        is_muted: mixer_values.is_muted,
    }
}

fn slint_global_options_from_ui_global_options(
    global_option_values: &UIGlobalOptions,
) -> GlobalOptions {
    GlobalOptions {
        portamento_time: global_option_values.portamento_time,
        portamento_is_enabled: global_option_values.portamento_is_enabled,
        pitch_bend_range: global_option_values.pitch_bend_range,
        velocity_curve_slope: global_option_values.velocity_curve_slope,
        hard_sync_is_enabled: global_option_values.hard_sync_is_enabled,
        key_sync_is_enabled: global_option_values.key_sync_is_enabled,
        polarity_is_flipped: global_option_values.polarity_is_flipped,
    }
}

fn slint_effect_values_from_effect_parameters(
    input_values: &[EffectParameters],
) -> ModelRc<EffectsValues> {
    ModelRc::new(VecModel::from(
        input_values
            .iter()
            .map(|effect| EffectsValues {
                is_enabled: effect.is_enabled,
                parameters: ModelRc::new(VecModel::from(effect.parameters.clone())),
            })
            .collect::<Vec<EffectsValues>>(),
    ))
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
