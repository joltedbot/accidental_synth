use super::constants::{DEFAULT_AM_TONE_AMOUNT, DEFAULT_AMPLITUDE_MODULATION_AMOUNT};
use super::sine::Sine;
use super::{GenerateSamples, WaveShape};
use crate::modules::oscillator::constants::{MAX_PHASE, MIN_PHASE};

pub struct AM {
    shape: WaveShape,
    carrier: Box<dyn GenerateSamples + Send + Sync>,
    modulator: Box<dyn GenerateSamples + Send + Sync>,
    modulation_amount: f32,
    am_tone_amount: f32, // 0.0 is ring modulation, 1.0 is proper AM
}

impl AM {
    pub fn new(sample_rate: u32) -> Self {
        log::info!("Constructing AM WaveShape Module");
        Self {
            shape: WaveShape::AM,
            carrier: Box::new(Sine::new(sample_rate)),
            modulator: Box::new(Sine::new(sample_rate)),
            modulation_amount: DEFAULT_AMPLITUDE_MODULATION_AMOUNT,
            am_tone_amount: DEFAULT_AM_TONE_AMOUNT,
        }
    }
}

impl GenerateSamples for AM {
    fn next_sample(&mut self, tone_frequency: f32, modulation: Option<f32>) -> f32 {
        let modulator = self
            .modulator
            .next_sample(tone_frequency * self.modulation_amount, modulation);

        let am_tone_adjusted_modulator = modulator + (1.0 * self.am_tone_amount);

        self.carrier.next_sample(tone_frequency, None) * am_tone_adjusted_modulator
    }

    fn set_shape_parameter1(&mut self, parameter: f32) {
        self.modulation_amount = parameter;
    }

    fn set_shape_parameter2(&mut self, parameter: f32) {
        self.am_tone_amount = parameter * 0.5;
    }

    fn set_phase(&mut self, phase: f32) {
        self.carrier.set_phase(phase.clamp(MIN_PHASE, MAX_PHASE));
    }

    fn shape(&self) -> WaveShape {
        self.shape
    }

    fn reset(&mut self) {
        self.carrier.reset();
        self.modulator.reset();
    }
}
