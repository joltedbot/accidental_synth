/// UI update events sent from the engine, audio, and MIDI subsystems to the UI.
#[derive(Debug, Clone, PartialEq)]
pub enum UIUpdates {
    /// Display a MIDI status message on screen.
    MidiScreen(String),
    /// Updated list of available MIDI input port names.
    MidiPortList(Vec<String>),
    /// Currently selected MIDI port index.
    MidiPortIndex(i32),
    /// Currently selected MIDI channel filter index.
    MidiChannelIndex(i32),
    /// Updated list of available audio output device names.
    AudioDeviceList(Vec<String>),
    /// Currently selected audio output device index.
    AudioDeviceIndex(i32),
    /// Number of output channels on the current audio device.
    AudioDeviceChannelCount(u16),
    /// Currently assigned left and right output channel indexes.
    AudioDeviceChannelIndexes {
        /// Left output channel index.
        left: i32,
        /// Right output channel index.
        right: i32,
    },
    /// Index into the supported sample rates list for the current device.
    AudioDeviceSampleRateIndex(i32),
    /// Index into the supported buffer sizes list for the current device.
    AudioDeviceBufferSizeIndex(i32),
    /// Oscillator waveform shape changed (oscillator index, shape index).
    OscillatorWaveShape(i32, i32),
    /// Oscillator fine tune changed (oscillator index, normalized value, cents display).
    OscillatorFineTune(i32, f32, i32),
    /// Oscillator fine tune cents display value (oscillator index, cents).
    OscillatorFineTuneCents(i32, i32),
    /// Oscillator coarse tune changed (oscillator index, semitones).
    OscillatorCourseTune(i32, i32),
    /// Oscillator clipper boost changed (oscillator index, normalized value).
    OscillatorClipperBoost(i32, f32),
    /// Oscillator waveform parameter 1 changed (oscillator index, normalized value).
    OscillatorParameter1(i32, f32),
    /// Oscillator waveform parameter 2 changed (oscillator index, normalized value).
    OscillatorParameter2(i32, f32),
    /// LFO frequency changed (LFO index, normalized value).
    LFOFrequency(i32, f32),
    /// LFO frequency display value in Hz (LFO index, Hz).
    LFOFrequencyDisplay(i32, f32),
    /// LFO phase changed (LFO index, normalized value).
    LFOPhase(i32, f32),
    /// LFO waveform shape changed (LFO index, normalized value).
    LFOWaveShape(i32, f32),
    /// Envelope attack time changed (envelope index, normalized value).
    EnvelopeAttackTime(i32, f32),
    /// Envelope decay time changed (envelope index, normalized value).
    EnvelopeDecayTime(i32, f32),
    /// Envelope sustain level changed (envelope index, normalized value).
    EnvelopeSustainLevel(i32, f32),
    /// Envelope release time changed (envelope index, normalized value).
    EnvelopeReleaseTime(i32, f32),
    /// Envelope inversion toggled (envelope index, normalized value).
    EnvelopeInverted(i32, f32),
    /// Filter cutoff frequency changed (normalized value).
    FilterCutoff(f32),
    /// Filter resonance changed (normalized value).
    FilterResonance(f32),
    /// Filter pole count changed (normalized value).
    FilterPoles(f32),
    /// Filter key tracking amount changed (normalized value).
    FilterKeyTracking(f32),
    /// Filter envelope modulation amount changed (normalized value).
    FilterEnvelopeAmount(f32),
    /// Filter LFO modulation amount changed (normalized value).
    FilterLFOAmount(f32),
    /// Output mixer stereo balance changed (normalized value).
    OutputMixerBalance(f32),
    /// Output mixer level changed (normalized value).
    OutputMixerLevel(f32),
    /// Output mixer mute state changed (0.0 or 1.0).
    OutputMixerIsMuted(f32),
    /// Per-oscillator mixer balance changed (oscillator index, normalized value).
    OscillatorMixerBalance(i32, f32),
    /// Per-oscillator mixer level changed (oscillator index, normalized value).
    OscillatorMixerLevel(i32, f32),
    /// Per-oscillator mixer mute state changed (oscillator index, 0.0 or 1.0).
    OscillatorMixerIsMuted(i32, f32),
    /// Portamento time changed (normalized value).
    PortamentoTime(f32),
    /// Portamento enabled state changed (0.0 or 1.0).
    PortamentoEnabled(f32),
    /// Pitch bend range changed (normalized value).
    PitchBendRange(f32),
    /// Velocity curve changed (normalized value).
    VelocityCurve(f32),
    /// Hard sync enabled state changed (0.0 or 1.0).
    HardSync(f32),
    /// Key sync enabled state changed (0.0 or 1.0).
    KeySync(f32),
    /// Effect parameters changed (effect index, enabled, param1, param2, param3, param4).
    Effect(i32, bool, f32, f32, f32, f32),
    /// Patch changed — UI should reload all parameter values from the patch at this index.
    Patches(i32),
}

/// ADSR envelope stage identifier.
#[derive(Debug, Clone, Copy)]
pub enum EnvelopeStage {
    /// Attack phase — level rising from zero to peak.
    Attack,
    /// Decay phase — level falling from peak to sustain.
    Decay,
    /// Sustain phase — level held while key is pressed.
    Sustain,
    /// Release phase — level falling from sustain to zero.
    Release,
}
