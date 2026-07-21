# Controls

There are 3 sections to the UI. The Synth and Effects Tabs and the Settings Menu.

The main Synth tab contains all controls for the synth voices, the mixer, and the performance controls like Mod Wheel and Pitch Bend parameters. 


The Effects tab contains all the effects modules and sit in the over all signal chain between everything in the Synth panel, with the exception of 
the output mixer, and the final audio output. The effects are laid out in order of signal flow starting from the top left and proceeding effect to 
effect. The last effect in a row then goes to the first (left most) effect in the next row down.


The Settings Menu has is own page [here](./settings-menu.md)

Most controls should reset to the defaults for the patch you selected, or the init patch if you haven't, when you double click them.

---

## Synth Tab

### Oscillators

AccSyn has four oscillators: **Sub Oscillator**, **Oscillator 1**, **Oscillator 2**, and **Oscillator 3**. They are identical except that the sub 
oscillator is pitched 1 octave below the played note. 

| Control                   | Description                                                                                                      |
|---------------------------|------------------------------------------------------------------------------------------------------------------|
| **Wave Shape**            | Selects the waveform. See the [Waveforms](#waveforms) section below.                                             |
| **Coarse Tune**           | Pitch offset in semitones (−12 to +12).                                                                          |
| **Fine Tune**             | Pitch offset in cents (−63 to +63).                                                                              |
| **Boost**                 | Per Osillator Clipper output boost (0–30db). Adds harmonic content and dirt before the signal reaches the mixer. |
| **Pitch Envelope Amount** | Determines maximum pitch above or below the fundamental that the envelope controls. +/- 1 octave                 |
| **Shape Parameter 1**     | Waveform dependent.See the [Waveforms](#waveforms) section below.                                                |
| **Shape Parameter 2**     | Waveform dependent. See the [Waveforms](#waveforms) section below.                                               |


#### Waveforms

| Index | Name     | Shape Parameter 1 | Shape Parameter 2 | Notes                       |
|-------|----------|-------------------|-------------------|-----------------------------|
| 0     | Sine     | N/A               | N/A               |                             |
| 1     | Triangle | N/A               | N/A               |                             |
| 2     | Square   | N/A               | N/A               |                             |
| 3     | Saw      | N/A               | N/A               | Rising sawtooth wave        |
| 4     | Pulse    | Pulse Width       | N/A               | Width adjustable pulse      |
| 5     | Ramp     | N/A               | N/A               | Falling sawtooth wave       |
| 6     | Supersaw | Detune            | N/A               | 7 detunes Sawtooth waves    |
| 7     | AM/RM    | Modulation Amount | Ring Mod Amount   | Morph between AM & Ring Mod |
| 8     | FM       | Modulation Amount | Ratio             | Bare Bones 2 Op FM          |
| 9     | PM       | Modulation Amount | N/A               | Phase Modulation            |
| 10    | Broken   | How Broken?       | N/A               | Sort of self explanitory    |
| 11    | Noise    | N/A               | N/A               | N/A                         |

---

### Filter

A resonant lowpass filter placed after the oscillator mix.

| Control          | Description                                           |
|------------------|-------------------------------------------------------|
| **Cutoff**       | Filter cutoff frequency in Hz.                        |
| **Resonance**    | Resonance (peak) at the cutoff frequency.             |
| **Poles**        | Filter slope: 1–4 poles (6, 12, 18, or 24 dB/octave). |
| **Key Tracking** | Positive and negative key tracking. See note.         |
| **EG Amount**    | How much the Filter Envelope modulates the cutoff.    |
| **LFO Amount**   | How much the Filter LFO modulates the cutoff.         |

__Filter Key Tracking Note__ - The value is bipolar.  The key tracking centers around E4/MIDI note 64 which will always get the filter value you set with cutoff and then notes above or
below are altered as you change the key tracking. 
In the middle there is no tracking, To the right you get normal key tracking where the 
filter opens up as you play higher notes. To the left you get negative key tracking where the higher notes get darker and the lower notes get 
brighter.   

#### Filter Envelope

An ADSR envelope dedicated to filter cutoff modulation. An ADSR envelope to modulate the cutoff of the filter over time with the played note. The 
inverted mode flips the envelope so that the filter starts at maximum cutoff frequency (full brightness) and then the envelope lowers the cutoff and 
brings it back to full based on how you set it.

Due to the interaction of the filter envelope sustain and the need for there to be somewhere for the cutoff to go I suggest starting with the 
cutoff either all the way off, or maybe in the center and tune by ear from there. 

| Control      | Description                   |
|--------------|-------------------------------|
| **Atk**      | Attack time in milliseconds.  |
| **Dec**      | Decay time in milliseconds.   |
| **Sus**      | Sustain level (0.0–1.0).      |
| **Rel**      | Release time in milliseconds. |
| **Inverted** | Inverts the envelope shape    |

#### Filter LFO

A dedicated LFO for filter cutoff modulation

The LFO modulates the cutoff around (above and below) the current frequency set by cutoff slider. It also follows the cut off value produced by the 
Filter Envelope value so using both can produce some interesting effects. 

| Control        | Description                                                                    |
|----------------|--------------------------------------------------------------------------------|
| **Frequency**  | LFO rate in Hz (or in beat intervals when Clock Sync is enabled.               |
| **Wave Shape** | LFO waveform (same options as oscillators).                                    |
| **Phase**      | Adjust the phase of the LFO in real time (0 to 360 degrees)                    |
| **Clock Sync** | Syncs the LFO rate to incoming MIDI clock pulses                               |
| **Key Sync**   | Syncs the LFO rate to each midi note on event. Can be combined with Clock Sync |

---

### Amp Envelope

An ADSR envelope to modulate the amplitude of the played notes over time. The inverted mode flips the envelope so that the 
note starts at maximum amplitude and then the envelope lowers the amplitude and brings it back to full based on how you set it.

| Control      | Description                                                                                                                |
|--------------|----------------------------------------------------------------------------------------------------------------------------|
| **Atk**      | Attack time in milliseconds.                                                                                               |
| **Dec**      | Decay time in milliseconds.                                                                                                |
| **Sus**      | Sustain level (0.0–1.0).                                                                                                   |
| **Rel**      | Release time in milliseconds.                                                                                              |
| **Inverted** | Inverts the envelope — amplitude starts at full and decreases on attack.                                                   |

---

### Pitch Envelope

An ADSR envelope that modulates oscillator pitch. Each oscillator has a **Pitch Envelope Amount** control that determines how much it is affected.

| Control | Description                   |
|---------|-------------------------------|
| **Atk** | Attack time in milliseconds.  |
| **Dec** | Decay time in milliseconds.   |
| **Sus** | Sustain level (0.0–1.0).      |
| **Rel** | Release time in milliseconds. |

---

### Mixer

Controls the level, stereo position, and mute state for each oscillator and for the master output.

#### Per-Oscillator (Sub, Osc 1, Osc 2, Osc 3)

| Control     | Description                                                  |
|-------------|--------------------------------------------------------------|
| **Level**   | Oscillator output level (0.0–1.0).                           |
| **Balance** | Stereo pan (left = −1.0, center = 0, right = 1.0).           |
| **Mute**    | Silences this oscillator without changing its level setting. |

#### Output

| Control     | Description                    |
|-------------|--------------------------------|
| **Level**   | Master output level (0.0–1.0). |
| **Balance** | Master stereo pan.             |
| **Mute**    | Silences all output.           |

---

### Oscillator Options

Global and per-oscillator performance settings.

| Control              | Description                                                                                                                                                         |
|----------------------|---------------------------------------------------------------------------------------------------------------------------------------------------------------------|
| **Portamento**       | Enables pitch glide between notes. The UI controls all oscillators together; per-oscillator values can be set manually in patch files.                              |
| **Time**             | Portamento glide duration.                                                                                                                                          |
| **Hard Sync**        | Syncs oscillator 2's phase to oscillator 1, producing hard sync effect.                                                                                             |
| **Key Sync**         | Resets the oscillator phase on each note-on. Useful for consistent attack transients.                                                                               |
| **Pitch Bend Range** | Maximum pitch bend in semitones (2–12).                                                                                                                             |
| **Velocity Curve**   | Shapes how MIDI velocity maps to amplitude. 0.5 is linear; lower values compress dynamics; higher values expand them.                                               |
| **Polarity**         | Inverts the polarity of the output signal. Usually not audible on its own; useful when AccSyn is combined with another signal source and phase cancellation occurs. |
| **Sustain**          | Sustain pedal state (mirrors MIDI CC 64).                                                                                                                           |

---

### Modulation (Mod) Wheel

The midi mod wheel parameter `CC#1` controls the range parameter of a dedicated LFO which controls the depth/amount of the effect on the oscillators.

With some exceptions noted in this table that LFO is applied to the pitch of the oscillator producing a Vibrator effect. Here is the effect it has 
on each oscillator.

| Waveshape | Target              | Effect Produced                                      | 
|-----------|---------------------|------------------------------------------------------|
| Sine      | Frequency           | Vibrato                                              |
| Triangle  | Frequency           | Vibrato                                              |
| Square    | Frequency           | Vibrato                                              |
| Ramp      | Frequency           | Vibrato                                              |
| Supersaw  | Frequency           | Vibrato                                              |
| AM/RM     | Modulator Frequency | Ghostly Overtone Vibrato                             |
| FM        | Modulator Frequency | Basic 3rd Operator. Modwheel -> Modulator -> Carrier |
| PM        | Frequency           | Pulsing effect that changes speed in time to the LFO |
| Broken    | Frequency           | Vibrato                                              |
| Noise     | N/A                 | N/A                                                  |


The Mod Wheel control section has the following controls for the underlying Mod Wheel LFO

| Control        | Description                                                       |
|----------------|-------------------------------------------------------------------|
| **Frequency**  | The LFO's rate in Hz or clock divisions if clock synced           |
| **Wave Shape** | Wave shape for the LFO                                            |
| **Phase**      | Set the current phase of the LFO. Manually sync to other sounds   |
| **Clock Sync** | Syncs LFO rate to MIDI clock                                      |
| **Key Sync**   | Reset the LFO on each key press. Works with or without Clock Sync |


---

## Effects Tab

All effects are applied in series after the synthesizer signal chain. Each has an **Enable** toggle. Disabled effects pass audio through unchanged.

### 1 — Saturation

Adds harmonic distortion.

| Control                      | Description                                                    |
|------------------------------|----------------------------------------------------------------|
| **Type**                     | Saturation algorithm.                                          |
| **Amount**                   | Drive amount.                                                  |
| **Post Saturation Gain Cut** | Level trim after saturation to compensate for volume increase. |

### 2 — Colour Compressor

Dynamics compression with a colored character.

| Control         | Description                                          |
|-----------------|------------------------------------------------------|
| **Threshold**   | Level above which compression is applied.            |
| **Ratio**       | Compression ratio.                                   |
| **Makeup Gain** | Output gain to compensate for gain reduction.        |
| **Blend**       | Wet/Dry signal blend (Left: 0% Wet, Right: 100% Wet) |

### 3 — Wave Folder

Folds the waveform back on itself, adding upper harmonics.

| Control                  | Description                                                           |
|--------------------------|-----------------------------------------------------------------------|
| **Fold Amount**          | Folding depth applied to positive samples.                            |
| **Asymmetrical**         | When enabled, positive and negative samples are folded independently. |
| **Negative Fold Amount** | Folding depth for negative samples (active in Asymmetrical mode).     |

### 4 — Bit Crusher

Reduces bit depth for lo-fi aliasing and quantization noise.

| Control           | Description                                          |
|-------------------|------------------------------------------------------|
| **Bit Reduction** | Amount of bit depth reduction. Higher = more lo-fi.  |
| **Blend**         | Wet/Dry signal blend (Left: 0% Wet, Right: 100% Wet) | 

### 5 — Clipper

Hard clips the signal, adding aggressive saturation at high levels.

| Control                   | Description                                                                    |
|---------------------------|--------------------------------------------------------------------------------|
| **Threshold**             | Clip threshold.                                                                |
| **Pre-Clip Boost**        | Gain added before clipping to push more of the signal into saturation.         |
| **Post-Clip Makeup Gain** | Output level trim after clipping.                                              |
| **Notch**                 | Clips the value to 0 rather than to the threshold for more extreme distortion. |

### 6 — Gate Clipping



| Control                   | Description                                                                    |
|---------------------------|--------------------------------------------------------------------------------|
| **Threshold**             | Level below which the gate closes.                                             |
| **Pre-Gate Cut**          | Attenuates signal before the gate.                                             |
| **Post-Gate Makeup Gain** | Amplifies signal after the gate opens.                                         |
| **Notch**                 | Clips the value to 0 rather than to the threshold for more extreme distortion. |

### 7 — Wave Rectifier

Flips or removes negative samples, changing the waveform symmetry and adding harmonics.

| Control                   | Description                                                           |
|---------------------------|-----------------------------------------------------------------------|
| **Half Wave / Full Wave** | Half Wave removes negative samples; Full Wave flips them to positive. |
| **Blend**                 | Wet/Dry signal blend (Left: 0% Wet, Right: 100% Wet)                  |

### 8 — Chorus

2 voice chorus effect

| Control                   | Description                                                           |
|---------------------------|-----------------------------------------------------------------------|
| **Depth** | The depth of the chorusing effect  |
| **Rate** | Speed of the modulation |
| **Feedback** | How much of the wet signal is fed back into the delay buffer |
| **Blend** | Wet/Dry signal blend (Left: 0% Wet, Right: 100% Wet) |

### 9 — Flanger

Flanger effect

| Control                   | Description                                                           |
|---------------------------|-----------------------------------------------------------------------|
| **Depth** | The depth of the flanger effect  |
| **Rate** | Speed of the modulation |
| **Feedback** | How much of the wet signal is fed back into the delay buffer |
| **Blend** | Wet/Dry signal blend (Left: 0% Wet, Right: 100% Wet) |

### 10 — Auto Pan

Automatically pans the signal between left and right.

| Control        | Description                             |
|----------------|-----------------------------------------|
| **Rate**       | Panning speed.                          |
| **Width**      | Maximum pan amount.                     |
| **Wave Shape** | LFO shape driving the panning movement. |

### 11 — Tremolo

Modulates the output amplitude.

| Control        | Description                                 |
|----------------|---------------------------------------------|
| **Rate**       | Tremolo speed.                              |
| **Depth**      | Tremolo intensity.                          |
| **Wave Shape** | LFO shape driving the amplitude modulation. |

### 12 — Delay

A simple stereo delay.

| Control      | Description                                     |
|--------------|-------------------------------------------------|
| **Amount**   | Wet delay level.                                |
| **Time**     | Delay time.                                     |
| **Feedback** | Amount of delay output fed back into the input. |
