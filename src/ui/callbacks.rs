mod effects;
mod filter;
mod global;
mod modulation;
mod oscillators;
mod settings;

use crate::AccidentalSynth;
use crate::audio::AudioDeviceUpdateEvents;
use crate::midi::MidiDeviceUpdateEvents;
use crate::synthesizer::SynthesizerUpdateEvents;
use crossbeam_channel::Sender;
use slint::Weak;

pub fn register_callbacks(
    ui_weak: &Weak<AccidentalSynth>,
    midi_update_sender: Sender<MidiDeviceUpdateEvents>,
    audio_output_device_sender: &Sender<AudioDeviceUpdateEvents>,
    synthesizer_update_sender: &Sender<SynthesizerUpdateEvents>,
) {
    settings::callback_midi_input_channel_changed(ui_weak, midi_update_sender.clone());
    settings::callback_midi_input_port_changed(ui_weak, midi_update_sender);
    settings::callback_audio_output_device_changed(ui_weak, audio_output_device_sender.clone());
    settings::callback_audio_output_left_channel_changed(
        ui_weak,
        audio_output_device_sender.clone(),
    );
    settings::callback_audio_output_right_channel_changed(
        ui_weak,
        audio_output_device_sender.clone(),
    );
    settings::callback_audio_sample_rate_changed(ui_weak, audio_output_device_sender.clone());
    settings::callback_audio_buffer_size_changed(ui_weak, audio_output_device_sender.clone());

    oscillators::callback_osc_oscillator_shape_changed(ui_weak, synthesizer_update_sender.clone());
    oscillators::callback_osc_course_tune_changed(ui_weak, synthesizer_update_sender.clone());
    oscillators::callback_osc_fine_tune_changed(ui_weak, synthesizer_update_sender.clone());
    oscillators::callback_osc_clipper_boost_changed(ui_weak, synthesizer_update_sender.clone());
    oscillators::callback_osc_parameter1_changed(ui_weak, synthesizer_update_sender.clone());
    oscillators::callback_osc_parameter2_changed(ui_weak, synthesizer_update_sender.clone());

    filter::callback_filter_cutoff_changed(ui_weak, synthesizer_update_sender.clone());
    filter::callback_filter_resonance_changed(ui_weak, synthesizer_update_sender.clone());
    filter::callback_filter_poles_changed(ui_weak, synthesizer_update_sender.clone());
    filter::callback_filter_key_tracking_changed(ui_weak, synthesizer_update_sender.clone());
    filter::callback_filter_envelope_amount_changed(ui_weak, synthesizer_update_sender.clone());
    filter::callback_filter_lfo_amount_changed(ui_weak, synthesizer_update_sender.clone());

    modulation::callback_envelope_attack_changed(ui_weak, synthesizer_update_sender.clone());
    modulation::callback_envelope_decay_changed(ui_weak, synthesizer_update_sender.clone());
    modulation::callback_envelope_sustain_changed(ui_weak, synthesizer_update_sender.clone());
    modulation::callback_envelope_release_changed(ui_weak, synthesizer_update_sender.clone());
    modulation::callback_envelope_invert_changed(ui_weak, synthesizer_update_sender.clone());
    modulation::callback_lfo_frequency_changed(ui_weak, synthesizer_update_sender.clone());
    modulation::callback_lfo_shape_changed(ui_weak, synthesizer_update_sender.clone());
    modulation::callback_lfo_phase_changed(ui_weak, synthesizer_update_sender.clone());
    modulation::callback_lfo_phase_reset(ui_weak, synthesizer_update_sender.clone());

    global::callback_portamento_enabled(ui_weak, synthesizer_update_sender.clone());
    global::callback_portamento_time_changed(ui_weak, synthesizer_update_sender.clone());
    global::callback_pitch_bend_range_changed(ui_weak, synthesizer_update_sender.clone());
    global::callback_velocity_curve_changed(ui_weak, synthesizer_update_sender.clone());
    global::callback_hard_sync_enabled(ui_weak, synthesizer_update_sender.clone());
    global::callback_key_sync_enabled(ui_weak, synthesizer_update_sender.clone());
    global::callback_polarity_flipped(ui_weak, synthesizer_update_sender.clone());
    global::callback_output_balance_update(ui_weak, synthesizer_update_sender.clone());
    global::callback_output_level_update(ui_weak, synthesizer_update_sender.clone());
    global::callback_output_mute_update(ui_weak, synthesizer_update_sender.clone());
    global::callback_osc_mixer_balance_update(ui_weak, synthesizer_update_sender.clone());
    global::callback_osc_mixer_level_update(ui_weak, synthesizer_update_sender.clone());
    global::callback_osc_mixer_mute_update(ui_weak, synthesizer_update_sender.clone());

    effects::callback_effect_enable(ui_weak, synthesizer_update_sender.clone());
    effects::callback_effect_parameter_changed(ui_weak, synthesizer_update_sender.clone());
}
