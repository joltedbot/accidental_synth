use crate::modules::oscillator::GenerateSamples;

const RADS_PER_CYCLE: f32 = 2.0 * std::f32::consts::PI;
const DEFAULT_PHASE: f32 = 0.0;

pub struct Sine {
    pub phase: f32,
    pub phase_increment: f32,
}

impl Sine {
    pub fn new(sample_rate: u32) -> Self {
        log::info!("Constructing Sine WaveShape Module");
        let phase: f32 = DEFAULT_PHASE;
        let seconds_per_sample = 1.0 / sample_rate as f32;
        let phase_increment = RADS_PER_CYCLE * seconds_per_sample;

        Self {
            phase,
            phase_increment,
        }
    }
}
impl GenerateSamples for Sine {
    fn next_sample(&mut self, tone_frequency: f32, modulation: Option<f32>) -> f32 {
        let new_frequency = tone_frequency * modulation.unwrap_or(1.0);
        self.phase += self.phase_increment * new_frequency;

        if self.phase >= RADS_PER_CYCLE {
            self.reset();
        }

        self.phase.sin()
    }

    fn reset(&mut self) {
        self.phase = DEFAULT_PHASE;
    }
}
