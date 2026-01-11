# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.1.1] - 2026-01-11

### Added
- Effects system with multiple audio effects:
  - Wave folder (optionally asymmetrical)
  - Clipper with notch option
  - Bitshifter effect
  - Full-wave and half-wave rectifiers
- MIDI aftertouch (channel pressure) now controls oscillator clipper boost
- Strum crate for improved enum maintainability

### Fixed
- Oscillator waveshape switching issue that prevented switching from sine to triangle

### Changed
- Refactored LFO parameter handling to use array of structs (matching oscillators and envelopes pattern)
- Various code cleanup and refactoring improvements

## [0.1.0] - Initial Release

### Added
- Four-oscillator mono synthesizer with multiple waveforms (sine, saw, square, triangle, noise, pulse, supersaw, FM, AM)
- Resonant lowpass filter with key tracking and envelope modulation
- ADSR envelopes for amplitude and filter
- Two LFOs for modulation
- Native UI built with Slint framework
- MIDI input support (Note On/Off, Velocity, Pitch Bend, Control Change, Channel Pressure)
- Virtual MIDI input device for DAW integration
- Hot-swappable MIDI and audio devices
- CoreAudio integration for macOS
- Settings panel for device selection and configuration
