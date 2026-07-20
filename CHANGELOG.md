# Changelog

This project has switched from Semantic Versioning to Calendar Versioning.

## Unreleased

### Added
- New Chorus effect (stereo chorus, or a crude Leslie-cabinet simulation at 100% wet blend), plus a Chorus Organ preset
- New Flanger effect
- New mdBook-based user manual, replacing the scattered markdown docs, including a block diagram of the signal flow
- Post-crush, pre-blend gain-cut slider for the BitCrusher effect
- Integration tests covering adversarial and malformed patch files (out-of-range values, divide-by-zero, NaN, and infinity)
- `cargo-deny` configuration for dependency license/advisory auditing
- `#[serde(default)]` applied to `ModuleParameters` and every nested patch parameter struct, so a field added in a future update no longer breaks loading of previously saved user patches
- New preset `Hard Bass Pulse`

### Fixed
- Security: `assign_from` now clamps deserialized envelope attack/decay/release values to their valid range, same as the UI-driven path, preventing malformed patches from loading out-of-range values
- Security: fixed a divide-by-zero when portamento time was set to 0 buffers
- Keyboard pitch bend range no longer falls back to an invalid value (0, outside the 2-12 semitone range) when missing from a patch; now defaults to 12 semitones, matching the Init patch
- Filter cutoff frequency no longer silently falls back to 0 Hz (a fully closed, silent filter) when missing from a patch; now defaults to 16800 Hz, matching the Init patch
- WaveShape indexing bug that prevented MIDI CC from properly addressing all wave shapes
- Autopan and Tremolo effects were stuck on the sine wave shape due to a normalized-value bug, including a related off-by-one indexing error and incorrect float-to-combobox-index denormalization
- Oscillator and effect index mismatches in factory patches introduced by the PM oscillator and Autopan/Tremolo wave shape changes
- Inverse envelope mode no longer runs away toward infinity instead of stopping at 1.0 when returning to the ground state
- Filter response to the inverse envelope now correctly runs from the base frequency up to the filter cutoff
- LFOs could not be set to a 0 range/amount
- Gate Clipper effect now behaves as an actual clipper and shares its controls with the Clipper effect
- Supersaw level and default voice blend were too subdued
- Various MIDI byte-handling issues
- Delay effect UI spacing issue
- Settings window was cutting off patch text fields
- The example "Saw Lead" patch was broken due to a missing clock parameter
- Inaccuracies in the patch format reference documentation
- Inconsistent naming and UI defaults for the PM oscillator
- All the presets that had mixer levels incorrectly set to 1.0002447 are no set to the max value 1.0

### Changed
- Default output mixer level raised from 0.5 to 0.8 to match the Init patch
- Renamed the Bit Shifter effect to BitCrusher (its correct name)
- Updated the default Mod Wheel LFO rate from 10 Hz to 2 Hz in the factory presets
- Updated and improved several factory presets, including renaming one for clarity and revising the Init patch's envelope settings
- Changed how the build number is generated so it's derived from the date rather than a git commit count/env var
- Updated dependencies, including unpinning Slint from 0.16.1 to 0.17.1 now that the combobox popup clipping regression is resolved
- Removed an "understand anything" tool output directory that had been accidentally checked into the repo

## 2026.07.04.1

### Added
- New `PM` Phase Modulation Oscillator 
- New `Phase Mod Bass Pulse` preset

### Fixed
- Negative pitch envelope values work again resolving a regression introduced in a previous update 
- LFO frequency display values now correctly show clock divisions rather than frequency when loading a patch with clock sync enabled
- Oscillator parameter display values now update properly when the wave shape is changed

### Changed
- Updated dependencies
- Updated the README screenshots to represent the current UI state
- Updated the CC number of the Mod Wheel LFO Key Sync from 115 to 110 so that it was next to the other ModWheel CC numbers
- Tweaked the global panel layout in the UI to take up the ugly extra space
- Added additional missing logging in various files to provide more debug and tracing coverage

## 2026.06.30.1

### Fixed
- Slint version rolled back from 1.17.0 to 1.16.1 because of a regression in how it renders combobox popups causing it to clip at the Window 
  boundary
- Fixed the noise that occurred when changing audio input channels while audio is playing due to samples still in the output frame
- Fixed a race condition in the midi crate that was causing stuck notes in some situation with multiple midi devices being played in the same midi network at the same time

## 2026.06.28.1

### Added
- MIDI clock sync for LFOs: LFO rates can lock to the incoming MIDI clock in 32nd-note intervals, phase-synced to the beat
- MIDI clock BPM display in the application header
- Key sync (MIDI note-on) feature for the LFOs, including MIDI CC control and UI updates
- Voice blend control for the supersaw oscillator, plus a detune control in the UI
- Blend control for the Bit Shifter, Wave Rectifier, and Compressor effects
- Display values for the Oscillator panel controls
- Display values for the Filter option panel controls
- New presets
- Integration tests verifying the engine starts up and generates sound on a MIDI gate-on command
- MIDI Transport Stop messages reset the midi clock BPM detection to 0

### Fixed
- Security: f32 values deserialized from patch files are now validated and sanitized, preventing a malicious patch from passing `f32::INFINITY` to the filter cutoff and resonance
- Security: potential use-after-free in the audio device stop function
- Security: unhandled error in the patch generation function
- Security: unchecked array access
- The f64-to-f32 clamping helper was cutting off negative sample values
- Phase precision issue in the oscillators and LFOs at the slowest BPM and LFO intervals (processing switched from 32-bit to 64-bit)
- Impossible default for the AM oscillator amount
- Default values for the supersaw detune and voice blend
- Wave-shape-specific oscillator parameters now receive proper defaults in the UI when the wave shape changes
- Regression that hid the units on the oscillator coarse and fine tune control labels
- Fine tune cents display value now updates correctly when a patch is loaded
- Aftertouch scaling so the boost is no longer weak when the base oscillator boost is set very low
- Audio output device list no longer incorrectly includes input devices
- Numerous Clippy pedantic casting warnings

### Changed
- Completed the migration from CPAL to coreaudio-rs, removing the unmaintained CPAL dependency; audio output now supports i8, i16, i24, i32, and f32 sample formats
- MIDI system message handling updated to properly handle clock and reset messages
- Increased the maximum patch file size used to filter out invalid JSON, allowing for formatting variation and headroom for new fields
- Updated PATCHES.md to document the new patch format (clock and LFO sync fields)
- Updated existing presets to incorporate the new defaults and key/clock sync fields
- Updated dependencies

## [2026.05.10.1] - 2026-05-10

### Added
- New `Broken` waveform oscillator
- 3 New presets

### Fixed
- The Wave Rectifier effect now correctly labels the half and full wave modes
- Updated the README with more accurate information and a second screenshot
- The Wave folder effect negative amount label now correctly follows the enabled state of the negative amount slider
- The pitch envelope now has correct logic for positive and negative envelope amounts and the inverted mode

### Changed
- Changed the release build time that is used for CalVer version numbers to local time rather than UTC time so the relase number and release date match
- Updated dependencies
- Started the move to coreaudio-rs for audio processing by cleaning up the audio crates device enumeration function
- The slider for pitch envelope per oscillator is now a bipolar slider allowing the new logic for negative envelope amounts to be applied



## [2026.05.07.1] - 2026-05-7

### Fixed
 - LFO default frequency value display now correctly reflects the patch file value
 - AM Oscillator Ring Mod Amount default is now set to AM by default rather than full ring mode by default
 - Full & Half wave rectifier effects were labeled backwards
 - Bumped some dependency versions
 - Various formatting and linting fixes

### Changed
 - Effect names are now shown in the patch files to make it easier to identify them if editing manually
 - Updated the supersaw to try and match the original detune spread better

## [2026.05.03.1] - 2026-05-03

### Added
- Pitch envelope: a single ADSR envelope with independent per-oscillator amount controls
- MIDI CC control for pitch envelope parameters and per-oscillator pitch envelope amounts

### Fixed
- ToggleSwitch UI: MIDI and patch changes now correctly update the toggle state after manual interaction
- Sustain pedal switch: completed missing MIDI and patch update connectivity
- Envelope display bug: attack, decay, and release values were displayed on a linear scale but are stored via a two-segment exponential curve; sliders now correctly reflect loaded values
- `UIEnvelope` decay field was incorrectly reading release constants, causing decay and release to show the same value after patch load
- Wavefolder asymmetric mode: negative folding side never fired due to an incorrect parameter index check; asymmetric mode now applies independent positive and negative folding as intended
- Wavefolder enable toggle was not correctly restoring the negative folding slider after being disabled and re-enabled
- LFO default range and all preset LFO values corrected to valid normalized range
- Several small formatting and naming fixes

### Changed
- Slider track size increased; global panel layout cleaned up
- Version scheme updated from SemVer to CalVer


## [0.2.0] - 2026-04-21

### Added
- Complete user patch system: save, load, delete, and list patches; user patches persist to `~/Library/Application Support/AccidentalSynthesizer/patches/`
- System patches embedded at compile time and prefixed with `*` (e.g. `*1 - Init`) to distinguish them from user patches
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
