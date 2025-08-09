use super::sine::Sine;
use super::{GenerateSamples, WaveShape};

const SHAPE: WaveShape = WaveShape::AM;
const DEFAULT_MODULATION_AMOUNT: f32 = 4.0;

pub struct AM {
    shape: WaveShape,
    carrier: Box<dyn GenerateSamples + Send + Sync>,
    modulator: Box<dyn GenerateSamples + Send + Sync>,
    modulation_amount: f32,
}

impl AM {
    pub fn new(sample_rate: u32) -> Self {
        log::info!("Constructing AM WaveShape Module");
        Self {
            shape: SHAPE,
            carrier: Box::new(Sine::new(sample_rate)),
            modulator: Box::new(Sine::new(sample_rate)),
            modulation_amount: DEFAULT_MODULATION_AMOUNT,
        }
    }
}

impl GenerateSamples for AM {
    fn next_sample(&mut self, tone_frequency: f32, modulation: Option<f32>) -> f32 {
        let modulator = self
            .modulator
            .next_sample(tone_frequency * self.modulation_amount, None);
        self.carrier.next_sample(tone_frequency, modulation) * modulator
    }

    fn set_shape_parameters(&mut self, parameters: Vec<f32>) {
        self.modulation_amount = parameters[0];
    }

    fn shape(&self) -> WaveShape {
        self.shape
    }

    fn reset(&mut self) {
        self.carrier.reset();
        self.modulator.reset();
    }
}
