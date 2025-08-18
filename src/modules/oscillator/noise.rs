use super::{GenerateSamples, WaveShape};

pub struct Noise {
    shape: WaveShape,
}

impl Noise {
    pub fn new() -> Self {
        log::info!("Constructing Noise WaveShape Module");
        Self {
            shape: WaveShape::Noise,
        }
    }
}

impl GenerateSamples for Noise {
    fn next_sample(&mut self, tone_frequency: f32, modulation: Option<f32>) -> f32 {
        if tone_frequency == 0.0 {
            return 0.0;
        }
        rand::random_range(-1.0..=1.0) * modulation.unwrap_or(1.0)
    }

    fn set_shape_parameter1(&mut self, _parameter: f32) {}

    fn set_shape_parameter2(&mut self, _parameter: f32) {}

    fn set_phase(&mut self, _phase: f32) {}

    fn shape(&self) -> WaveShape {
        self.shape
    }

    fn reset(&mut self) {}
}
