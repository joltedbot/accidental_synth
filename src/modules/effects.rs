use crate::math::load_f32_from_atomic_u32;
use crate::modules::effects::constants::{
    MAX_WAVEFOLDER_THRESHOLD, NUMBER_OF_EFFECTS, PARAMETERS_PER_EFFECT,
};
use crate::modules::effects::wavefolder::WaveFolder;
use coreaudio_sys::os_workgroup_attr_s;
use std::char::MAX;
use std::sync::atomic::Ordering::Relaxed;
use std::sync::atomic::{AtomicBool, AtomicU32};

mod constants;
pub mod wavefolder;

pub trait AudioEffect {
    fn process_samples(&self, samples: (f32, f32), effect: &EffectParameters) -> (f32, f32);
}

#[derive(Debug, Clone, Copy)]
pub enum EffectIndex {
    WaveFolder,
}

impl EffectIndex {
    pub fn count() -> usize {
        NUMBER_OF_EFFECTS
    }

    pub fn from_i32(index: i32) -> Option<Self> {
        match index {
            0 => Some(EffectIndex::WaveFolder),
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

#[derive(Debug, Default)]
pub struct EffectParameters {
    is_enabled: bool,
    parameters: Vec<f32>,
}


pub struct Effects {
    effects: Vec<Box<dyn AudioEffect>>,
    parameters: [EffectParameters; NUMBER_OF_EFFECTS],
}

impl Effects {
    pub fn new() -> Self {
        let wavefolder = Box::new(WaveFolder::new());
        let wavefolder_parameters = EffectParameters::default();

        Self {
            effects: vec![wavefolder],
            parameters: [wavefolder_parameters],
        }
    }

    pub fn set_parameters(&mut self, parameters: &Vec<AudioEffectParameters>) {
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
                samples = self.effects[index].process_samples(samples, &parameter)
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
            .map(|parameter| load_f32_from_atomic_u32(&parameter)).collect(),
    }
}
