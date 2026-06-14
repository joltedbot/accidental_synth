use accsyn_engine::modules::envelope::{Envelope, EnvelopeParameters, MIN_ATTACK_MILLISECONDS};
use accsyn_engine::modules::oscillator::{Oscillator, WaveShape};
use std::sync::atomic::Ordering::Relaxed;

/// Verifies that triggering a gate-on produces non-silent audio output.
///
/// Uses a 1ms attack so the envelope reaches full sustain by sample 48, giving
/// a clear non-zero signal well before the assertion window ends at sample 500.
#[test]
fn gate_on_produces_nonzero_audio() {
    let sample_rate = 48_000u32;

    let mut oscillator = Oscillator::new(sample_rate, WaveShape::Sine);
    oscillator.tune(69); // A4 (440 Hz)

    let params = EnvelopeParameters::default();
    params.attack_ms.store(MIN_ATTACK_MILLISECONDS); // 1 ms = 48 samples at 48 kHz
    params.gate_flag.store(1, Relaxed); // MidiGateEvent::GateOn

    let mut envelope = Envelope::new(sample_rate);
    envelope.set_parameters(&params);

    let sample_count = 500usize;
    let mut sum_sq = 0.0f32;
    for _ in 0..sample_count {
        envelope.check_gate(&params.gate_flag);
        let env_val = envelope.generate();
        let osc_sample = oscillator.generate(None, None);
        sum_sq += (osc_sample * env_val).powi(2);
    }

    // It is set to a fixed value above so it won't truncate
    #[allow(clippy::cast_precision_loss)]
    let rms = (sum_sq / sample_count as f32).sqrt();
    assert!(
        rms > 0.0,
        "Expected non-silent audio after gate on, got RMS={rms}"
    );
}
