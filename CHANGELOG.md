# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.1.4] - 2026-02-07

### Fixed
- Bumped the dependencies to the latest versions
- Fixed the defaults for the Autopan and Tremolo effects that didn't match the UI
- Reworked the inverted mode in the envelopes so that it now works as expected

### Changed
- Major restructured of the project from a single monolithic binary crate to several library crates and a binary crate for main and the UI.
- Reduced the release build time from 30 minutes to < 2 minutes with the restructure and turning the build paramaters

## [0.1.3] - 2026-02-03

### Added
- Saturation effect with multiple saturation modes
- Compressor-type effect
- Auto Pan effect
- Tremolo effect
- Delay effect

### Fixed
- Optimized delay buffer implementation using direct indexing vector
- Fixed buffer rollover crash in delay effect
- Fixed brittle tests in effects.rs

### Changed
- Updated dependencies to latest versions
- Updated screenshot in README with newest features

## [0.1.2] - 2026-01-11

### Added
- Midi Implementation markdown file in root folder
- A switch to invert the polarity of output to work around phase cancelation issues
- A new gate effect that acts like an inverted clipper

### Fixed
- Oscillator 3 was not responding to MIDI CC 18 to modulate its parameter 1.

### Changed
- Updated the dependencies to the latest versions
- Various code cleanup and refactoring improvements

## [0.1.1] - 2026-01-11

### Added
- Effects system with multiple audio effects:
  - Wave folder (optionally asymmetrical)
  - Clipper with notch option
  - Bit Shifter effect
  - Full-wave and half-wave rectifiers
- MIDI aftertouch (channel pressure) now controls oscillator clipper boost
- Strum crate for improved enum maintainability

### Fixed
- Oscillator wave shape switching issue that prevented switching from sine to triangle

### Changed
- Refactored LFO parameter handling to use an array of structs (matching oscillators and envelopes pattern)
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

