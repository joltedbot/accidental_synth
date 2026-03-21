//! Shared types, constants, and utilities for the AccSyn synthesizer.
//!
//! This crate provides the common vocabulary used across all AccSyn crates,
//! including event enums, parameter types, default values, and math helpers.

#![warn(missing_docs)]

/// Audio device events and stream configuration types.
pub mod audio_events;
/// Default values and constants for synthesizer parameters.
pub mod defaults;
/// Audio effect trait, effect index enum, and effect parameter types.
pub mod effects;
/// Mathematical utility functions for DSP and parameter conversion.
pub mod math;
/// MIDI event types for note, control change, and pitch bend messages.
pub mod midi_events;
/// Atomic wrapper types for thread-safe synthesizer parameters.
pub mod parameter_types;
/// Synthesizer control events sent from the UI and MIDI subsystems.
pub mod synth_events;
/// UI update events sent from the engine, audio, and MIDI subsystems.
pub mod ui_events;
