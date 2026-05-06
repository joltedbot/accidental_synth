use super::WaveShape;
use super::constants::{DEFAULT_AMPLITUDE_MODULATION_AMOUNT, DEFAULT_RING_MOD_AMOUNT};
use super::sine::Sine;
use crate::modules::oscillator::constants::{MAX_PHASE, MIN_PHASE};
use crate::modules::oscillator::generate_wave_trait::GenerateWave;

/// Amplitude modulation oscillator using carrier and modulator sine waves.
pub struct AM {
    shape: WaveShape,
    carrier: Box<dyn GenerateWave + Send + Sync>,
    modulator: Box<dyn GenerateWave + Send + Sync>,
    modulation_amount: f32,
    ring_mod_amount: f32, // 1.0 is full ring modulation, 0.0 is proper AM
}

impl AM {
    /// Creates a new AM oscillator with default modulation depth and tone amount.
    pub(crate) fn new(sample_rate: u32) -> Self {
        log::debug!(target: "synth::oscillator", shape = "AM"; "Constructing wave generator");
        Self {
            shape: WaveShape::AM,
            carrier: Box::new(Sine::new(sample_rate)),
            modulator: Box::new(Sine::new(sample_rate)),
            modulation_amount: DEFAULT_AMPLITUDE_MODULATION_AMOUNT,
            ring_mod_amount: DEFAULT_RING_MOD_AMOUNT,
        }
    }
}

impl GenerateWave for AM {
    fn next_sample(&mut self, tone_frequency: f32, modulation: Option<f32>) -> f32 {
        let modulator = self
            .modulator
            .next_sample(tone_frequency * self.modulation_amount, modulation);

        let am_tone_adjusted_modulator = modulator + (1.0 - self.ring_mod_amount);

        self.carrier.next_sample(tone_frequency, None) * am_tone_adjusted_modulator
    }

    fn set_shape_parameter1(&mut self, parameter: f32) {
        self.modulation_amount = parameter;
    }

    fn set_shape_parameter2(&mut self, parameter: f32) {
        self.ring_mod_amount = parameter;
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
