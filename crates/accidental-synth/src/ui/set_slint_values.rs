use crate::ui::constants::{
    AUDIO_DEVICE_CHANNEL_INDEX_TO_NAME_OFFSET, AUDIO_DEVICE_CHANNEL_NULL_VALUE,
    MIDI_SCREEN_TOTAL_SLOTS, MONO_CHANNEL_COUNT,
};
use crate::ui::structs::{
    UIAudioDevice, UIEnvelope, UIFilterCutoff, UIFilterOptions, UIGlobalOptions, UILfo, UIMidiPort,
    UIMixer, UIOscillator,
};
use crate::ui::{
    slint_patches_list_from_ui_patches_list, slint_patches_save_status_from_ui_patch_save_status,
};
use crate::{AccidentalSynth, Mixer, ui};
use accsyn_core::effects::EffectParameters;
use accsyn_core::synth_events::{EnvelopeIndex, LFOIndex};
use accsyn_core::ui_events::EnvelopeStage;
use accsyn_engine::synthesizer::midi_value_converters::normal_value_to_bool;
use slint::{ModelRc, VecModel, Weak};
use std::rc::Rc;

pub fn set_midi_screen_values(
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
        ui.set_midi_display_values(ui::vec_to_model_rc_shared_string(&ui_midi_screen_values));
    });
}

pub fn set_midi_port_values(
    ui_weak_thread: &Weak<AccidentalSynth>,
    midi_port_values: &mut UIMidiPort,
) {
    let ui_midi_ports = midi_port_values.clone();
    let _ = ui_weak_thread.upgrade_in_event_loop(move |ui| {
        ui.set_midi_port_values(ui::slint_midi_port_from_ui_midi_port(&ui_midi_ports));
    });
}

pub fn set_audio_device_values(
    ui_weak_thread: &Weak<AccidentalSynth>,
    audio_device_values: &mut UIAudioDevice,
) {
    let ui_audio_devices = audio_device_values.clone();
    let _ = ui_weak_thread.upgrade_in_event_loop(move |ui| {
        ui.set_audio_device_values(ui::slint_audio_device_from_ui_audio_device(
            &ui_audio_devices,
        ));
    });
}

pub fn set_audio_device_channel_list(
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
        ui.set_audio_device_values(ui::slint_audio_device_from_ui_audio_device(
            &ui_audio_devices,
        ));
    });
}

pub fn set_audio_device_channel_indexes(
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
        ui.set_audio_device_values(ui::slint_audio_device_from_ui_audio_device(
            &ui_audio_devices,
        ));
    });
}

pub fn set_oscillator_fine_tune_display(
    ui_weak_thread: &Weak<AccidentalSynth>,
    oscillator_fine_tune_values: &mut [i32],
    oscillator_index: i32,
    cents: i32,
) {
    // oscillator_index is a Slint UI-sourced index, always non-negative and bounded by array size
    #[allow(clippy::cast_sign_loss)]
    let idx = oscillator_index as usize;
    oscillator_fine_tune_values[idx] = cents;
    let ui_oscillator_fine_tune_values = oscillator_fine_tune_values.to_vec();
    let _ = ui_weak_thread.upgrade_in_event_loop(move |ui| {
        ui.set_osc_fine_tune_cents(ui::vec_to_model_rc_int(&ui_oscillator_fine_tune_values));
    });
}

pub fn set_oscillator_values(
    ui_weak_thread: &Weak<AccidentalSynth>,
    oscillator_values: &mut [UIOscillator],
) {
    let ui_oscillators = oscillator_values.to_vec();
    let _ = ui_weak_thread.upgrade_in_event_loop(move |ui| {
        ui.set_oscillator_values(ui::slint_oscillators_from_oscillators(&ui_oscillators));
    });
}

pub fn set_lfo_values(
    ui_weak_thread: &Weak<AccidentalSynth>,
    lfo_index: LFOIndex,
    lfo_values: &mut UILfo,
) {
    let ui_lfo_values = lfo_values.clone();
    let _ = ui_weak_thread.upgrade_in_event_loop(move |ui| match lfo_index {
        LFOIndex::ModWheel => {
            ui.set_mod_wheel_lfo_values(ui::slint_lfo_from_ui_lfo(&ui_lfo_values));
        }
        LFOIndex::Filter => ui.set_filter_lfo_values(ui::slint_lfo_from_ui_lfo(&ui_lfo_values)),
    });
}

pub fn set_lfo_frequency_display(
    ui_weak_thread: &Weak<AccidentalSynth>,
    lfo_index: LFOIndex,
    frequency: f32,
) {
    let _ = ui_weak_thread.upgrade_in_event_loop(move |ui| match lfo_index {
        LFOIndex::ModWheel => ui.set_mod_wheel_lfo_frequency_display(frequency),
        LFOIndex::Filter => ui.set_filter_lfo_frequency_display(frequency),
    });
}

pub fn set_lfo_phase_display(
    ui_weak_thread: &Weak<AccidentalSynth>,
    lfo_index: LFOIndex,
    degrees: i32,
) {
    let _ = ui_weak_thread.upgrade_in_event_loop(move |ui| match lfo_index {
        LFOIndex::ModWheel => ui.set_mod_wheel_lfo_phase_display(degrees),
        LFOIndex::Filter => ui.set_filter_lfo_phase_display(degrees),
    });
}

pub fn set_envelope_stage_value(
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

    set_ui_envelope_values(ui_weak_thread, envelope_index, envelope_values);
}

pub fn set_envelope_inverted(
    ui_weak_thread: &Weak<AccidentalSynth>,
    envelope_index: EnvelopeIndex,
    envelope_values: &mut UIEnvelope,
    normal_value: f32,
) {
    envelope_values.inverted = normal_value_to_bool(normal_value);
    set_ui_envelope_values(ui_weak_thread, envelope_index, envelope_values);
}

fn set_ui_envelope_values(
    ui_weak_thread: &Weak<AccidentalSynth>,
    envelope_index: EnvelopeIndex,
    envelope_values: &mut UIEnvelope,
) {
    let ui_envelope_values = envelope_values.clone();
    let _ = ui_weak_thread.upgrade_in_event_loop(move |ui| match envelope_index {
        EnvelopeIndex::Amp => {
            ui.set_amp_envelope_values(ui::slint_envelope_from_ui_envelope(&ui_envelope_values));
        }
        EnvelopeIndex::Filter => {
            ui.set_filter_envelope_values(ui::slint_envelope_from_ui_envelope(&ui_envelope_values));
        }
        EnvelopeIndex::Pitch => {
            ui.set_pitch_envelope_values(ui::slint_envelope_from_ui_envelope(&ui_envelope_values));
        }
    });
}

pub fn set_filter_cutoff_values(
    ui_weak_thread: &Weak<AccidentalSynth>,
    filter_cutoff_values: &mut UIFilterCutoff,
) {
    let ui_filter_cutoff_values = filter_cutoff_values.clone();
    let _ = ui_weak_thread.upgrade_in_event_loop(move |ui| {
        ui.set_filter_cutoff_values(ui::slint_filter_cutoff_from_ui_filter_cutoff(
            &ui_filter_cutoff_values,
        ));
    });
}

pub fn set_filter_options_values(
    ui_weak_thread: &Weak<AccidentalSynth>,
    filter_option_values: &mut UIFilterOptions,
) {
    let ui_filter_option_values = filter_option_values.clone();
    let _ = ui_weak_thread.upgrade_in_event_loop(move |ui| {
        ui.set_filter_options_values(ui::slint_filter_options_from_ui_filter_options(
            &ui_filter_option_values,
        ));
    });
}

pub fn set_output_mixer_values(ui_weak_thread: &Weak<AccidentalSynth>, mixer_values: &UIMixer) {
    let ui_mixer_values = mixer_values.clone();
    let _ = ui_weak_thread.upgrade_in_event_loop(move |ui| {
        ui.set_output_mixer_values(ui::slint_mixer_from_ui_mixer_options(&ui_mixer_values));
    });
}

pub fn set_oscillator_mixer_values(
    ui_weak_thread: &Weak<AccidentalSynth>,
    mixer_values: &[UIMixer],
) {
    let oscillator_mixers: Vec<Mixer> = mixer_values
        .iter()
        .map(ui::slint_mixer_from_ui_mixer_options)
        .collect();

    let _ = ui_weak_thread.upgrade_in_event_loop(move |ui| {
        ui.set_oscillator_mixer_values(ModelRc::from(Rc::new(VecModel::from(oscillator_mixers))));
    });
}

pub fn set_global_options_values(
    ui_weak_thread: &Weak<AccidentalSynth>,
    global_option_values: &UIGlobalOptions,
) {
    let ui_global_option_values = *global_option_values;
    let _ = ui_weak_thread.upgrade_in_event_loop(move |ui| {
        ui.set_global_options_values(ui::slint_global_options_from_ui_global_options(
            &ui_global_option_values,
        ));
    });
}

pub fn set_effect_display(
    ui_weak_thread: &Weak<AccidentalSynth>,
    effect_values: &mut [EffectParameters],
    effect_index: i32,
    is_enabled: bool,
    parameters: Vec<f32>,
) {
    // effect_index is a Slint UI-sourced index, always non-negative and bounded by array size
    #[allow(clippy::cast_sign_loss)]
    let idx = effect_index as usize;
    effect_values[idx].is_enabled = is_enabled;
    effect_values[idx].parameters = parameters;

    let ui_effect_values = effect_values.to_vec();
    let _ = ui_weak_thread.upgrade_in_event_loop(move |ui| {
        ui.set_effects_values(ui::slint_effect_values_from_effect_parameters(
            &ui_effect_values,
        ));
    });
}

pub fn set_patch_list(ui_weak_thread: &Weak<AccidentalSynth>, patch_list: Vec<String>) {
    let _ = ui_weak_thread.upgrade_in_event_loop(move |ui| {
        ui.set_patch_list(slint_patches_list_from_ui_patches_list(&patch_list));
    });
}
pub fn set_user_patch_list(ui_weak_thread: &Weak<AccidentalSynth>, user_patch_list: Vec<String>) {
    let _ = ui_weak_thread.upgrade_in_event_loop(move |ui| {
        ui.set_user_patch_list(slint_patches_list_from_ui_patches_list(&user_patch_list));
    });
}

pub fn set_patch_save_status(ui_weak_thread: &Weak<AccidentalSynth>, save_status: (bool, String)) {
    let _ = ui_weak_thread.upgrade_in_event_loop(move |ui| {
        ui.set_patch_save_status(slint_patches_save_status_from_ui_patch_save_status(
            &save_status,
        ));
    });
}

pub fn set_patch_delete_status(
    ui_weak_thread: &Weak<AccidentalSynth>,
    save_status: (bool, String),
) {
    let _ = ui_weak_thread.upgrade_in_event_loop(move |ui| {
        ui.set_patch_delete_status(slint_patches_save_status_from_ui_patch_save_status(
            &save_status,
        ));
    });
}
