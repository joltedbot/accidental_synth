use crate::math::load_f32_from_atomic_u32;
use crate::modules::effects::clipper::Clipper;
use crate::modules::effects::constants::{
    MAX_CLIPPER_THRESHOLD, PARAMETER_DISABLED_VALUE, PARAMETERS_PER_EFFECT,
};
use crate::modules::effects::rectifier::Rectifier;
use crate::modules::effects::wavefolder::WaveFolder;
use std::sync::atomic::Ordering::Relaxed;
use std::sync::atomic::{AtomicBool, AtomicU32};

mod clipper;
mod constants;
mod rectifier;
mod wavefolder;

pub trait AudioEffect {
    fn process_samples(&self, samples: (f32, f32), effect: &EffectParameters) -> (f32, f32);
}

pub const NUMBER_OF_EFFECTS: usize = 3;

#[derive(Debug, Clone, Copy)]
pub enum EffectIndex {
    WaveFolder,
    Clipper,
    Rectifier,
}

impl EffectIndex {
    pub fn count() -> usize {
        NUMBER_OF_EFFECTS
    }

    pub fn from_i32(index: i32) -> Option<Self> {
        match index {
            0 => Some(EffectIndex::WaveFolder),
            1 => Some(EffectIndex::Clipper),
            2 => Some(EffectIndex::Rectifier),
            _ => None,
        }
    }
}

#[derive(Debug)]
pub struct AudioEffectParameters {
    pub is_enabled: AtomicBool,
    pub parameters: [AtomicU32; PARAMETERS_PER_EFFECT],
}

impl Default for AudioEffectParameters {
    fn default() -> Self {
        Self {
            is_enabled: AtomicBool::new(false),
            parameters: Default::default(),
        }
    }
}

#[derive(Debug)]
pub struct EffectParameters {
    is_enabled: bool,
    parameters: Vec<f32>,
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

pub struct Effects {
    effects: Vec<Box<dyn AudioEffect>>,
    parameters: [EffectParameters; NUMBER_OF_EFFECTS],
}

impl Effects {
    pub fn new() -> Self {
        let wavefolder = Box::new(WaveFolder::new());
        let wavefolder_parameters = EffectParameters::default();

        let clipper = Box::new(Clipper::new());
        let mut clipper_parameters = EffectParameters::default();
        clipper_parameters.parameters[0] = MAX_CLIPPER_THRESHOLD;

        let rectifier = Box::new(Rectifier::new());
        let rectifier_parameters = EffectParameters::default();

        Self {
            effects: vec![wavefolder, clipper, rectifier],
            parameters: [
                wavefolder_parameters,
                clipper_parameters,
                rectifier_parameters,
            ],
        }
    }

    pub fn set_parameters(&mut self, parameters: &[AudioEffectParameters]) {
        parameters
            .iter()
            .enumerate()
            .for_each(|(index, effect_parameters)| {
                self.parameters[index] = extract_parameters(effect_parameters);
            });
    }

    pub fn process(&self, mut samples: (f32, f32)) -> (f32, f32) {
        for (index, parameter) in self.parameters.iter().enumerate() {
            if parameter.is_enabled {
                samples = self.effects[index].process_samples(samples, parameter);
            }
        }

        samples
    }
}

fn extract_parameters(source: &AudioEffectParameters) -> EffectParameters {
    EffectParameters {
        is_enabled: source.is_enabled.load(Relaxed),
        parameters: source
            .parameters
            .iter()
            .map(load_f32_from_atomic_u32)
            .collect(),
    }
}

pub fn default_audio_effect_parameters() -> Vec<AudioEffectParameters> {
    let mut audio_effect_parameters = Vec::new();

    for index in 0..EffectIndex::count() {
        if let Some(index) = EffectIndex::from_i32(index as i32) {
            match index {
                EffectIndex::WaveFolder => {
                    audio_effect_parameters.push(AudioEffectParameters {
                        is_enabled: AtomicBool::new(false),
                        parameters: [
                            AtomicU32::new(0.0_f32.to_bits()),
                            AtomicU32::new(PARAMETER_DISABLED_VALUE.to_bits()),
                            AtomicU32::new(PARAMETER_DISABLED_VALUE.to_bits()),
                            AtomicU32::new(PARAMETER_DISABLED_VALUE.to_bits()),
                        ],
                    });
                }
                EffectIndex::Clipper => {
                    audio_effect_parameters.push(AudioEffectParameters {
                        is_enabled: AtomicBool::new(false),
                        parameters: [
                            AtomicU32::new(MAX_CLIPPER_THRESHOLD.to_bits()),
                            AtomicU32::new(0.0_f32.to_bits()),
                            AtomicU32::new(0.0_f32.to_bits()),
                            AtomicU32::new(PARAMETER_DISABLED_VALUE.to_bits()),
                        ],
                    });
                }
                EffectIndex::Rectifier => {
                    audio_effect_parameters.push(AudioEffectParameters {
                        is_enabled: AtomicBool::new(false),
                        parameters: [
                            AtomicU32::new(0.0_f32.to_bits()),
                            AtomicU32::new(PARAMETER_DISABLED_VALUE.to_bits()),
                            AtomicU32::new(PARAMETER_DISABLED_VALUE.to_bits()),
                            AtomicU32::new(PARAMETER_DISABLED_VALUE.to_bits()),
                        ],
                    });
                }
            }
        }
    }

    audio_effect_parameters
}
