use super::WaveShape;
use crate::modules::oscillator::constants::RADS_PER_CYCLE;
use crate::modules::oscillator::generate_wave_trait::GenerateWave;
use crate::modules::oscillator::sine::Sine;
use accsyn_core::casting::f64_to_f32_clamped;

const SHAPE: WaveShape = WaveShape::PM;
const DEFAULT_CARRIER_PHASE: f64 = 0.0;
const DEFAULT_MODULATION_AMOUNT: f64 = 0.229;
const DEFAULT_MODULATION_INDEX: f64 = 16.0;
const PM_INDEX_CURVE_POWER_COEFFICIENT: f64 = 2.125;

/// Frequency modulation oscillator using carrier and modulator sine waves.
pub struct PM {
    shape: WaveShape,
    modulator: Box<dyn GenerateWave + Send + Sync>,
    modulation_amount: f64,
    carrier_phase: f64,
    phase_coefficient: f64,
    modulator_index: f64,
}

impl PM {
    /// Creates a new PM oscillator with default modulation amount and ratio.
    pub(crate) fn new(sample_rate: u32) -> Self {
        log::debug!(target: "synth::oscillator", shape = "PM"; "Constructing wave generator");

        let seconds_per_sample = 1.0 / f64::from(sample_rate);
        let phase_coefficient = RADS_PER_CYCLE * seconds_per_sample;

        Self {
            shape: SHAPE,
            modulator: Box::new(Sine::new(sample_rate)),
            modulation_amount: DEFAULT_MODULATION_AMOUNT,
            carrier_phase: DEFAULT_CARRIER_PHASE,
            phase_coefficient,
            modulator_index: DEFAULT_MODULATION_INDEX,
        }
    }

    fn single_sine_sample(&mut self, tone_frequency: f32, modulator: f64) -> f64 {
        self.carrier_phase += self.phase_coefficient * f64::from(tone_frequency);

        if self.carrier_phase >= RADS_PER_CYCLE {
            self.carrier_phase -= RADS_PER_CYCLE;
        }
        let offset_rads = self.modulator_index
            * self
                .modulation_amount
                .powf(PM_INDEX_CURVE_POWER_COEFFICIENT)
            * modulator;

        (self.carrier_phase + offset_rads).sin()
    }
}

impl GenerateWave for PM {
    fn next_sample(&mut self, tone_frequency: f32, modulation: Option<f32>) -> f32 {
        let new_frequency = tone_frequency * modulation.unwrap_or(1.0);
        let modulator = self.modulator.next_sample(tone_frequency, None);

        f64_to_f32_clamped(self.single_sine_sample(new_frequency, f64::from(modulator)))
    }

    fn set_shape_parameter1(&mut self, parameter: f32) {
        self.modulation_amount = f64::from(parameter);
    }

    fn set_shape_parameter2(&mut self, _parameter: f32) {}

    fn set_phase(&mut self, phase: f32) {
        self.carrier_phase = f64::from(phase);
    }

    fn shape(&self) -> WaveShape {
        self.shape
    }

    fn reset(&mut self) {
        self.carrier_phase = DEFAULT_CARRIER_PHASE;
        self.modulator.reset();
    }
}
