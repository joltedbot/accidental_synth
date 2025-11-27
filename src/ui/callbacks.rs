use crate::AccidentalSynth;
use crate::audio::AudioDeviceUpdateEvents;
use crate::midi::MidiDeviceUpdateEvents;
use crate::ui::constants::AUDIO_DEVICE_CHANNEL_NULL_VALUE;
use crossbeam_channel::Sender;
use slint::Weak;
use crate::synthesizer::SynthesizerUpdateEvents;

pub fn register_callbacks(
    ui_weak: &Weak<AccidentalSynth>,
    midi_update_sender: Sender<MidiDeviceUpdateEvents>,
    audio_output_device_sender: &Sender<AudioDeviceUpdateEvents>,
    synthesizer_update_sender: &Sender<SynthesizerUpdateEvents>,
) {
    callback_midi_input_channel_changed(ui_weak, midi_update_sender.clone());
    callback_midi_input_port_changed(ui_weak, midi_update_sender);
    callback_audio_output_device_changed(ui_weak, audio_output_device_sender.clone());
    callback_audio_output_left_channel_changed(ui_weak, audio_output_device_sender.clone());
    callback_audio_output_right_channel_changed(ui_weak, audio_output_device_sender.clone());
    callback_audio_sample_rate_changed(ui_weak);
    
    callback_osc_oscillator_shape_changed(ui_weak, synthesizer_update_sender.clone());
    callback_osc_course_tune_changed(ui_weak, synthesizer_update_sender.clone());
    callback_osc_fine_tune_changed(ui_weak, synthesizer_update_sender.clone());
    callback_osc_clipper_boost_changed(ui_weak, synthesizer_update_sender.clone());
    callback_osc_parameter1_changed(ui_weak, synthesizer_update_sender.clone());
    callback_osc_parameter2_changed(ui_weak, synthesizer_update_sender.clone());
    
    callback_filter_cutoff_changed(ui_weak, synthesizer_update_sender.clone());
    callback_filter_resonance_changed(ui_weak, synthesizer_update_sender.clone());
    callback_filter_poles_changed(ui_weak, synthesizer_update_sender.clone());
    callback_filter_key_tracking_changed(ui_weak, synthesizer_update_sender.clone());
    callback_filter_eg_amount_changed(ui_weak, synthesizer_update_sender.clone());
    callback_filter_lfo_amount_changed(ui_weak, synthesizer_update_sender.clone());
    callback_envelope_attack_changed(ui_weak, synthesizer_update_sender.clone());
    callback_envelope_decay_changed(ui_weak, synthesizer_update_sender.clone());
    callback_envelope_sustain_changed(ui_weak, synthesizer_update_sender.clone());
    callback_envelope_release_changed(ui_weak, synthesizer_update_sender.clone());
    callback_envelope_invert_changed(ui_weak, synthesizer_update_sender.clone());
    callback_lfo_frequency_changed(ui_weak, synthesizer_update_sender.clone());
    callback_lfo_shape_changed(ui_weak, synthesizer_update_sender.clone());
    callback_lfo_phase_changed(ui_weak, synthesizer_update_sender.clone());
    callback_lfo_phase_reset(ui_weak, synthesizer_update_sender.clone());
}

fn callback_midi_input_channel_changed(
    ui_weak: &Weak<AccidentalSynth>,
    midi_update_sender: Sender<MidiDeviceUpdateEvents>,
) {
    if let Some(ui) = ui_weak.upgrade() {
        ui.on_midi_input_channel_changed(move |channel| {
            midi_update_sender.send(MidiDeviceUpdateEvents::UIMidiInputChannelIndex(channel.to_string())).expect(
                "callback_midi_input_port_changed(): Could not send new midi port name update to the midi module. \
                Exiting. ",
            );
        });
    }
}

fn callback_midi_input_port_changed(
    ui_weak: &Weak<AccidentalSynth>,
    midi_update_sender: Sender<MidiDeviceUpdateEvents>,
) {
    if let Some(ui) = ui_weak.upgrade() {
        ui.on_midi_input_port_changed(move |port|{
            midi_update_sender.send(MidiDeviceUpdateEvents::UIMidiInputPort(port.to_string())).expect(
                "callback_midi_input_port_changed(): Could not send new midi port name update to the midi module. \
                Exiting. ",
            );
        });
    }
}

fn callback_audio_output_device_changed(
    ui_weak: &Weak<AccidentalSynth>,
    audio_output_device_sender: Sender<AudioDeviceUpdateEvents>,
) {
    if let Some(ui) = ui_weak.upgrade() {
        ui.on_audio_output_device_changed(move |device| {
            audio_output_device_sender.send(AudioDeviceUpdateEvents::UIOutputDevice(device.to_string())).expect(
                "callback_audio_output_device_changed(): Could not send new audio output device update to the audio module.Exiting.",
            );
        });
    }
}

fn callback_audio_output_left_channel_changed(
    ui_weak: &Weak<AccidentalSynth>,
    audio_output_device_sender: Sender<AudioDeviceUpdateEvents>,
) {
    if let Some(ui) = ui_weak.upgrade() {
        ui.on_audio_output_left_channel_changed(move |channel| {
            let channel_number = channel.parse::<i32>().unwrap_or(AUDIO_DEVICE_CHANNEL_NULL_VALUE);
            audio_output_device_sender.send(AudioDeviceUpdateEvents::UIOutputDeviceLeftChannel(channel_number)).expect(
                "callback_audio_output_device_changed(): Could not send new audio output device update to the audio module.Exiting.",
            );
        });
    }
}

fn callback_audio_output_right_channel_changed(
    ui_weak: &Weak<AccidentalSynth>,
    audio_output_device_sender: Sender<AudioDeviceUpdateEvents>,
) {
    if let Some(ui) = ui_weak.upgrade() {
        ui.on_audio_output_right_channel_changed(move |channel| {
            let channel_number = channel.parse::<i32>().unwrap_or(AUDIO_DEVICE_CHANNEL_NULL_VALUE);
            audio_output_device_sender.send(AudioDeviceUpdateEvents::UIOutputDeviceRightChannel(channel_number))
                .expect(
                "callback_audio_output_device_changed(): Could not send new audio output device update to the audio module.Exiting.",
            );
        });
    }
}

fn callback_audio_sample_rate_changed(ui_weak: &Weak<AccidentalSynth>) {
    if let Some(ui) = ui_weak.upgrade() {
        ui.on_audio_sample_rate_changed(|channel| {
            println!("Audio sample rate changed. {channel}");
        });
    }
}

fn callback_osc_oscillator_shape_changed(ui_weak: &Weak<AccidentalSynth>, synthesizer_update_sender: Sender<SynthesizerUpdateEvents>) {
    if let Some(ui) = ui_weak.upgrade() {
        ui.on_osc_wave_shape_changed(move |oscillator_index, shape_index| {
            synthesizer_update_sender.send(SynthesizerUpdateEvents::WaveShapeIndex(oscillator_index, shape_index)).expect
            ("callback_osc_oscillator_shape_changed(): Could not send new \
            synthesizer oscillator shape update to the synthesizer module.Exiting.");
        });
    }
}

fn callback_osc_course_tune_changed(ui_weak: &Weak<AccidentalSynth>, synthesizer_update_sender: Sender<SynthesizerUpdateEvents>) {
    if let Some(ui) = ui_weak.upgrade() {
        ui.on_osc_course_tune_changed(move |oscillator_index, course_tune| {
            synthesizer_update_sender.send(SynthesizerUpdateEvents::CourseTune(oscillator_index, course_tune)).expect
            ("callback_osc_course_tune_changed(): Could not send new \
            synthesizer oscillator course tune to the synthesizer module.Exiting.");
        });
    }
}

fn callback_osc_fine_tune_changed(ui_weak: &Weak<AccidentalSynth>, synthesizer_update_sender: Sender<SynthesizerUpdateEvents>) {
    if let Some(ui) = ui_weak.upgrade() {
        ui.on_osc_fine_tune_changed(move |oscillator_index, fine_tune| {
            synthesizer_update_sender.send(SynthesizerUpdateEvents::FineTune(oscillator_index, fine_tune)).expect
            ("callback_osc_fine_tune_changed(): Could not send new \
            synthesizer oscillator fine tune to the synthesizer module.Exiting.");
        });
    }
}

fn callback_osc_clipper_boost_changed(ui_weak: &Weak<AccidentalSynth>, synthesizer_update_sender:
Sender<SynthesizerUpdateEvents>) {
    if let Some(ui) = ui_weak.upgrade() {
        ui.on_osc_clipper_boost_changed(move |oscillator_index, boost| {
            synthesizer_update_sender.send(SynthesizerUpdateEvents::ClipperBoost(oscillator_index, boost)).expect
            ("callback_osc_clipper_boost_changed(): Could not send new \
            synthesizer oscillator clipper boost to the synthesizer module.Exiting.");
        });
    }
}

fn callback_osc_parameter1_changed(ui_weak: &Weak<AccidentalSynth>, synthesizer_update_sender: Sender<SynthesizerUpdateEvents>) {
    if let Some(ui) = ui_weak.upgrade() {
        ui.on_osc_parameter1_changed(move |oscillator_index, value| {
            synthesizer_update_sender.send(SynthesizerUpdateEvents::Parameter1(oscillator_index, value)).expect
            ("callback_osc_parameter1_changed(): Could not send new \
            synthesizer oscillator parameter1 to the synthesizer module.Exiting.");
        });
    }
}

fn callback_osc_parameter2_changed(ui_weak: &Weak<AccidentalSynth>, synthesizer_update_sender: Sender<SynthesizerUpdateEvents>) {
    if let Some(ui) = ui_weak.upgrade() {
        ui.on_osc_parameter2_changed(move |oscillator_index, value| {
            synthesizer_update_sender.send(SynthesizerUpdateEvents::Parameter2(oscillator_index, value)).expect
            ("callback_osc_parameter2_changed(): Could not send new \
            synthesizer oscillator parameter2 to the synthesizer module.Exiting.");
        });
    }
}

fn callback_filter_cutoff_changed(ui_weak: &Weak<AccidentalSynth>, synthesizer_update_sender: Sender<SynthesizerUpdateEvents>) {
    if let Some(ui) = ui_weak.upgrade() {
        ui.on_filter_cutoff_changed(move |cutoff| {
            synthesizer_update_sender.send(SynthesizerUpdateEvents::FilterCutoffFrequency(cutoff)).expect
            ("callback_filter_cutoff_changed(): Could not send new \
            synthesizer filter cutoff frequency to the synthesizer module.Exiting.");
        });
    }
}

fn callback_filter_resonance_changed(ui_weak: &Weak<AccidentalSynth>, synthesizer_update_sender: Sender<SynthesizerUpdateEvents>) {
    if let Some(ui) = ui_weak.upgrade() {
        ui.on_filter_resonance_changed(move |value| {
            synthesizer_update_sender.send(SynthesizerUpdateEvents::FilterResonance(value)).expect
            ("callback_filter_resonance_changed(): Could not send new \
            synthesizer filter resonance to the synthesizer module.Exiting.");
        });
    }
}

fn callback_filter_poles_changed(ui_weak: &Weak<AccidentalSynth>, synthesizer_update_sender: Sender<SynthesizerUpdateEvents>) {
    if let Some(ui) = ui_weak.upgrade() {
        ui.on_filter_poles_changed(move |value| {
            synthesizer_update_sender.send(SynthesizerUpdateEvents::FilterPoleCount(value)).expect
            ("callback_filter_poles_changed(): Could not send new \
            synthesizer filter pole count to the synthesizer module.Exiting.");
        });
    }
}

fn callback_filter_key_tracking_changed(ui_weak: &Weak<AccidentalSynth>, synthesizer_update_sender: Sender<SynthesizerUpdateEvents>) {
    if let Some(ui) = ui_weak.upgrade() {
        ui.on_filter_key_tracking_changed(move |value| {
            synthesizer_update_sender.send(SynthesizerUpdateEvents::FilterKeyTrackingAmount(value)).expect
            ("callback_filter_key_tracking_changed(): Could not send new \
            synthesizer filter key tracking amount to the synthesizer module.Exiting.");
        });
    }
}

fn callback_filter_eg_amount_changed(ui_weak: &Weak<AccidentalSynth>, synthesizer_update_sender: Sender<SynthesizerUpdateEvents>) {
    if let Some(ui) = ui_weak.upgrade() {
        ui.on_filter_eg_amount_changed(move |value| {
            synthesizer_update_sender.send(SynthesizerUpdateEvents::FilterEnvelopeAmount(value)).expect
            ("callback_filter_eg_amount_changed(): Could not send new \
            synthesizer filter envelope amount to the synthesizer module.Exiting.");
        });
    }
}

fn callback_filter_lfo_amount_changed(ui_weak: &Weak<AccidentalSynth>, synthesizer_update_sender: Sender<SynthesizerUpdateEvents>) {
    if let Some(ui) = ui_weak.upgrade() {
        ui.on_filter_lfo_amount_changed(move |value| {
            synthesizer_update_sender.send(SynthesizerUpdateEvents::FilterLfoAmount(value)).expect
            ("callback_filter_lfo_amount_changed(): Could not send new \
            synthesizer filter lfo amount to the synthesizer module.Exiting.");
        });
    }
}

fn callback_envelope_attack_changed(ui_weak: &Weak<AccidentalSynth>, synthesizer_update_sender: Sender<SynthesizerUpdateEvents>) {
    if let Some(ui) = ui_weak.upgrade() {
        ui.on_envelope_attack_changed(move |envelope_index, milliseconds| {
            synthesizer_update_sender.send(SynthesizerUpdateEvents::FilterEnvelopeAttack(envelope_index, milliseconds)).expect
            ("callback_filter_lfo_amount_changed(): Could not send new \
            envelope attack to the synthesizer module.Exiting.");
        });
    }
}

fn callback_envelope_decay_changed(ui_weak: &Weak<AccidentalSynth>, synthesizer_update_sender: Sender<SynthesizerUpdateEvents>) {
    if let Some(ui) = ui_weak.upgrade() {
        ui.on_envelope_decay_changed(move |envelope_index, milliseconds| {
            synthesizer_update_sender.send(SynthesizerUpdateEvents::FilterEnvelopeDecay(envelope_index, milliseconds)).expect
            ("callback_filter_lfo_amount_changed(): Could not send new \
            envelope decay to the synthesizer module.Exiting.");
        });
    }
}

fn callback_envelope_sustain_changed(ui_weak: &Weak<AccidentalSynth>, synthesizer_update_sender: Sender<SynthesizerUpdateEvents>) {
    if let Some(ui) = ui_weak.upgrade() {
        ui.on_envelope_sustain_changed(move |envelope_index, level| {
            synthesizer_update_sender.send(SynthesizerUpdateEvents::FilterEnvelopeSustain(envelope_index, level)).expect
            ("callback_filter_lfo_amount_changed(): Could not send new \
            envelope sustain to the synthesizer module.Exiting.");
        });
    }
}

fn callback_envelope_release_changed(ui_weak: &Weak<AccidentalSynth>, synthesizer_update_sender: Sender<SynthesizerUpdateEvents>) {
    if let Some(ui) = ui_weak.upgrade() {
        ui.on_envelope_release_changed(move |envelope_index, milliseconds| {
            synthesizer_update_sender.send(SynthesizerUpdateEvents::FilterEnvelopeRelease(envelope_index, milliseconds)).expect
            ("callback_filter_lfo_amount_changed(): Could not send new \
            envelope release to the synthesizer module.Exiting.");
        });
    }
}

fn callback_envelope_invert_changed(ui_weak: &Weak<AccidentalSynth>, synthesizer_update_sender: Sender<SynthesizerUpdateEvents>) {
    if let Some(ui) = ui_weak.upgrade() {
        ui.on_envelope_invert_changed(move |filter_index, is_inverted| {
            synthesizer_update_sender.send(SynthesizerUpdateEvents::FilterEnvelopeInvert(filter_index, is_inverted)).expect
            ("callback_filter_lfo_amount_changed(): Could not send new \
            envelope inverted state to the synthesizer module.Exiting.");
        });
    }
}

fn callback_lfo_frequency_changed(ui_weak: &Weak<AccidentalSynth>, synthesizer_update_sender:
Sender<SynthesizerUpdateEvents>) {
    if let Some(ui) = ui_weak.upgrade() {
        ui.on_lfo_frequency_changed(move |lfo_index, frequency| {
            synthesizer_update_sender.send(SynthesizerUpdateEvents::LfoFrequency(lfo_index, frequency)).expect
            ("callback_filter_lfo_amount_changed(): Could not send new \
            LFO Frequency to the synthesizer module.Exiting.");
        });
    }
}

fn callback_lfo_shape_changed(ui_weak: &Weak<AccidentalSynth>, synthesizer_update_sender:
Sender<SynthesizerUpdateEvents>) {
    if let Some(ui) = ui_weak.upgrade() {
        ui.on_lfo_shape_changed(move |lfo_index, shape_index| {
            synthesizer_update_sender.send(SynthesizerUpdateEvents::LfoShapeIndex(lfo_index, shape_index)).expect
            ("callback_filter_lfo_amount_changed(): Could not send new \
            LFO shape index to the synthesizer module.Exiting.");
        });
    }
}

fn callback_lfo_phase_changed(ui_weak: &Weak<AccidentalSynth>, synthesizer_update_sender:
Sender<SynthesizerUpdateEvents>) {
    if let Some(ui) = ui_weak.upgrade() {
        ui.on_lfo_phase_changed(move |lfo_index, phase| {
            synthesizer_update_sender.send(SynthesizerUpdateEvents::LfoPhase(lfo_index, phase)).expect
            ("callback_filter_lfo_amount_changed(): Could not send new \
            LFO phase to the synthesizer module.Exiting.");
        });
    }
}

fn callback_lfo_phase_reset(ui_weak: &Weak<AccidentalSynth>, synthesizer_update_sender:
Sender<SynthesizerUpdateEvents>) {
    if let Some(ui) = ui_weak.upgrade() {
        ui.on_lfo_phase_reset(move |lfo_index| {
            synthesizer_update_sender.send(SynthesizerUpdateEvents::LfoPhaseReset(lfo_index)).expect
            ("callback_filter_lfo_amount_changed(): Could not send  \
            LFO phase reset command to the synthesizer module.Exiting.");
        });
    }
}

