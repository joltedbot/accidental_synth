use crate::defaults::{
    AUTOPAN_DEFAULT_PARAMETERS, CLIPPER_DEFAULT_PARAMETERS, COMPRESSOR_DEFAULT_PARAMETERS,
    DEFAULT_EFFECT_PARAMETERS, DELAY_DEFAULT_PARAMETERS, GATE_DEFAULT_PARAMETERS,
    SATURATION_DEFAULT_PARAMETERS, TREMOLO_DEFAULT_PARAMETERS,
};
use std::string::ToString;
use strum::IntoEnumIterator;
use strum_macros::{Display, EnumCount, EnumIter, FromRepr};

/// Number of adjustable parameters per audio effect.
pub const PARAMETERS_PER_EFFECT: usize = 4;

/// Trait for audio effects that process stereo sample pairs.
pub trait AudioEffect {
    /// Processes a stereo sample pair through the effect and returns the result.
    fn process_samples(&mut self, samples: (f32, f32), effect: &EffectParameters) -> (f32, f32);
}

/// Index identifying each available audio effect type.
#[derive(Debug, Display, Clone, Copy, EnumCount, EnumIter, FromRepr, Eq, PartialEq, Hash)]
#[repr(i32)]
pub enum EffectIndex {
    /// Soft saturation distortion effect.
    #[strum(to_string = "Saturation")]
    Saturation,
    /// Dynamic range compressor effect.
    #[strum(to_string = "Colour Compressor")]
    Compressor,
    /// Wavefolder distortion effect.
    #[strum(to_string = "Wave Folder")]
    WaveFolder,
    /// Bit-shifting digital distortion effect.
    #[strum(to_string = "Bit Shifter")]
    BitShifter,
    /// Hard clipper distortion effect.
    #[strum(to_string = "Clipper")]
    Clipper,
    /// Gate Clipping effect.
    #[strum(to_string = "Gate Clipping")]
    Gate,
    /// Full-wave rectifier distortion effect.
    #[strum(to_string = "Full/Half Wave Rectifier")]
    Rectifier,
    /// Automatic stereo panning effect.
    #[strum(to_string = "Auto-Pan")]
    AutoPan,
    /// Amplitude tremolo modulation effect.
    #[strum(to_string = "Tremolo")]
    Tremolo,
    /// Stereo delay effect with feedback.
    #[strum(to_string = "Delay")]
    Delay,
}

impl EffectIndex {
    /// Converts an i32 index to the corresponding effect variant.
    #[must_use]
    pub fn from_i32(index: i32) -> Option<Self> {
        Self::from_repr(index)
    }
}

/// Runtime parameters for a single audio effect instance.
#[derive(Debug, Clone)]
pub struct EffectParameters {
    /// Effect Name
    pub name: String,
    /// Whether this effect is currently active in the signal chain.
    pub is_enabled: bool,
    /// The effect's adjustable parameter values.
    pub parameters: Vec<f32>,
}

impl EffectParameters {
    /// Creates default parameters for all effect types.
    #[must_use]
    pub fn default_all() -> Vec<Self> {
        let mut effect_parameters = Vec::new();

        for effect in EffectIndex::iter() {
            match effect {
                EffectIndex::WaveFolder | EffectIndex::Rectifier | EffectIndex::BitShifter => {
                    effect_parameters.push(EffectParameters {
                        name: effect.to_string(),
                        is_enabled: false,
                        parameters: DEFAULT_EFFECT_PARAMETERS.to_vec(),
                    });
                }
                EffectIndex::Saturation => {
                    effect_parameters.push(EffectParameters {
                        name: effect.to_string(),
                        is_enabled: false,
                        parameters: SATURATION_DEFAULT_PARAMETERS.to_vec(),
                    });
                }
                EffectIndex::Clipper => {
                    effect_parameters.push(EffectParameters {
                        name: effect.to_string(),
                        is_enabled: false,
                        parameters: CLIPPER_DEFAULT_PARAMETERS.to_vec(),
                    });
                }
                EffectIndex::Gate => {
                    effect_parameters.push(EffectParameters {
                        name: effect.to_string(),
                        is_enabled: false,
                        parameters: GATE_DEFAULT_PARAMETERS.to_vec(),
                    });
                }
                EffectIndex::Compressor => {
                    effect_parameters.push(EffectParameters {
                        name: effect.to_string(),
                        is_enabled: false,
                        parameters: COMPRESSOR_DEFAULT_PARAMETERS.to_vec(),
                    });
                }
                EffectIndex::Delay => {
                    effect_parameters.push(EffectParameters {
                        name: effect.to_string(),
                        is_enabled: false,
                        parameters: DELAY_DEFAULT_PARAMETERS.to_vec(),
                    });
                }
                EffectIndex::AutoPan => {
                    effect_parameters.push(EffectParameters {
                        name: effect.to_string(),
                        is_enabled: false,
                        parameters: AUTOPAN_DEFAULT_PARAMETERS.to_vec(),
                    });
                }
                EffectIndex::Tremolo => {
                    effect_parameters.push(EffectParameters {
                        name: effect.to_string(),
                        is_enabled: false,
                        parameters: TREMOLO_DEFAULT_PARAMETERS.to_vec(),
                    });
                }
            }
        }

        effect_parameters
    }
}
