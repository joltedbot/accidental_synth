use super::WaveShape;
use crate::modules::oscillator::constants::{
    DEFAULT_X_COORDINATE, DEFAULT_X_INCREMENT, RADS_PER_CYCLE,
};
use crate::modules::oscillator::generate_wave_trait::GenerateWave;
use accsyn_core::casting::f64_to_f32_clamped;
use std::f64::consts::PI;

/// Triangle wave oscillator using arcsine shaping.
pub struct Triangle {
    shape: WaveShape,
    x_coordinate: f64,
    sample_rate: u32,
    phase: Option<f64>,
}

impl Triangle {
    pub(crate) fn new(sample_rate: u32) -> Self {
        log::debug!(target: "synth::oscillator", shape = "Triangle"; "Constructing wave generator");
        let x_coordinate = DEFAULT_X_COORDINATE;

        Self {
            shape: WaveShape::Triangle,
            x_coordinate,
            sample_rate,
            phase: None,
        }
    }
}

impl GenerateWave for Triangle {
    fn next_sample(&mut self, tone_frequency: f32, modulation: Option<f32>) -> f32 {
        let sample_rate_f64 = f64::from(self.sample_rate);
        let tone_frequency_f64 = f64::from(tone_frequency);
        let period = sample_rate_f64 / tone_frequency_f64;
        let new_x_increment = DEFAULT_X_INCREMENT * f64::from(modulation.unwrap_or(1.0));

        if let Some(phase) = self.phase {
            self.x_coordinate = (phase / RADS_PER_CYCLE) * period;
            self.phase = None;
        }

        let normalized_x_increment = new_x_increment / period;
        let normalized_x_coordinate = (self.x_coordinate / period).rem_euclid(1.0);

        let mut y_coordinate = 2.0 / PI
            * (tone_frequency_f64 * RADS_PER_CYCLE * (self.x_coordinate / sample_rate_f64))
                .sin()
                .asin();

        let scale = 4.0 * normalized_x_increment;
        y_coordinate -= scale
            * poly_blamp(
                (normalized_x_coordinate - 0.25).rem_euclid(1.0),
                normalized_x_increment,
            );
        y_coordinate += scale
            * poly_blamp(
                (normalized_x_coordinate - 0.75).rem_euclid(1.0),
                normalized_x_increment,
            );

        self.x_coordinate += new_x_increment;

        if tone_frequency > 0.0 && self.x_coordinate >= period {
            self.x_coordinate -= period;
        }
        f64_to_f32_clamped(y_coordinate)
    }

    fn set_shape_parameter1(&mut self, _parameter: f32) {}

    fn set_shape_parameter2(&mut self, _parameter: f32) {}

    fn set_phase(&mut self, phase: f32) {
        self.phase = Some(f64::from(phase));
    }

    fn shape(&self) -> WaveShape {
        self.shape
    }

    fn reset(&mut self) {
        self.x_coordinate = DEFAULT_X_COORDINATE;
    }
}

fn poly_blamp(mut normalized_phase: f64, phase_increment: f64) -> f64 {
    if phase_increment <= 0.0 {
        return 0.0;
    }

    if normalized_phase < phase_increment {
        normalized_phase = normalized_phase / phase_increment - 1.0;
        return -1.0 / 3.0 * (normalized_phase * normalized_phase * normalized_phase);
    }

    if normalized_phase > (1.0 - phase_increment) {
        normalized_phase = (normalized_phase - 1.0) / phase_increment + 1.0;
        return 1.0 / 3.0 * (normalized_phase * normalized_phase * normalized_phase);
    }

    0.0
}

/*



// Setup oscillator.
double freq = 220; // Hz
double phase = 0;
double phase_inc = freq / sample_rate;

// Generate samples.
for (int i = 0; i < num_samples; ++i)
{
    // Start with naive triangle.
    double sample = 4 * phase;
    if (sample >= 3)
    {
        sample = sample - 4;
    }
    else if (sample > 1)
    {
        sample = 2 - sample;
    }

    // Correct falling discontinuity.
    double scale = 4 * phase_inc;
    double phase2 = phase + 0.25;
    phase2 = phase2 - floor(phase2);
    sample = sample + scale * poly_blamp(phase2, phase_inc);

    // Correct rising discontinuity.
    phase2 = phase2 + 0.5;
    phase2 = phase2 - floor(phase2);
    sample = sample - scale * poly_blamp(phase2, phase_inc);

    // Increment phase for next sample.
    phase = phase + phase_inc;
    phase = phase - floor(phase);

    // Output current sample.
    output_buffer[i] = sample;
}
 */
