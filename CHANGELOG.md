# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.2.1] - 2026-04-24

### Fixed
- Envelope display bug: attack, decay, and release values were displayed on a linear scale but are stored via a two-segment exponential curve; sliders now correctly reflect loaded values
- `UIEnvelope` decay field was incorrectly reading release constants, causing decay and release to show the same value after patch load
- Wavefolder asymmetric mode: negative folding side never fired due to an incorrect paramater index check; asymmetric mode now applies independent 
  positive and negative folding as intended
- Wavefolder toggle buttons. The enable toggle was not correctly brining back the negative folding slider after being disable and reenabled. This now works as expected.
- LFO default range and all factory preset LFO values corrected to valid normalized range
- Several small formating and naming fixes


## [0.2.0] - 2026-04-21

### Added
- Complete user patch system: save, load, delete, and list patches; user patches persist to `~/Library/Application Support/AccidentalSynthesizer/patches/`
- Factory/system patches embedded at compile time and prefixed with `*` (e.g. `*1 - Init`) to distinguish them from user patches
- Patch numbers for stable ordering via MIDI program change (sorted by modification date)
- Type wrappers for module parameters enabling human-readable values in patch JSON files
- File size validation for patch loading (patches limited to 5 KB)
- Program change support for patch selection via MIDI
- Sustain pedal CC handling and sustain pedal button in the main UI for visual feedback and manual toggle
- Save feedback in the UI: success, failure, and file-already-exists messages shown after a patch save operation
- Patches and presets unified into a single patch list in the main window header
- Structured logging throughout `accsyn-engine` and `accsyn-midi` (all log macros include `target:` for log filtering)

### Fixed
- Security: path traversal vulnerability in patch saving
- Security: filename sanitization for patch save and load operations
- Security: replaced deprecated `std::env::home_dir()` with `dirs` crate
- Security: malformed or truncated MIDI messages no longer panic the MIDI input thread (added length guards)
- Security: MIDI note number out of valid range no longer panics the engine (index masked to 0–127)
- Security: removed `panic!` from CoreAudio `extern "C"` device callback — panicking across FFI boundary is undefined behaviour
- Security: MIDI connection reload failure no longer kills the device-management thread
- Pulse waveform x-coordinate rollover crash (unbounded growth exceeding type max)
- Oscillator x-position rollover with bounds check and rollover mechanism
- AM waveform regression where ring-mod parameter was not applied
- Filter state not resetting between patches when filter envelope or LFO was engaged
- Effects UI binding bug causing stale control states (declarative vs. imperative conflict)
- Wave shape parameter labels not updating correctly when switching patches
- Program change out-of-range event crashing the update event listener thread (`return` → `continue`)
- TOCTOU race in patch deletion — deletion is now by name, not index
- Filter modulation early-return bug preventing internal state reset on patch change
- Patch balance default corrected (0.0 → 0.5; center = normalized 0.5)
- Velocity curve using normalized instead of raw value during save/load

### Changed
- Polarity toggle relocated from oscillator section to Settings panel
- MIDI node display relocated to application header bar
- Visual improvements: window width, font selection, panel separation, colour scheme
- Replaced `.expect()` / `.unwrap()` panics on channel sends in engine, audio, and MIDI threads with graceful error logging
- Clippy pedantic cast warnings resolved across all crates
- CPAL pinned to v0.17.1 (v0.17.2+ broken on macOS)

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

[Unreleased]: https://gitlab.com/joltedbot-public/accidental-synth/-/compare/v0.2.1...HEAD
[0.2.1]: https://gitlab.com/joltedbot-public/accidental-synth/-/compare/v0.2.0...v0.2.1
[0.2.0]: https://gitlab.com/joltedbot-public/accidental-synth/-/compare/v0.1.4...v0.2.0
[0.1.4]: https://gitlab.com/joltedbot-public/accidental-synth/-/compare/v0.1.3...v0.1.4
[0.1.3]: https://gitlab.com/joltedbot-public/accidental-synth/-/compare/v0.1.2...v0.1.3
[0.1.2]: https://gitlab.com/joltedbot-public/accidental-synth/-/compare/v0.1.1...v0.1.2
[0.1.1]: https://gitlab.com/joltedbot-public/accidental-synth/-/compare/v0.1.0...v0.1.1
[0.1.0]: https://gitlab.com/joltedbot-public/accidental-synth/-/tags/v0.1.0
