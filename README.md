# Accidental Synthesizer (AccSyn)

Standalone, native UI, four‑oscillator mono synthesizer written in Rust.

[![Pipeline Status](https://gitlab.com/joltedbot-public/accidental-synth/badges/main/pipeline.svg)](https://gitlab.com/joltedbot-public/accidental-synth/-/pipelines)
[![License: Apache-2.0](https://img.shields.io/badge/license-Apache%202.0-blue.svg)](LICENSE)
[![Latest Tag](https://img.shields.io/gitlab/v/tag/joltedbot-public/accidental-synth)](https://gitlab.com/joltedbot-public/accidental-synth/-/tags)

---

## Overview
Accidental Synthesizer (AccSyn) is a standalone four‑oscillator mono synth for macOS with a native UI.  

I am, perhaps foolishly, writing it in Rust, largely for my own education and amusement.

## Project Status
Accidental Synthesizer is a working, playable synthesizer. The synth voice, MIDI integration, audio output, and UI are all fully connected and functional.

## Features
Current:
- Four-oscillator mono synth voice with multiple waveforms (sine, saw, square, triangle, noise, pulse, supersaw, FM, AM)
- Resonant lowpass filter with key tracking, envelope, and LFO modulation
- Three ADSR envelopes (amplitude, pitch, and filter) and two LFOs for modulation (pitch via the mod wheel and filter)
- Audio effects: saturation, compressor, wave folder, clipper, gate, rectifier, bit shifter, delay, auto pan, and tremolo 
- Preset & user patch system: save, load, and delete user patches; factory patches loaded in the app
- MIDI input: Note On/Off, Velocity, Pitch Bend, Control Change, Channel Pressure, Sustain Pedal, Program Change (see implementation chart in the 
  Wiki)
- Virtual MIDI input device for DAW integration; omni mode or per-channel filtering
- Hot-swappable MIDI and audio devices
- Native macOS UI

## Quick Start

### Prerequisites
- Rust (stable, edition 2024). No specific toolchain version required.
- macOS (Apple Silicon) -- It might work on Intel macOS but I can't test that myself.
- No external system dependencies beyond those pulled via Cargo. 
- On macOS, the app is not code‑signed; you may need to allow running apps from unidentified developers in System Settings.

### Build
```bash
cargo build --release
```


### Run
```bash
./accidental-synth
```


## Usage
1. Connect your MIDI input device and ensure your audio output device is available. Both MIDI and audio devices are hot-swappable.
2. Run accidental-synth. By default, it will choose the first MIDI input and first audio output device it discovers. You can change them in the
   settings panel (gear icon, top right).
3. Play from a MIDI keyboard, sequencer, or arpeggiator.

### MIDI
- AccSyn also presents itself as a MIDI input device in CoreMIDI so you can use it directly from a DAW or other tools.
- It is basically fully controllable with midi CC
- It defaults to omni-channel mode but you can change that in the settings panel
- Supported messages: Note On/Off, Velocity, Pitch Bend, Control Change (see the [MIDI Implementation Chart](https://gitlab.com/joltedbot-public/accidental-synth/-/wikis/home)).

### Audio
- By default, the first CoreAudio output device is selected at startup. You can select other devices in the settings panel.
- Mono devices will use the left channel only and for devices with 2 or more channels you get independently selectable stereo channels.
- Audio devices are hot-swappable - the app will automatically detect when devices are connected or disconnected.

### CLI Commands
```
Usage: accidental-synth [OPTIONS]

Options:
  -h, --help      Print help
  -V, --version   Print version
```

## UI


Screenshot:

![AccSyn UI](crates/accidental-synth/ui/images/screenshot.png)


## License
Licensed under the Apache License, Version 2.0. See the [LICENSE](LICENSE) file for details.

## Links
- Repository: https://gitlab.com/joltedbot-public/accidental-synth
- Issues: https://gitlab.com/joltedbot-public/accidental-synth/-/issues
- Wiki & MIDI Implementation Chart: https://gitlab.com/joltedbot-public/accidental-synth/-/wikis/home
