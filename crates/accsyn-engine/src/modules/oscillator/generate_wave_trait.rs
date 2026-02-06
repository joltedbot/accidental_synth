use crate::modules::oscillator::WaveShape;

pub trait GenerateWave {
    fn next_sample(&mut self, tone_frequency: f32, modulation: Option<f32>) -> f32;

    fn set_shape_parameter1(&mut self, parameter: f32);
    fn set_shape_parameter2(&mut self, parameter: f32);

    fn set_phase(&mut self, phase: f32);

    fn shape(&self) -> WaveShape;

    fn reset(&mut self);
}
