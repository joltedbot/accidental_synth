use super::constants::{MAX_PHASE, MIN_PHASE};
use super::{GenerateSamples, WaveShape};
use crate::modules::oscillator::sine::Sine;

const SHAPE: WaveShape = WaveShape::FM;
const DEFAULT_RATIO: f32 = 1.0;
const DEFAULT_MODULATION_AMOUNT: f32 = 1.0;
const AMOUNT_PARAMETER_MAX: f32 = 6000.0;
const RATIO_PARAMETER_MAX: f32 = 10.0;
const RATIO_PARAMETER_MIN: f32 = 0.01;
const RATIO_PARAMETER_CENTER_POINT: f32 = 0.5;

pub struct FM {
    shape: WaveShape,
    carrier: Box<dyn GenerateSamples + Send + Sync>,
    modulator: Box<dyn GenerateSamples + Send + Sync>,
    modulation_amount: f32,
    modulation_ratio: f32,
}

impl FM {
    pub fn new(sample_rate: u32) -> Self {
        log::info!("Constructing FM WaveShape Module");
        Self {
            shape: SHAPE,
            carrier: Box::new(Sine::new(sample_rate)),
            modulator: Box::new(Sine::new(sample_rate)),
            modulation_amount: DEFAULT_MODULATION_AMOUNT,
            modulation_ratio: DEFAULT_RATIO,
        }
    }
}

impl GenerateSamples for FM {
    fn next_sample(&mut self, tone_frequency: f32, modulation: Option<f32>) -> f32 {
        let modulator = self
            .modulator
            .next_sample(tone_frequency * self.modulation_ratio, modulation);
        let modulated_frequency = tone_frequency + (modulator * self.modulation_amount);
        self.carrier.next_sample(modulated_frequency, None)
    }

    fn set_shape_parameter1(&mut self, parameter: f32) {
        self.modulation_amount = parameter * AMOUNT_PARAMETER_MAX;
    }

    fn set_shape_parameter2(&mut self, parameter: f32) {
        self.modulation_ratio = if parameter == RATIO_PARAMETER_CENTER_POINT {
            DEFAULT_RATIO
        } else if parameter == 0.0 {
            RATIO_PARAMETER_MIN
        } else if parameter < RATIO_PARAMETER_CENTER_POINT {
            let scaled_parameter = parameter * 2.0;
            (scaled_parameter * 10.0).round() / 10.0
        } else {
            let scaled_parameter = (parameter - RATIO_PARAMETER_CENTER_POINT) * 2.0;
            (scaled_parameter * RATIO_PARAMETER_MAX).round()
        }
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
