use accsyn_core::audio_events::OutputStreamParameters;
use accsyn_engine::synthesizer::Synthesizer;
use std::sync::Arc;
use std::sync::atomic::{AtomicU16, AtomicU32};

/// Verifies that `Synthesizer::new()` succeeds with default stream parameters.
///
/// Exercises the full construction path: embedded init-patch deserialization,
/// Application Support directory creation, and user patch discovery.
#[test]
fn new_returns_ok() {
    let params = OutputStreamParameters {
        sample_rate: Arc::new(AtomicU32::new(48_000)),
        buffer_size: Arc::new(AtomicU32::new(256)),
        channel_count: Arc::new(AtomicU16::new(2)),
    };
    let result = Synthesizer::new(params);
    assert!(
        result.is_ok(),
        "Synthesizer::new() failed: {:?}",
        result.err()
    );
}
