mod callbacks;
mod constants;
mod set_slint_values;
mod structs;
mod update_listener;

use super::{
    AccidentalSynth, AudioDevice, EffectsValues, EnvelopeValues, FilterCutoff, FilterOptions,
    GlobalOptions, LFOValues, MidiPort, Mixer, Oscillator,
};
use crate::ui::callbacks::register_callbacks;
use crate::ui::structs::{
    UIAudioDevice, UIEnvelope, UIFilterCutoff, UIFilterOptions, UILfo, UIMidiPort, UIMixer,
    UIOscillator,
};
use crate::ui::update_listener::start_ui_update_listener;
use accsyn_engine::modules::effects::AudioEffectParameters;
use accsyn_engine::modules::lfo::DEFAULT_LFO_FREQUENCY;
use accsyn_engine::modules::oscillator::OscillatorParameters;
use accsyn_engine::synthesizer::{ModuleParameters, QuadMixerInput};
use accsyn_midi::MidiDeviceUpdateEvents;
use accsyn_types::audio_events::AudioDeviceUpdateEvents;
use accsyn_types::defaults::Defaults;
use accsyn_types::effects::EffectParameters;
use accsyn_types::math::load_f32_from_atomic_u32;
use accsyn_types::synth_events::{
    EnvelopeIndex, LFOIndex, OscillatorIndex, SynthesizerUpdateEvents,
};
use accsyn_types::ui_events::UIUpdates;
use anyhow::Result;
use crossbeam_channel::{Receiver, Sender, bounded};
use slint::{ModelRc, SharedString, VecModel, Weak};
use std::rc::Rc;
use std::sync::atomic::Ordering::Relaxed;
use std::sync::{Arc, Mutex};
use structs::UIGlobalOptions;
use strum::EnumCount;

const UI_UPDATE_CHANNEL_CAPACITY: usize = 10;

#[derive(Clone, Debug)]
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
    pub fn new(parameters: Arc<ModuleParameters>) -> Self {
        log::info!("Constructing UI Module");

        let (ui_update_sender, ui_update_receiver) = bounded(UI_UPDATE_CHANNEL_CAPACITY);

        let oscillator_mixer = UIMixer {
            balance: Defaults::QUAD_MIXER_BALANCE,
            level: Defaults::QUAD_MIXER_LEVEL,
            is_muted: Defaults::QUAD_MIXER_IS_MUTED,
        };
        let mut oscillator_mixer = vec![oscillator_mixer; OscillatorIndex::COUNT];
        oscillator_mixer[OscillatorIndex::Sub as usize].level = Defaults::QUAD_MIXER_SUB_LEVEL;

        let parameter_values = init_ui_values_from_module_parameters(parameters);

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
        ui.set_mod_wheel_lfo_values(slint_lfo_from_ui_lfo(&ui_default_values.mod_wheel_lfo));
        ui.set_filter_lfo_values(slint_lfo_from_ui_lfo(&ui_default_values.filter_lfo));
        ui.set_mod_wheel_lfo_frequency_display(DEFAULT_LFO_FREQUENCY);
        ui.set_filter_lfo_frequency_display(DEFAULT_LFO_FREQUENCY);
        ui.set_oscillator_values(slint_oscillators_from_oscillators(
            &ui_default_values.oscillators,
        ));
    })?;

    Ok(())
}

fn init_ui_values_from_module_parameters(parameters: Arc<ModuleParameters>) -> ParameterValues {
    ParameterValues {
        audio_device: UIAudioDevice::default(),
        midi_port: UIMidiPort::default(),
        oscillators: oscillator_values_to_ui_oscillator_values(&parameters.oscillators),
        oscillator_fine_tune: oscillator_fine_tune_to_ui_oscillator_fine_tune(
            &parameters.oscillators,
        ),
        amp_envelope: UIEnvelope::from_synth_parameters(
            &parameters.envelopes[EnvelopeIndex::Amp as usize],
        ),
        filter_envelope: UIEnvelope::from_synth_parameters(
            &parameters.envelopes[EnvelopeIndex::Filter as usize],
        ),
        mod_wheel_lfo: UILfo::from_synth_parameters(&parameters.lfos[LFOIndex::ModWheel as usize]),
        filter_lfo: UILfo::from_synth_parameters(&parameters.lfos[LFOIndex::Filter as usize]),
        filter_cutoff: UIFilterCutoff::from_synth_parameters(&parameters.filter),
        filter_options: UIFilterOptions::from_synth_parameters(
            &parameters.filter,
            &parameters.envelopes[EnvelopeIndex::Filter as usize],
            &parameters.lfos[LFOIndex::Filter as usize],
        ),
        output_mixer: UIMixer::from_synth_parameters(&parameters.mixer),
        oscillator_mixer: oscillator_mixer_to_ui_oscillator_mixer(
            &parameters.mixer.quad_mixer_inputs,
        ),
        global_options: UIGlobalOptions::from_synth_parameters(
            &parameters.keyboard,
            &parameters.oscillators[0],
        ),
        midi_screen: Vec::new(),
        effects: synthesizer_effects_to_ui_effects(&parameters.effects),
    }
}

fn oscillator_values_to_ui_oscillator_values(
    oscillators: &[OscillatorParameters],
) -> Vec<UIOscillator> {
    let mut ui_oscillators: Vec<UIOscillator> = Vec::with_capacity(OscillatorIndex::COUNT);
    oscillators
        .iter()
        .for_each(|osc| ui_oscillators.push(UIOscillator::from_synth_parameters(osc)));

    ui_oscillators
}

fn oscillator_fine_tune_to_ui_oscillator_fine_tune(
    oscillators: &[OscillatorParameters],
) -> Vec<i32> {
    oscillators
        .iter()
        .map(|osc| osc.fine_tune.load(Relaxed) as i32)
        .collect()
}

fn synthesizer_effects_to_ui_effects(effects: &[AudioEffectParameters]) -> Vec<EffectParameters> {
    let mut ui_effects: Vec<EffectParameters> = Vec::new();
    effects.iter().for_each(|effect| {
        ui_effects.push(EffectParameters {
            is_enabled: effect.is_enabled.load(Relaxed),
            parameters: effect
                .parameters
                .iter()
                .map(load_f32_from_atomic_u32)
                .collect::<Vec<f32>>(),
        });
    });

    ui_effects
}

fn oscillator_mixer_to_ui_oscillator_mixer(quad_mixer: &[QuadMixerInput]) -> Vec<UIMixer> {
    let mut ui_quad_mixer_values: Vec<UIMixer> = Vec::new();
    quad_mixer.iter().for_each(|strip| {
        ui_quad_mixer_values.push(UIMixer {
            level: load_f32_from_atomic_u32(&strip.level),
            balance: load_f32_from_atomic_u32(&strip.balance),
            is_muted: strip.mute.load(Relaxed),
        })
    });
    ui_quad_mixer_values
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
        let synth_parameters = Arc::new(ModuleParameters::default());
        let ui = UI::new(synth_parameters);
        let ui_update_sender = ui.get_ui_update_sender();
        assert!(ui_update_sender.is_ready());
    }
}
