use accsyn_engine::modules::effects::AudioEffectParameters;
use accsyn_engine::synthesizer::ModuleParameters;
use accsyn_engine::synthesizer::patches::system_patches;

/// Applies `preset` onto a freshly defaulted `ModuleParameters` the same way
/// `set_module_parameters_from_preset` does in production, using each module's public
/// `assign_from`. `clock` is intentionally left untouched — the real preset-load path never
/// assigns it either, since clock/bpm is live MIDI-clock state, not a per-patch value.
///
/// `ModuleParameters::default()` gives an empty `effects` Vec (unlike a running `Synthesizer`,
/// whose live parameters are always sized from the init patch first) — it's resized here to
/// match `preset.effects` so the assign_from loop below actually copies every effect rather
/// than silently iterating zero elements.
fn apply_preset(preset: &ModuleParameters) -> ModuleParameters {
    let live = ModuleParameters {
        effects: preset
            .effects
            .iter()
            .map(|_| AudioEffectParameters::default())
            .collect(),
        ..Default::default()
    };
    live.filter.assign_from(&preset.filter);
    live.mixer.assign_from(&preset.mixer);
    live.keyboard.assign_from(&preset.keyboard);
    live.lfos
        .iter()
        .enumerate()
        .for_each(|(index, lfo)| lfo.assign_from(&preset.lfos[index]));
    live.envelopes
        .iter()
        .enumerate()
        .for_each(|(index, envelope)| envelope.assign_from(&preset.envelopes[index]));
    live.oscillators
        .iter()
        .enumerate()
        .for_each(|(index, oscillator)| oscillator.assign_from(&preset.oscillators[index]));
    live.effects.iter().enumerate().for_each(|(index, effect)| {
        if index < preset.effects.len() {
            effect.assign_from(&preset.effects[index]);
        }
    });
    live
}

/// Asserts every f32-backed field in `parameters` is finite. Written as an explicit field walk
/// (rather than a generic/reflective one) so a failure message points at the exact field.
fn assert_all_f32_fields_finite(parameters: &ModuleParameters, patch_name: &str) {
    assert!(
        parameters.filter.cutoff_frequency.load().is_finite(),
        "{patch_name}: filter.cutoff_frequency is not finite"
    );
    assert!(
        parameters.filter.resonance.load().is_finite(),
        "{patch_name}: filter.resonance is not finite"
    );
    assert!(
        parameters.filter.key_tracking_amount.load().is_finite(),
        "{patch_name}: filter.key_tracking_amount is not finite"
    );

    assert!(
        parameters.mixer.level.load().is_finite(),
        "{patch_name}: mixer.level is not finite"
    );
    assert!(
        parameters.mixer.balance.load().is_finite(),
        "{patch_name}: mixer.balance is not finite"
    );
    for (index, input) in parameters.mixer.quad_mixer_inputs.iter().enumerate() {
        assert!(
            input.level.load().is_finite(),
            "{patch_name}: mixer.quad_mixer_inputs[{index}].level is not finite"
        );
        assert!(
            input.balance.load().is_finite(),
            "{patch_name}: mixer.quad_mixer_inputs[{index}].balance is not finite"
        );
    }

    assert!(
        parameters.keyboard.velocity_curve.load().is_finite(),
        "{patch_name}: keyboard.velocity_curve is not finite"
    );

    for (index, lfo) in parameters.lfos.iter().enumerate() {
        assert!(
            lfo.frequency.load().is_finite(),
            "{patch_name}: lfos[{index}].frequency is not finite"
        );
        assert!(
            lfo.synced_frequency.load().is_finite(),
            "{patch_name}: lfos[{index}].synced_frequency is not finite"
        );
        assert!(
            lfo.center_value.load().is_finite(),
            "{patch_name}: lfos[{index}].center_value is not finite"
        );
        assert!(
            lfo.range.load().is_finite(),
            "{patch_name}: lfos[{index}].range is not finite"
        );
        assert!(
            lfo.phase.load().is_finite(),
            "{patch_name}: lfos[{index}].phase is not finite"
        );
    }

    for (index, envelope) in parameters.envelopes.iter().enumerate() {
        assert!(
            envelope.sustain_level.load().is_finite(),
            "{patch_name}: envelopes[{index}].sustain_level is not finite"
        );
        assert!(
            envelope.amount.load().is_finite(),
            "{patch_name}: envelopes[{index}].amount is not finite"
        );
    }

    for (index, oscillator) in parameters.oscillators.iter().enumerate() {
        assert!(
            oscillator.shape_parameter1.load().is_finite(),
            "{patch_name}: oscillators[{index}].shape_parameter1 is not finite"
        );
        assert!(
            oscillator.shape_parameter2.load().is_finite(),
            "{patch_name}: oscillators[{index}].shape_parameter2 is not finite"
        );
        assert!(
            oscillator.pitch_envelope_amount.load().is_finite(),
            "{patch_name}: oscillators[{index}].pitch_envelope_amount is not finite"
        );
    }

    for (effect_index, effect) in parameters.effects.iter().enumerate() {
        for (param_index, parameter) in effect.parameters.iter().enumerate() {
            assert!(
                parameter.load().is_finite(),
                "{patch_name}: effects[{effect_index}].parameters[{param_index}] is not finite"
            );
        }
    }
}

/// Every embedded factory patch must deserialize into `ModuleParameters` and apply via the real
/// `assign_from` path without panicking, and every resulting f32-backed field must be finite.
/// This is the exact path a real patch load takes (see `set_module_parameters_from_preset`),
/// exercised here without needing a running `Synthesizer` or audio thread.
#[test]
fn all_factory_patches_load_without_panicking_and_produce_finite_values() {
    for (name, content) in system_patches() {
        let preset: ModuleParameters = serde_json::from_str(content)
            .unwrap_or_else(|err| panic!("factory patch '{name}' failed to deserialize: {err}"));
        let live = apply_preset(&preset);
        assert_all_f32_fields_finite(&live, name);
    }
}
