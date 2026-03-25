use strum_macros::{EnumCount, EnumIter, FromRepr};

/// Parameter change events sent to the synthesizer from the UI and MIDI subsystems.
pub enum SynthesizerUpdateEvents {
    /// Change oscillator waveform shape (oscillator index, shape index).
    WaveShapeIndex(i32, i32),
    /// Change oscillator coarse tune (oscillator index, normalized value).
    CourseTune(i32, f32),
    /// Change oscillator fine tune (oscillator index, normalized value).
    FineTune(i32, f32),
    /// Change oscillator clipper boost (oscillator index, normalized value).
    ClipperBoost(i32, f32),
    /// Change oscillator waveform parameter 1 (oscillator index, normalized value).
    Parameter1(i32, f32),
    /// Change oscillator waveform parameter 2 (oscillator index, normalized value).
    Parameter2(i32, f32),
    /// Change filter cutoff frequency (normalized value).
    FilterCutoffFrequency(f32),
    /// Change filter resonance (normalized value).
    FilterResonance(f32),
    /// Change filter pole count (normalized value).
    FilterPoleCount(f32),
    /// Change filter key tracking amount (normalized value).
    FilterKeyTrackingAmount(f32),
    /// Change filter envelope modulation amount (normalized value).
    FilterEnvelopeAmount(f32),
    /// Change filter LFO modulation amount (normalized value).
    FilterLfoAmount(f32),
    /// Change filter envelope attack time (envelope index, normalized value).
    FilterEnvelopeAttack(i32, f32),
    /// Change filter envelope decay time (envelope index, normalized value).
    FilterEnvelopeDecay(i32, f32),
    /// Change filter envelope sustain level (envelope index, normalized value).
    FilterEnvelopeSustain(i32, f32),
    /// Change filter envelope release time (envelope index, normalized value).
    FilterEnvelopeRelease(i32, f32),
    /// Toggle filter envelope inversion (envelope index, inverted).
    FilterEnvelopeInvert(i32, bool),
    /// Change LFO frequency (LFO index, normalized value).
    LfoFrequency(i32, f32),
    /// Change LFO waveform shape (LFO index, shape index).
    LfoShapeIndex(i32, i32),
    /// Change LFO phase offset (LFO index, normalized value).
    LfoPhase(i32, f32),
    /// Reset LFO phase to zero (LFO index).
    LfoPhaseReset(i32),
    /// Toggle portamento on/off.
    PortamentoEnabled(bool),
    /// Change portamento glide time (normalized value).
    PortamentoTime(f32),
    /// Change pitch bend range in semitones (normalized value).
    PitchBendRange(f32),
    /// Change velocity sensitivity curve (normalized value).
    VelocityCurve(f32),
    /// Toggle oscillator hard sync on/off.
    HardSyncEnabled(bool),
    /// Toggle oscillator key sync on/off.
    KeySyncEnabled(bool),
    /// Toggle polarity flip on/off.
    PolarityFlipped(bool),
    /// Change output mixer stereo balance (normalized value).
    OutputBalance(f32),
    /// Change output mixer level (normalized value).
    OutputLevel(f32),
    /// Toggle output mixer mute on/off.
    OutputMute(bool),
    /// Change per-oscillator mixer balance (oscillator index, normalized value).
    OscillatorMixerBalance(i32, f32),
    /// Change per-oscillator mixer level (oscillator index, normalized value).
    OscillatorMixerLevel(i32, f32),
    /// Toggle per-oscillator mixer mute (oscillator index, muted).
    OscillatorMixerMute(i32, bool),
    /// Toggle an audio effect on/off (effect index, enabled).
    EffectEnabled(i32, bool),
    /// Change an effect parameter value (effect index, parameter index, value).
    EffectParameterValues(i32, i32, f32),
    /// Change to a new preset (preset index).
    PresetChanged(i32),
    /// Save the current module parameters to a patch file
    PatchSaved(String),
}

/// Index identifying each oscillator in the synthesizer.
#[derive(Debug, Clone, Copy, EnumCount, EnumIter, FromRepr)]
#[repr(i32)]
pub enum OscillatorIndex {
    /// Sub-oscillator (index 0).
    Sub = 0,
    /// Oscillator 1 (index 1).
    One = 1,
    /// Oscillator 2 (index 2).
    Two = 2,
    /// Oscillator 3 (index 3).
    Three = 3,
}

impl OscillatorIndex {
    /// Converts an i32 index to the corresponding oscillator variant.
    pub fn from_i32(index: i32) -> Option<Self> {
        Self::from_repr(index)
    }
}

/// Index identifying each LFO in the synthesizer.
#[derive(Debug, Clone, Copy, EnumCount, EnumIter, FromRepr)]
#[repr(i32)]
pub enum LFOIndex {
    /// Mod wheel LFO (index 0).
    ModWheel = 0,
    /// Filter modulation LFO (index 1).
    Filter = 1,
}

impl LFOIndex {
    /// Converts an i32 index to the corresponding LFO variant.
    pub fn from_i32(index: i32) -> Option<Self> {
        Self::from_repr(index)
    }
}

/// Index identifying each envelope generator in the synthesizer.
#[derive(Debug, Clone, Copy, EnumCount, EnumIter, FromRepr)]
#[repr(i32)]
pub enum EnvelopeIndex {
    /// Amplitude envelope (index 0).
    Amp = 0,
    /// Filter modulation envelope (index 1).
    Filter = 1,
}

impl EnvelopeIndex {
    /// Converts an i32 index to the corresponding envelope variant.
    pub fn from_i32(index: i32) -> Option<Self> {
        Self::from_repr(index)
    }
}
