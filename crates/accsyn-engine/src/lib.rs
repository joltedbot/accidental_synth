//! DSP synthesis engine for the `AccSyn` synthesizer.
//!
//! Contains oscillators, filters, envelopes, LFOs, effects, and the main
//! synthesizer that ties them together for real-time audio generation.

#![warn(missing_docs)]

/// DSP synthesis modules: oscillator, filter, envelope, LFO, mixer, amplifier, and effects.
pub mod modules;
/// Synthesizer core: sample generation, parameter management, MIDI processing, and patches.
pub mod synthesizer;
