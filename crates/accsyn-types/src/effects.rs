use crate::defaults::{
    AUTOPAN_DEFAULT_PARAMETERS, CLIPPER_DEFAULT_PARAMETERS, COMPRESSOR_DEFAULT_PARAMETERS,
    DELAY_DEFAULT_PARAMETERS, GATE_DEFAULT_PARAMETERS, TREMOLO_DEFAULT_PARAMETERS,
};
use strum::IntoEnumIterator;
use strum_macros::{EnumCount, EnumIter, FromRepr};

pub const PARAMETERS_PER_EFFECT: usize = 4;

pub trait AudioEffect {
    fn process_samples(&mut self, samples: (f32, f32), effect: &EffectParameters) -> (f32, f32);
}

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

#[derive(Debug, Clone)]
pub struct EffectParameters {
    pub is_enabled: bool,
    pub parameters: Vec<f32>,
}

impl EffectParameters {
    pub fn default_all() -> Vec<Self> {
        let mut effect_parameters = Vec::new();

        for effect in EffectIndex::iter() {
            match effect {
                EffectIndex::WaveFolder
                | EffectIndex::Rectifier
                | EffectIndex::BitShifter
                | EffectIndex::Saturation => {
                    effect_parameters.push(EffectParameters::default());
                }
                EffectIndex::Clipper => {
                    effect_parameters.push(EffectParameters {
                        is_enabled: false,
                        parameters: CLIPPER_DEFAULT_PARAMETERS.to_vec(),
                    });
                }
                EffectIndex::Gate => {
                    effect_parameters.push(EffectParameters {
                        is_enabled: false,
                        parameters: GATE_DEFAULT_PARAMETERS.to_vec(),
                    });
                }
                EffectIndex::Compressor => {
                    effect_parameters.push(EffectParameters {
                        is_enabled: false,
                        parameters: COMPRESSOR_DEFAULT_PARAMETERS.to_vec(),
                    });
                }
                EffectIndex::Delay => {
                    effect_parameters.push(EffectParameters {
                        is_enabled: false,
                        parameters: DELAY_DEFAULT_PARAMETERS.to_vec(),
                    });
                }
                EffectIndex::AutoPan => {
                    effect_parameters.push(EffectParameters {
                        is_enabled: false,
                        parameters: AUTOPAN_DEFAULT_PARAMETERS.to_vec(),
                    });
                }
                EffectIndex::Tremolo => {
                    effect_parameters.push(EffectParameters {
                        is_enabled: false,
                        parameters: TREMOLO_DEFAULT_PARAMETERS.to_vec(),
                    });
                }
            }
        }

        effect_parameters
    }
}

impl Default for EffectParameters {
    fn default() -> Self {
        let parameters = vec![0.0; PARAMETERS_PER_EFFECT];
        Self {
            is_enabled: false,
            parameters,
        }
    }
}
