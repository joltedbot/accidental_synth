# MIDI Implementation Chart v2.0

**Manufacturer:** Dave White
**Model:** Accidental Synthesizer (AccSyn)
**Version:** 0.1.1
**Date:** 2026-01-11

---

### Basic Information, MIDI Timing & Synchronization, and Extensions Compatibility

| Function | Transmitted/Export | Recognized/Import | Remarks |
|----------|-------------------|-------------------|---------|
| **1. Basic Information** | | | |
| MIDI channels | No | 1-16 | Omni mode by default, configurable via UI |
| Note numbers | No | 0-127 | Full MIDI range |
| Program change | No | No | |
| Bank Select response? | No | No | |
| **Modes supported:** | | | |
| Mode 1: Omni-On, Poly | No | No | Mono synth |
| Mode 2: Omni-On, Mono | No | Yes | Default mode |
| Mode 3: Omni-Off, Poly | No | No | Mono synth |
| Mode 4: Omni-Off, Mono | No | Yes | User-selectable |
| Multi Mode | No | No | |
| Note-On Velocity | No | Yes | |
| Note-Off Velocity | No | No | |
| Channel Aftertouch | No | Yes | Controls oscillator clipper boost |
| Poly (Key) Aftertouch | No | No | |
| Pitch Bend | No | Yes | |
| Active Sensing | No | No | |
| System Reset | No | No | |
| Tune Request | No | No | |
| **Universal System Exclusive:** | | | |
| Sample Dump Standard | No | No | |
| Device Inquiry | No | No | |
| File Dump | No | No | |
| MIDI Tuning | No | No | |
| Master Volume | No | No | |
| Master Balance | No | No | |
| Notation Information | No | No | |
| Turn GM1 System On | No | No | |
| Turn GM2 System On | No | No | |
| Turn GM System Off | No | No | |
| DLS-1 | No | No | |
| File Reference | No | No | |
| Controller Destination | No | No | |
| Key-based Instrument Ctrl | No | No | |
| Master Fine/Coarse Tune | No | No | |
| Other Universal System Exclusive | No | No | |
| **Manufacturer or Non-Commercial System Exclusive** | No | No | |
| **NRPNs** | No | No | |
| **RPNs:** | | | |
| RPN 00 (Pitch Bend Sensitivity) | No | No | Use CC #5 instead |
| RPN 01 (Channel Fine Tune) | No | No | |
| RPN 02 (Channel Coarse Tune) | No | No | |
| RPN 03 (Tuning Program Select) | No | No | |
| RPN 04 (Tuning Bank Select) | No | No | |
| RPN 05 (Modulation Depth Range) | No | No | |
| **2. MIDI Timing and Synchronization** | | | |
| MIDI Clock | No | No | |
| Song Position Pointer | No | No | |
| Song Select | No | No | |
| Start | No | No | |
| Continue | No | No | |
| Stop | No | No | |
| MIDI Time Code | No | No | |
| MIDI Machine Control | No | No | |
| MIDI Show Control | No | No | |
| **3. Extensions Compatibility** | | | |
| General MIDI compatible? | No | No | Dedicated synthesizer |
| Is GM default power-up mode? | No | No | |
| DLS compatible? | No | No | |
| (DLS File Types) | No | No | |
| Standard MIDI Files | No | No | |
| XMF Files | No | No | |
| SP-MIDI compatible? | No | No | |

---

## Control Number Information

| Control # | Function | Transmitted (Y/N) | Recognized (Y/N) | Remarks |
|-----------|----------|-------------------|------------------|---------|
| 0 | Bank Select (MSB) | N | N | |
| 1 | Modulation Wheel (MSB) | N | Y | Controls LFO depth |
| 2 | Breath Controller (MSB) | N | N | |
| 3 | Velocity Curve | N | Y |  |
| 4 | Foot Controller (MSB) | N | N | |
| 5 | Pitch Bend Range | N | Y |  |
| 6 | Data Entry (MSB) | N | N | |
| 7 | Master Volume | N | Y | Channel Volume (MSB) |
| 8 | Mute | N | Y |  |
| 9 | | N | N | |
| 10 | Stereo Balance | N | Y | Pan (MSB) |
| 11 | Expression (MSB) | N | N | |
| 12 | Sub Oscillator Shape Parameter 1 | N | Y |  |
| 13 | Sub Oscillator Shape Parameter 2 | N | Y |  |
| 14 | Oscillator 1 Shape Parameter 1 | N | Y |  |
| 15 | Oscillator 1 Shape Parameter 2 | N | Y |  |
| 16 | Oscillator 2 Shape Parameter 1 | N | Y |  |
| 17 | Oscillator 2 Shape Parameter 2 | N | Y |  |
| 18 | Oscillator 3 Shape Parameter 1 | N | Y |  |
| 19 | Oscillator 3 Shape Parameter 2 | N | Y |  |
| 20 | Oscillator Key Sync Enabled | N | Y |  |
| 21 | | N | N | |
| 22 | | N | N | |
| 23 | | N | N | |
| 24 | | N | N | |
| 25 | | N | N | |
| 26 | | N | N | |
| 27 | | N | N | |
| 28 | | N | N | |
| 29 | | N | N | |
| 30 | | N | N | |
| 31 | | N | N | |
| 32 | Bank Select (LSB) | N | N | |
| 33 | Modulation Wheel (LSB) | N | N | |
| 34 | Breath Controller (LSB) | N | N | |
| 35 | | N | N | |
| 36 | Foot Controller (LSB) | N | N | |
| 37 | Portamento Time | N | Y | |
| 38 | Oscillator Hard Sync | N | Y |  |
| 39 | Channel Volume (LSB) | N | N | |
| 40 | Sub Oscillator Shape | N | Y |  |
| 41 | Oscillator 1 Shape | N | Y |  |
| 42 | Oscillator 2 Shape | N | Y |  |
| 43 | Oscillator 3 Shape | N | Y |  |
| 44 | Sub Oscillator Coarse Tune | N | Y |  |
| 45 | Oscillator 1 Coarse Tune | N | Y |  |
| 46 | Oscillator 2 Coarse Tune | N | Y |  |
| 47 | Oscillator 3 Coarse Tune | N | Y |  |
| 48 | Sub Oscillator Fine Tune | N | Y |  |
| 49 | Oscillator 1 Fine Tune | N | Y |  |
| 50 | Oscillator 2 Fine Tune | N | Y |  |
| 51 | Oscillator 3 Fine Tune | N | Y |  |
| 52 | Sub Oscillator Level | N | Y |  |
| 53 | Oscillator 1 Level | N | Y |  |
| 54 | Oscillator 2 Level | N | Y |  |
| 55 | Oscillator 3 Level | N | Y |  |
| 56 | Sub Oscillator Mute | N | Y |  |
| 57 | Oscillator 1 Mute | N | Y |  |
| 58 | Oscillator 2 Mute | N | Y |  |
| 59 | Oscillator 3 Mute | N | Y |  |
| 60 | Sub Oscillator Balance | N | Y |  |
| 61 | Oscillator 1 Balance | N | Y |  |
| 62 | Oscillator 2 Balance | N | Y |  |
| 63 | Oscillator 3 Balance | N | Y |  |
| 64 | Sustain Pedal | N | Y | |
| 65 | Portamento On/Off | N | Y | |
| 66 | Sub Oscillator Clip Boost | N | Y |  |
| 67 | Oscillator 1 Clip Boost | N | Y |  |
| 68 | Oscillator 2 Clip Boost | N | Y |  |
| 69 | Oscillator 3 Clip Boost | N | Y |  |
| 70 | Filter Poles | N | Y |  |
| 71 | Filter Resonance | N | Y | Sound Controller 2 (default: Timbre / Harmonic Quality) |
| 72 | Amp Envelope Release Time | N | Y | Sound Controller 3 (default: Release Time) |
| 73 | Amp Envelope Attack Time | N | Y | Sound Controller 4 (default: Attack Time) |
| 74 | Filter Cutoff | N | Y | Sound Controller 5 (default: Brightness) |
| 75 | Amp Envelope Decay Time | N | Y | Sound Controller 6 (GM2 default: Decay Time) |
| 76 | Sound Controller 7 (GM2 default: Vibrato Rate) | N | N | |
| 77 | Sound Controller 8 (GM2 default: Vibrato Depth) | N | N | |
| 78 | Sound Controller 9 (GM2 default: Vibrato Delay) | N | N | |
| 79 | Amp Envelope Sustain Level | N | Y |  |
| 80 | Amp Envelope Inverted | N | Y |  |
| 81 | General Purpose Controller 6 | N | N | |
| 82 | General Purpose Controller 7 | N | N | |
| 83 | General Purpose Controller 8 | N | N | |
| 84 | Portamento Control | N | N | |
| 85 | Filter Envelope Attack Time | N | Y |  |
| 86 | Filter Envelope Decay Time | N | Y |  |
| 87 | Filter Envelope Sustain Level | N | Y |  |
| 88 | Filter Envelope Release Time | N | Y |  |
| 89 | Filter Envelope Inverted | N | Y |  |
| 90 | Filter Envelope Amount | N | Y |  |
| 91 | Key Tracking Amount | N | Y |  |
| 92 | Effects 2 Depth (default: Tremolo Depth) | N | N | |
| 93 | Effects 3 Depth (default: Chorus Send) | N | N | |
| 94 | Effects 4 Depth (default: Celeste [Detune] Depth) | N | N | |
| 95 | Effects 5 Depth (default: Phaser Depth) | N | N | |
| 96 | Data Increment | N | N | |
| 97 | Data Decrement | N | N | |
| 98 | Non-Registered Parameter Number (LSB) | N | N | |
| 99 | Non-Registered Parameter Number (MSB) | N | N | |
| 100 | Registered Parameter Number (LSB) | N | N | |
| 101 | Registered Parameter Number (MSB) | N | N | |
| 102 | Mod Wheel LFO Frequency | N | Y |  |
| 103 | Mod Wheel LFO Center Value | N | Y |  |
| 104 | Mod Wheel LFO Range | N | Y |  |
| 105 | Mod Wheel LFO Wave Shape | N | Y |  |
| 106 | Mod Wheel LFO Phase | N | Y |  |
| 107 | Mod Wheel LFO Reset | N | Y |  |
| 108 | Filter Mod LFO Frequency | N | Y |  |
| 109 | Filter Mod LFO Amount | N | Y |  |
| 110 | Filter Mod LFO Wave Shape | N | Y |  |
| 111 | Filter Mod LFO Phase | N | Y |  |
| 112 | Filter Mod LFO Reset | N | Y |  |
| 113 | | N | N | |
| 114 | | N | N | |
| 115 | | N | N | |
| 116 | | N | N | |
| 117 | | N | N | |
| 118 | | N | N | |
| 119 | | N | N | |
| 120 | All Sound Off | N | N | |
| 121 | Reset All Controllers | N | N | |
| 122 | Local Control On/Off | N | N | |
| 123 | All Notes Off | N | Y | |
| 124 | Omni Mode Off | N | N | |
| 125 | Omni Mode On | N | N | |
| 126 | Poly Mode Off | N | N | |
| 127 | Poly Mode On | N | N | |

---

## Notes

1. **Device Type**: AccSyn is a standalone hardware/software synthesizer and does not transmit MIDI messages. All "Transmitted" columns are marked "No".

2. **Monophonic Operation**: This is a monophonic (single-voice) synthesizer. When a new Note On is received while a note is playing, the new note takes priority.

3. **Virtual MIDI Port**: AccSyn creates a virtual MIDI input device named "AccSyn MIDI Input" that appears in CoreMIDI on macOS for integration with DAWs and other MIDI software.

4. **Channel Selection**: Defaults to Omni mode (responds to all channels). Users can select a specific channel (1-16) via the settings UI.

5. **Non-Standard CC Usage**: Many CCs are used for synthesizer-specific parameters not covered by the MIDI 1.0 specification. These are marked as "Non-standard" in the Remarks column.

6. **Velocity Sensitivity**: Note-On velocity is recognized and affects amplitude envelope and can be shaped using CC #3 (Velocity Curve).

7. **Pitch Bend**: Pitch bend is recognized with a range configurable via CC #5 (Pitch Bend Range), specified in semitones.

8. **LFO Control**: Two LFOs are available - one controlled by the mod wheel (CC #1) with parameters on CCs 102-107, and one for filter modulation with parameters on CCs 108-112.

9. **Effects**: Built-in effects include wave folder, clipper, bit shifter, and rectifiers. Some effects parameters can be controlled via aftertouch and CCs.

10. **Hot-Swappable Devices**: MIDI input devices can be changed without restarting the application.

---

**For complete parameter details and ranges, see the project documentation at:**
https://gitlab.com/joltedbot-public/accidental-synth

**Project Repository:**
https://gitlab.com/joltedbot-public/accidental-synth

**License:** Apache 2.0
