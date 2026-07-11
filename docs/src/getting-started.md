# Getting Started

## Requirements

AccSyn is a macOS application. A MIDI keyboard or controller is recommended but not required — you can trigger notes via MIDI from a DAW or other software.

You can download the release binaries for Apple Silicon Mac's. They are unsigned so you will have to do the dance with Apple security setting to get it running. 

To build AccSyn yourself you will need to setup a rust development chain as per: [Rustup](https://rustup.rs/) or [Rust's Getting started Page](https://rust-lang.org/learn/get-started/)


## Installation

Build from source using Cargo:

```sh
cargo build --release
```

The compiled binary is at `target/release/accidental-synth`. You can run it directly or copy it somewhere in your shell path.

## First Launch

On first launch AccSyn creates its user patch directory at:

```
~/Library/Application Support/AccidentalSynthesizer/patches/
```

Any patches you save in the UI will live here. You can modify or even create your own patches from scratch in this folder. This directory is scanned when the application starts and your patches will appear at the end of the patches dropdown list at the top center of the application.

## Audio Setup

Open the **Settings** panel and select the **Audio Settings** section. Choose your output device, left/right channels, sample rate, and buffer size. Changes take effect immediately.

A lower buffer size gives less latency at the cost of higher CPU load. If you hear clicks or dropouts, increase the buffer size.

## MIDI Setup

AccSyn creates a virtual MIDI input port named **AccSyn MIDI Input** that appears in CoreMIDI automatically. Any DAW or MIDI software on your Mac can send MIDI to it.

You can also choose a hardware or software MIDI device to listen to for MIDI messages. In **Settings → MIDI Settings**, select your input port and optionally restrict AccSyn to a specific MIDI channel (it responds to all channels by default).

You can actually use both midi mechanisms at the same time. You might for example choose a device to listen to for MIDI clock but point a midi controller at the Virtual Port for MIDI notes and CC. This could cause weird issues if you get crazy with it but it has been useful for me. Both methods share the same channel number selected in the settings menu.

## Playing AccSyn

Once audio and MIDI are configured, you can use it like any monophonic soft synth. It responds to MIDI note on & off, CC, Mod Wheel, monophonic aftertouch/channel pressure, etc.  See the [MIDI Implementation](midi-implementation.md) section for all the details.

## Loading Patches

The patch dropdown in the header selects the active patch. Factory patches are listed first (prefixed with `*`), followed by your saved user patches in alphabetical order. You can send MIDI Program Control messages to change patches. There is only one Bank and it ignores it anyway so don't worry about it. The number next to the patch name in the Patches dropdown is the Program Control number to send for that patch.


## Interface Overview

The interface has two main tabs and a settings panel:

- **Synth tab** — oscillators, filter, envelopes, LFOs, mixer, and performance options
- **Effects tab** — all effects slots processed in series after the synth signal chain
- **Settings** — audio device, MIDI device, synth options, and patch management


See the [Controls](./controls.md) section for a full description of every control, and [Settings Menu](./settings-menu.md) for the settings panel.
