# Patch File Format

This document describes the JSON patch file format used by Accidental Synthesizer for system presets and user created patches.

## Overview

Patch files are JSON documents that capture the complete state of a synthesizer patch, including oscillator settings, filter parameters, envelope configurations, LFO modulation, mixer levels, keyboard response, and effects processing. They allow users to save, share, and recall synthesizer configurations.

System presets are located in `crates/accsyn-engine/src/synthesizer/patches/` and use the `.json` file extension. `init.json` is loade by default 
at system startup.

## Root Structure

A patch file has the following top-level keys:

```json
{
  "effects": [...],
  "envelopes": [...],
  "filter": {...},
  "keyboard": {...},
  "lfos": [...],
  "mixer": {...},
  "oscillators": [...]
}
```

All keys are required. The following sections define the structure and valid ranges for each.

## Oscillators

Array of 4 oscillator objects, in order: Sub, Osc 1, Osc 2, Osc 3.

| Field | Type | Range | Description |
|-------|------|-------|-------------|
| `wave_shape_index` | integer | 0-9 | Waveform shape (see Waveforms table) |
| `course_tune` | integer | -12 to +12 | Coarse pitch in semitones |
| `fine_tune` | integer | -63 to +63 | Fine pitch in cents |
| `clipper_boost` | integer | 0-30 | Clipper output boost in dB |
| `shape_parameter1` | number | 0.0-1.0 | Waveform-dependent (e.g., pulse width, AM/FM depth) |
| `shape_parameter2` | number | 0.0-1.0 | Waveform-dependent |
| `pitch_bend` | integer | see below | Pitch bend offset in cents (see [Pitch Bend](#pitch-bend)) |
| `portamento_enabled` | boolean | | Enable pitch glide between notes |
| `portamento_time` | integer | 0-65535 | Glide duration in audio buffers |
| `hard_sync_enabled` | boolean | | Enable hard sync to master oscillator |
| `key_sync_enabled` | boolean | | Reset phase on each note |
| `gate_flag` | boolean | | [Performance state](#performance-state-fields) — leave as false |

### Example Oscillator

```json
{
  "wave_shape_index": 3,
  "course_tune": 0,
  "fine_tune": 0,
  "clipper_boost": 0,
  "shape_parameter1": 0,
  "shape_parameter2": 0,
  "pitch_bend": 0,
  "portamento_enabled": false,
  "portamento_time": 7,
  "hard_sync_enabled": false,
  "key_sync_enabled": false,
  "gate_flag": false
}
```

## Filter

Single filter object controlling the resonant lowpass filter.

| Field | Type | Range | Description                                               |
|-------|------|-------|-----------------------------------------------------------|
| `cutoff_frequency` | number | 0.0-20000.0 | Filter cutoff in Hz (clamped to 35% of Nyquist at runtime) |
| `filter_poles` | integer | 1-4 | Number of filter poles (1 = 6 dB/oct, 2 = 12, 3 = 18, 4 = 24) |
| `resonance` | number | 0.0-0.90 | Filter resonance (peak at cutoff)                         |
| `key_tracking_amount` | number | 0.0-1.0 | Bipolar key tracking (see [Key Tracking](#key-tracking))   |
| `current_note_number` | integer | | [Performance state](#performance-state-fields) — leave at 0 |

### Example Filter

```json
{
  "cutoff_frequency": 16800,
  "filter_poles": 4,
  "resonance": 0,
  "key_tracking_amount": 0.5,
  "current_note_number": 0
}
```

## Envelopes

Array of 2 envelope objects: [Amplitude Envelope, Filter Envelope].

Each envelope is an ADSR (Attack, Decay, Sustain, Release) generator.

| Field | Type | Range | Description |
|-------|------|-------|-------------|
| `amount` | number | 0.0-1.0 | Modulation depth (envelope intensity) |
| `attack_ms` | integer | 1-5000 | Attack time in milliseconds |
| `decay_ms` | integer | 10-5000 | Decay time in milliseconds |
| `sustain_level` | number | 0.0-1.0 | Sustain level after decay |
| `release_ms` | integer | 10-10000 | Release time in milliseconds |
| `is_inverted` | boolean | | Invert the envelope output |
| `sustain_pedal` | boolean | | [Performance state](#performance-state-fields) — leave as false |
| `gate_flag` | integer | | [Performance state](#performance-state-fields) — leave at 0 |

### Example Envelope

```json
{
  "amount": 1,
  "attack_ms": 200,
  "decay_ms": 200,
  "sustain_level": 0.8,
  "release_ms": 200,
  "is_inverted": false,
  "sustain_pedal": false,
  "gate_flag": 0
}
```

## LFOs

Array of 2 LFO objects: [LFO 1, LFO 2].

Low-frequency oscillators provide modulation sources for other parameters.

| Field | Type | Range | Description |
|-------|------|-------|-------------|
| `wave_shape` | integer | 0-9 | Waveform shape (see Waveforms table) |
| `frequency` | number | 0.01-20000.0 | Oscillation frequency in Hz |
| `center_value` | number | -1.0 to 1.0 | Center point of modulation range |
| `range` | number | 0.0-2.0 | Modulation depth (0 disables modulation) |
| `phase` | number | 0.0-1.0 | Starting phase (0.0 to 1.0 wraps one cycle) |
| `reset` | boolean | | Reset phase to 0 on note-on |

### Example LFO

```json
{
  "wave_shape": 0,
  "frequency": 10,
  "center_value": 1,
  "range": 0,
  "phase": 0,
  "reset": false
}
```

## Mixer

Single mixer object controlling oscillator levels and master output.

| Field | Type | Range | Description |
|-------|------|-------|-------------|
| `level` | number | 0.0-1.0 | Master output level |
| `balance` | number | -1.0 to 1.0 | Master stereo pan (0 = center) |
| `is_muted` | boolean | | Mute master output |
| `quad_mixer_inputs` | array | | 4 objects (one per oscillator) |

Each `quad_mixer_inputs` object has:

| Field | Type | Range | Description |
|-------|------|-------|-------------|
| `level` | number | 0.0-1.0 | Oscillator output level |
| `balance` | number | -1.0 to 1.0 | Oscillator stereo pan (0 = center) |
| `mute` | boolean | | Mute this oscillator |

### Example Mixer

```json
{
  "level": 0.8,
  "balance": 0,
  "is_muted": false,
  "quad_mixer_inputs": [
    { "level": 0, "balance": 0, "mute": false },
    { "level": 1, "balance": 0, "mute": false },
    { "level": 1, "balance": 0, "mute": false },
    { "level": 1, "balance": 0, "mute": false }
  ]
}
```

## Keyboard

Single keyboard object controlling MIDI response and velocity sensitivity.

| Field | Type | Range | Description |
|-------|------|-------|-------------|
| `pitch_bend_range` | integer | 2-12 | Maximum pitch bend in semitones |
| `velocity_curve` | number | 0.0-1.0 | Velocity response curve (see [Velocity Curve](#velocity-curve)) |
| `aftertouch_amount` | number | 0.0-1.0 | Aftertouch modulation depth |
| `mod_wheel_amount` | number | 0.0-1.0 | Modulation wheel depth |
| `polarity_flipped` | boolean | | Invert polarity of all pitch-related inputs |

### Example Keyboard

```json
{
  "pitch_bend_range": 12,
  "velocity_curve": 0.5,
  "aftertouch_amount": 0,
  "mod_wheel_amount": 0,
  "polarity_flipped": false
}
```

## Effects

Array of 10 effect objects in fixed order. Each effect has:

| Field | Type | Range | Description |
|-------|------|-------|-------------|
| `is_enabled` | boolean | | Enable/disable the effect |
| `parameters` | array of 4 | 0.0-1.0 | Effect-specific parameters |

Effects are processed in this order:

| Index | Name | Param 0                                             | Param 1               | Param 2                                    | Param 3      |
|-------|------|-----------------------------------------------------|-----------------------|--------------------------------------------|--------------|
| 0     | Saturation | type                                                | drive amount          | post drive level cut                       | unused       |
| 1     | Compressor | threshold                                           | ratio                 | makeup gain                                | unused       |
| 2     | WaveFolder | amount (in asymetric mode - positive sample amount) | asymetric mode toggle | in asymetric mode - negative sample amount | unused       |
| 3     | Clipper | threshold                                           | pre-clip boost        | post-clip boost                            | notch toggle |
| 4     | Gate | thereshold                                          | pre-gate level cut    | post-gate makeup gain                      | unused       |
| 5     | Rectifier | half wave mode toggle                               | unused                | unused                                     | unused       |
| 6     | BitShifter | bits (1-16 normalized)                              | unused                | unused                                     | unused       |
| 7     | Delay | delay level                                         | time                  | feedback                                   | unused       |
| 8     | AutoPan | rate                                                | width                 | wave shape                                 | unused       |
| 9     | Tremolo | rate                                                | depth                 | wave shape                                 | unused       |

All parameters use the range 0.0 to 1.0, including "unused" fields which should be set to 0.
Toggle parameters are 0.0 for Off and > 0.0 values for on but 1.0 is the preffered On value.

### Example Effect

```json
{
  "is_enabled": false,
  "parameters": [0.5, 0.5, 0.5, 0]
}
```

## Waveforms

Both oscillators and LFOs use the same waveform index table:

| Index | Name | Description |
|-------|------|-------------|
| 0 | Sine | Pure sinusoidal |
| 1 | Triangle | Triangular wave |
| 2 | Square | 50% duty cycle square |
| 3 | Saw | Rising sawtooth |
| 4 | Pulse | Pulse with duty cycle control (use shape_parameter1/wave_shape) |
| 5 | Ramp | Falling sawtooth ramp |
| 6 | Supersaw | Detuned sawtooth stack |
| 7 | AM | Amplitude modulation (shape_parameter1 controls depth) |
| 8 | FM | Frequency modulation (shape_parameter1/2 control depth) |
| 9 | Noise | White noise |

## Notes for Manual Patch Editing

### Performance State Fields

Some fields reflect live performance state — they are updated in real time by MIDI input or the synth engine during playback. While you can set these in a patch file, they will be overwritten as soon as the corresponding MIDI event occurs. In most cases they should be left at their default values.

- `oscillators[*].pitch_bend` — Leave at 0 (overwritten by MIDI pitch bend wheel)
- `oscillators[*].gate_flag` — Leave as false (overwritten by note on/off)
- `filter.current_note_number` — Leave at 0 (overwritten by note on)
- `envelopes[*].sustain_pedal` — Leave as false (overwritten by MIDI sustain pedal)
- `envelopes[*].gate_flag` — Leave at 0 (overwritten by note on/off)

### Key Tracking

`key_tracking_amount` is stored as a normalized 0.0–1.0 value but represents a bipolar range internally:

| Value | Effective tracking | Meaning |
|-------|-------------------|---------|
| 0.0 | -1.0 (full negative) | Higher notes lower the cutoff |
| 0.5 | 0.0 (no tracking) | Cutoff is independent of pitch — **this is the default** |
| 1.0 | +1.0 (full positive) | Higher notes raise the cutoff |

The conversion is: `bipolar = (key_tracking_amount - 0.5) * 2.0`. The reference note is MIDI note 64 (E4) — notes above shift the cutoff up or down relative to that center depending on the tracking direction.

### Pitch Bend

`pitch_bend` is stored in **cents** (100 cents = 1 semitone), not raw MIDI values. Its effective range is determined by the `keyboard.pitch_bend_range` setting:

- Maximum positive bend = `pitch_bend_range × 100` cents
- Maximum negative bend = `-(pitch_bend_range × 100)` cents

For example, with `pitch_bend_range: 12` (12 semitones = 1 octave), the range is -1200 to +1200 cents. With `pitch_bend_range: 2`, the range is -200 to +200 cents.

A value of 0 means no pitch bend (center position). This is a [performance state field](#performance-state-fields) — it will be overwritten by MIDI pitch bend input during playback.

### Velocity Curve

`velocity_curve` controls how MIDI velocity maps to amplitude. The value 0.5 is the midpoint:

| Value | Response | Behavior |
|-------|----------|----------|
| 0.0 | Fixed | All velocities produce maximum volume |
| 0.0–0.5 | Compressed | Softer touch still produces relatively loud output |
| 0.5 | Linear | Velocity maps 1:1 to amplitude |
| 0.5–1.0 | Expanded | Requires harder touch for loud output |
| 1.0 | Maximum expansion | Very sensitive to velocity differences |

### Parameter Validation

JSON files are validated at runtime. Invalid values will be clamped to their ranges. However, it is best practice to respect the documented ranges when editing patches manually.

### File Format

- Use standard JSON formatting (valid JSON is required)
- All numeric values must be valid JSON numbers (not strings)
- Boolean values use JSON `true` and `false` (not strings)
- Trailing commas are not valid JSON

### Creating New Patches

Start from `init.json` as a template and modify parameters as desired. The initial patch is a good reference for structure and default values.

## Example: Creating a Saw Lead

Here is a simplified example of a sawtooth lead patch focusing on the key changes:

```json
{
  "oscillators": [
    {
      "wave_shape_index": 3,
      "course_tune": -12,
      "fine_tune": 0,
      "clipper_boost": 0,
      "shape_parameter1": 0,
      "shape_parameter2": 0,
      "pitch_bend": 0,
      "portamento_enabled": true,
      "portamento_time": 150,
      "hard_sync_enabled": false,
      "key_sync_enabled": false,
      "gate_flag": false
    },
    {
      "wave_shape_index": 3,
      "course_tune": 0,
      "fine_tune": 0,
      "clipper_boost": 0,
      "shape_parameter1": 0,
      "shape_parameter2": 0,
      "pitch_bend": 0,
      "portamento_enabled": true,
      "portamento_time": 150,
      "hard_sync_enabled": false,
      "key_sync_enabled": false,
      "gate_flag": false
    },
    {
      "wave_shape_index": 3,
      "course_tune": 12,
      "fine_tune": 0,
      "clipper_boost": 0,
      "shape_parameter1": 0,
      "shape_parameter2": 0,
      "pitch_bend": 0,
      "portamento_enabled": true,
      "portamento_time": 150,
      "hard_sync_enabled": false,
      "key_sync_enabled": false,
      "gate_flag": false
    },
    {
      "wave_shape_index": 0,
      "course_tune": 0,
      "fine_tune": 0,
      "clipper_boost": 0,
      "shape_parameter1": 0,
      "shape_parameter2": 0,
      "pitch_bend": 0,
      "portamento_enabled": false,
      "portamento_time": 7,
      "hard_sync_enabled": false,
      "key_sync_enabled": false,
      "gate_flag": false
    }
  ],
  "filter": {
    "cutoff_frequency": 8000,
    "filter_poles": 4,
    "resonance": 0.5,
    "key_tracking_amount": 0.8,
    "current_note_number": 0
  },
  "envelopes": [
    {
      "amount": 1,
      "attack_ms": 50,
      "decay_ms": 300,
      "sustain_level": 0.7,
      "release_ms": 500,
      "is_inverted": false,
      "sustain_pedal": false,
      "gate_flag": 0
    },
    {
      "amount": 1,
      "attack_ms": 10,
      "decay_ms": 200,
      "sustain_level": 0.5,
      "release_ms": 300,
      "is_inverted": false,
      "sustain_pedal": false,
      "gate_flag": 0
    }
  ],
  "lfos": [
    {
      "wave_shape": 0,
      "frequency": 5,
      "center_value": 1,
      "range": 0,
      "phase": 0,
      "reset": false
    },
    {
      "wave_shape": 0,
      "frequency": 0.1,
      "center_value": 0,
      "range": 0,
      "phase": 0,
      "reset": false
    }
  ],
  "mixer": {
    "level": 0.8,
    "balance": 0,
    "is_muted": false,
    "quad_mixer_inputs": [
      { "level": 0.3, "balance": -0.3, "mute": false },
      { "level": 1, "balance": 0, "mute": false },
      { "level": 1, "balance": 0.3, "mute": false },
      { "level": 0, "balance": 0, "mute": true }
    ]
  },
  "keyboard": {
    "pitch_bend_range": 12,
    "velocity_curve": 0.5,
    "aftertouch_amount": 0,
    "mod_wheel_amount": 0,
    "polarity_flipped": false
  },
  "effects": [
    { "is_enabled": false, "parameters": [0, 0, 0, 0] },
    { "is_enabled": false, "parameters": [0, 0, 0, 0] },
    { "is_enabled": false, "parameters": [0, 0, 0, 0] },
    { "is_enabled": false, "parameters": [0, 0, 0, 0] },
    { "is_enabled": false, "parameters": [0, 0, 0, 0] },
    { "is_enabled": false, "parameters": [0, 0, 0, 0] },
    { "is_enabled": false, "parameters": [0, 0, 0, 0] },
    { "is_enabled": false, "parameters": [0, 0, 0, 0] },
    { "is_enabled": false, "parameters": [0, 0, 0, 0] },
    { "is_enabled": false, "parameters": [0, 0, 0, 0] }
  ]
}
```
