use crate::modules::oscillator::GenerateSamples;
use crate::modules::oscillator::sine::Sine;

const DEFAULT_RATIO: f32 = 1.0;
const DEFAULT_MODULATION_AMOUNT: f32 = 1.0;

pub struct FM {
    carrier: Box<dyn GenerateSamples + Send + Sync>,
    modulator: Box<dyn GenerateSamples + Send + Sync>,
    modulation_amount: f32,
    modulation_ratio: f32,
}

impl FM {
    pub fn new(sample_rate: u32) -> Self {
        log::info!("Constructing FM WaveShape Module");
        Self {
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

    fn set_shape_parameters(&mut self, parameters: Vec<f32>) {
        self.modulation_amount = parameters[0];
        self.modulation_ratio = parameters[1];
    }

    fn reset(&mut self) {
        self.carrier.reset();
        self.modulator.reset();
    }
}
