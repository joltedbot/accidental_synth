use crate::casting::f32_to_u16_clamped;
use anyhow::{Result, anyhow};
use strum::EnumCount as LfoEnumCount;
use strum_macros::{EnumCount, EnumIter, FromRepr};

/// Parameter change events sent to the synthesizer from the UI and MIDI subsystems.
#[derive(Debug)]
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
    /// Change oscillator Pitch Envelope Amount (oscillator index, normalized value).
    PitchEnvelopeAmount(i32, f32),
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
    /// Toggle the LFO clock sync to the clock (LFO index, is synced to clock).
    LfoClockSyncEnabled(i32, bool),
    /// Toggle the LFO key sync to the clock (LFO index, is synced to midi key press).
    LfoKeySyncEnabled(i32, bool),
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
    /// Sustain Pedal State on/off
    SustainPedal(bool),
    /// Change the output mixer stereo balance (normalized value).
    OutputBalance(f32),
    /// Change the output mixer level (normalized value).
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
    /// Change to a new patch (patch index).
    PatchChanged(i32),
    /// Save the current module parameters to a patch file
    PatchSaved(String),
    /// Delete the patch at the given index
    PatchDeleted(String),
    /// Clock ticks mark a 32nd note
    ThirtySecondNote,
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
    #[must_use]
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
    #[must_use]
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
    /// Pitch envelope (index 2)
    Pitch = 2,
}

impl EnvelopeIndex {
    /// Converts an i32 index to the corresponding envelope variant.
    #[must_use]
    pub fn from_i32(index: i32) -> Option<Self> {
        Self::from_repr(index)
    }
}

/// List of display names for time intervals for LFO when synced to a clock.
pub const LFO_SYNC_INTERVAL_NAMES: [&str; 21] = [
    "32/1", "24/1", "16/1", "12/1", "10/1", "8/1", "7/1", "6/1", "5/1", "4/1", "3/1", "2/1", "1/1",
    "1/2", "3/8", "1/4", "3/16", "1/8", "3/32", "1/16", "1/32",
];

#[derive(Debug, Clone, Copy, Default, EnumCount, EnumIter, FromRepr)]
#[repr(u16)]
/// List of time intervals for LFO when synced to a clock.
pub enum LfoSyncInterval {
    /// 32/1
    ThirtyTwoBars,
    /// 24/1
    TwentyFourBars,
    /// 16/1
    SixteenBars,
    /// 12/1
    TwelveBars,
    /// 10/1
    TenBars,
    /// 8/1
    EightBars,
    /// 7/1
    SevenBars,
    /// 6/1
    SixBars,
    /// 5/1
    FiveBars,
    /// 4/1
    FourBars,
    /// 3/1
    ThreeBars,
    /// 2/1
    TwoBars,
    #[default]
    /// 1/1
    OneBars,
    /// 1/2
    Half,
    /// 3/8
    DottedQuarter,
    /// 1/4
    Quarter,
    /// 3/16
    DottedEight,
    /// 1/8
    Eighth,
    /// 3/32
    DottedSixteenth,
    /// 1/16
    Sixteenth,
    /// 1/32
    ThirtySecond,
}

impl LfoSyncInterval {
    /// Converts a number of 32nd notes to an `LfoSyncInterval`
    ///
    /// # Errors
    ///
    /// Will return an error if the `ThirtySecondNotes` value does corespond to a valid sunc interval
    pub fn from_thirty_second_notes(thirty_second_notes: u16) -> Result<Self> {
        match thirty_second_notes {
            1024 => Ok(LfoSyncInterval::ThirtyTwoBars),
            762 => Ok(LfoSyncInterval::TwentyFourBars),
            512 => Ok(LfoSyncInterval::SixteenBars),
            384 => Ok(LfoSyncInterval::TwelveBars),
            320 => Ok(LfoSyncInterval::TenBars),
            256 => Ok(LfoSyncInterval::EightBars),
            224 => Ok(LfoSyncInterval::SevenBars),
            192 => Ok(LfoSyncInterval::SixBars),
            160 => Ok(LfoSyncInterval::FiveBars),
            128 => Ok(LfoSyncInterval::FourBars),
            96 => Ok(LfoSyncInterval::ThreeBars),
            64 => Ok(LfoSyncInterval::TwoBars),
            32 => Ok(LfoSyncInterval::OneBars),
            16 => Ok(LfoSyncInterval::Half),
            12 => Ok(LfoSyncInterval::DottedQuarter),
            8 => Ok(LfoSyncInterval::Quarter),
            6 => Ok(LfoSyncInterval::DottedEight),
            4 => Ok(LfoSyncInterval::Eighth),
            3 => Ok(LfoSyncInterval::DottedSixteenth),
            2 => Ok(LfoSyncInterval::Sixteenth),
            1 => Ok(LfoSyncInterval::ThirtySecond),
            _ => Err(anyhow!("Invalid Sync Interval")),
        }
    }

    /// Converts a time interval to the corresponding number of 32nd notes.
    #[must_use]
    pub fn to_thirty_second_notes(&self) -> u16 {
        match self {
            LfoSyncInterval::ThirtyTwoBars => 1024,
            LfoSyncInterval::TwentyFourBars => 762,
            LfoSyncInterval::SixteenBars => 512,
            LfoSyncInterval::TwelveBars => 384,
            LfoSyncInterval::TenBars => 320,
            LfoSyncInterval::EightBars => 256,
            LfoSyncInterval::SevenBars => 224,
            LfoSyncInterval::SixBars => 192,
            LfoSyncInterval::FiveBars => 160,
            LfoSyncInterval::FourBars => 128,
            LfoSyncInterval::ThreeBars => 96,
            LfoSyncInterval::TwoBars => 64,
            LfoSyncInterval::OneBars => 32,
            LfoSyncInterval::Half => 16,
            LfoSyncInterval::DottedQuarter => 12,
            LfoSyncInterval::Quarter => 8,
            LfoSyncInterval::DottedEight => 6,
            LfoSyncInterval::Eighth => 4,
            LfoSyncInterval::DottedSixteenth => 3,
            LfoSyncInterval::Sixteenth => 2,
            LfoSyncInterval::ThirtySecond => 1,
        }
    }

    /// Converts a time interval to the corresponding display string.
    #[must_use]
    pub fn display(self) -> String {
        LFO_SYNC_INTERVAL_NAMES[self as usize].to_string()
    }

    /// Converts a normal value into an `LfoSyncInterval` variant
    #[must_use]
    pub fn from_normal_value(normal_value: f32) -> LfoSyncInterval {
        let last_index = LfoSyncInterval::COUNT - 1;

        // The number of LfoSyncInterval enum variants is constrained to 24 options
        // and normal value is range [0.0, 1.0] It can never exceed the mantissa of an f32
        #[allow(clippy::cast_precision_loss)]
        let index = (last_index as f32 * normal_value).floor();
        LfoSyncInterval::from_repr(f32_to_u16_clamped(index)).unwrap_or_default()
    }
}
