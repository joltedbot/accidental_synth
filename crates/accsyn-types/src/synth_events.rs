use strum_macros::{EnumCount, EnumIter, FromRepr};

pub enum SynthesizerUpdateEvents {
    WaveShapeIndex(i32, i32),
    CourseTune(i32, f32),
    FineTune(i32, f32),
    ClipperBoost(i32, f32),
    Parameter1(i32, f32),
    Parameter2(i32, f32),
    FilterCutoffFrequency(f32),
    FilterResonance(f32),
    FilterPoleCount(f32),
    FilterKeyTrackingAmount(f32),
    FilterEnvelopeAmount(f32),
    FilterLfoAmount(f32),
    FilterEnvelopeAttack(i32, f32),
    FilterEnvelopeDecay(i32, f32),
    FilterEnvelopeSustain(i32, f32),
    FilterEnvelopeRelease(i32, f32),
    FilterEnvelopeInvert(i32, bool),
    LfoFrequency(i32, f32),
    LfoShapeIndex(i32, i32),
    LfoPhase(i32, f32),
    LfoPhaseReset(i32),
    PortamentoEnabled(bool),
    PortamentoTime(f32),
    PitchBendRange(f32),
    VelocityCurve(f32),
    HardSyncEnabled(bool),
    KeySyncEnabled(bool),
    PolarityFlipped(bool),
    OutputBalance(f32),
    OutputLevel(f32),
    OutputMute(bool),
    OscillatorMixerBalance(i32, f32),
    OscillatorMixerLevel(i32, f32),
    OscillatorMixerMute(i32, bool),
    EffectEnabled(i32, bool),
    EffectParameterValues(i32, i32, f32),
}

#[derive(Debug, Clone, Copy, EnumCount, EnumIter, FromRepr)]
#[repr(i32)]
pub enum OscillatorIndex {
    Sub = 0,
    One = 1,
    Two = 2,
    Three = 3,
}

impl OscillatorIndex {
    pub fn from_i32(index: i32) -> Option<Self> {
        Self::from_repr(index)
    }
}

#[derive(Debug, Clone, Copy, EnumCount, EnumIter, FromRepr)]
#[repr(i32)]
pub enum LFOIndex {
    ModWheel = 0,
    Filter = 1,
}

impl LFOIndex {
    pub fn from_i32(index: i32) -> Option<Self> {
        Self::from_repr(index)
    }
}

#[derive(Debug, Clone, Copy, EnumCount, EnumIter, FromRepr)]
#[repr(i32)]
pub enum EnvelopeIndex {
    Amp = 0,
    Filter = 1,
}

impl EnvelopeIndex {
    pub fn from_i32(index: i32) -> Option<Self> {
        Self::from_repr(index)
    }
}
