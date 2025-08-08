use crate::modules::oscillator::GenerateSamples;
use crate::modules::oscillator::sine::Sine;

const DEFAULT_MODULATION_AMOUNT: f32 = 4.0;

pub struct AM {
    carrier: Box<dyn GenerateSamples + Send + Sync>,
    modulator: Box<dyn GenerateSamples + Send + Sync>,
    modulation_amount: f32,
}

impl AM {
    pub fn new(sample_rate: u32) -> Self {
        log::info!("Constructing AM WaveShape Module");
        Self {
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

    fn reset(&mut self) {
        self.carrier.reset();
        self.modulator.reset();
    }
}
