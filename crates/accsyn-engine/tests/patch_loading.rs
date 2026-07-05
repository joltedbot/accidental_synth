use accsyn_core::parameter_types::Hertz;
use accsyn_engine::modules::effects::AudioEffectParameters;
use accsyn_engine::modules::envelope::MAX_ATTACK_MILLISECONDS;
use accsyn_engine::modules::oscillator::WaveShape;
use accsyn_engine::synthesizer::ModuleParameters;
use accsyn_engine::synthesizer::patches::system_patches;
use std::sync::atomic::Ordering::Relaxed;
use std::time::Duration;

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

/// A hand-authored patch combining three out-of-range values in one file: an envelope attack
/// time far beyond `MAX_ATTACK_MILLISECONDS`, an LFO with `clock_synced: true` and
/// `thirty_second_notes: 0` (a division-by-zero input to the clock-sync frequency calculation
/// in `event_listener.rs`), and an oscillator `wave_shape_index` far beyond the last valid
/// `WaveShape` variant. None of these come from a real factory patch — this fixture exists to
/// exercise combinations the shipped patches don't.
const ADVERSARIAL_PATCH_JSON: &str = include_str!("fixtures/adversarial-patch.json");

#[test]
fn adversarial_patch_loads_without_panicking_and_produces_finite_values() {
    let preset: ModuleParameters = serde_json::from_str(ADVERSARIAL_PATCH_JSON)
        .expect("adversarial fixture patch failed to deserialize");
    let live = apply_preset(&preset);
    assert_all_f32_fields_finite(&live, "adversarial-patch");
}

/// The out-of-range envelope attack time in the adversarial fixture is clamped by
/// `EnvelopeParameters::assign_from` (see Stage 2 fix), not passed through raw.
#[test]
fn adversarial_patch_attack_ms_is_clamped_on_load() {
    let preset: ModuleParameters = serde_json::from_str(ADVERSARIAL_PATCH_JSON)
        .expect("adversarial fixture patch failed to deserialize");
    let live = apply_preset(&preset);

    assert_eq!(live.envelopes[0].attack_ms.load(), MAX_ATTACK_MILLISECONDS);
}

/// An out-of-range `wave_shape_index` (raw `AtomicU8`, unclamped by `assign_from` since any
/// u8 is a structurally valid store) must still convert safely: `WaveShape::from_index` falls
/// back to the default (`Sine`) rather than panicking. This is the same safety net that made
/// the `WaveShape::COUNT` clamp bug (see AGENTS.md) non-fatal, pinned here against regression.
#[test]
fn adversarial_patch_out_of_range_wave_shape_index_falls_back_to_default() {
    let preset: ModuleParameters = serde_json::from_str(ADVERSARIAL_PATCH_JSON)
        .expect("adversarial fixture patch failed to deserialize");
    let live = apply_preset(&preset);

    let stored_index = live.oscillators[0].wave_shape_index.load(Relaxed);
    assert_eq!(
        stored_index, 200,
        "fixture should still round-trip the raw out-of-range index"
    );
    assert_eq!(WaveShape::from_index(stored_index), WaveShape::default());
}

/// The adversarial fixture's second LFO has `clock_synced: true` and `thirty_second_notes: 0`.
/// The real clock-sync frequency calculation (`event_listener.rs`, private to this crate) is
/// `1.0 / (thirty_second_notes as f64 * note_duration.as_secs_f64())`, which is a
/// divide-by-zero here. That calculation isn't reachable from an external test, so this test
/// reproduces the same formula with the fixture's loaded value and feeds the result into
/// `Hertz`, the exact type `synced_frequency` is stored as in production — pinning that the
/// resulting `f64::INFINITY` gets sanitized to a finite value on `store`/`new`, rather than
/// propagating into the audio path.
#[test]
fn adversarial_patch_thirty_second_notes_zero_does_not_produce_non_finite_synced_frequency() {
    let preset: ModuleParameters = serde_json::from_str(ADVERSARIAL_PATCH_JSON)
        .expect("adversarial fixture patch failed to deserialize");
    let live = apply_preset(&preset);

    let thirty_second_notes = live.lfos[1].thirty_second_notes.load(Relaxed);
    assert_eq!(
        thirty_second_notes, 0,
        "fixture is expected to exercise the zero case"
    );

    let note_duration = Duration::from_millis(100);
    let new_period = f64::from(thirty_second_notes) * note_duration.as_secs_f64();
    let new_frequency = 1.0 / new_period;
    assert!(
        new_frequency.is_infinite(),
        "test setup assumption: this should be a divide-by-zero"
    );

    #[allow(clippy::cast_possible_truncation)]
    let sanitized = Hertz::new(new_frequency as f32);
    assert!(
        sanitized.load().is_finite(),
        "Hertz::new must sanitize non-finite input rather than storing it"
    );
}

/// Programmatic edge case: `NaN`/`Infinity` cannot be written as bare JSON tokens, so this
/// combination can't be expressed as a fixture file. Instead it's constructed directly by
/// storing those values onto a `ModuleParameters::default()` before applying it — this is the
/// shape a bug in code that *builds* parameters programmatically (rather than deserializing
/// them) would take, and confirms the same sanitize-on-store guarantee holds either way.
#[test]
fn programmatic_nan_and_infinity_are_sanitized_through_assign_from() {
    let preset = ModuleParameters::default();
    preset.oscillators[0].shape_parameter1.store(f32::NAN);
    preset.filter.cutoff_frequency.store(f32::INFINITY);
    preset.lfos[0].center_value.store(f32::NEG_INFINITY);

    let live = apply_preset(&preset);

    assert!(live.oscillators[0].shape_parameter1.load().is_finite());
    assert!(live.filter.cutoff_frequency.load().is_finite());
    assert!(live.lfos[0].center_value.load().is_finite());
}
