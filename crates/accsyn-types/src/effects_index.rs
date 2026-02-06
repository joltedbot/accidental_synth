use strum_macros::{EnumCount, EnumIter, FromRepr};

#[derive(Debug, Clone, Copy, EnumCount, EnumIter, FromRepr)]
#[repr(i32)]
pub enum EffectIndex {
    WaveFolder,
    Clipper,
    Gate,
    Rectifier,
    BitShifter,
    Saturation,
    Compressor,
    Delay,
    AutoPan,
    Tremolo,
}

impl EffectIndex {
    pub fn from_i32(index: i32) -> Option<Self> {
        Self::from_repr(index)
    }
}
