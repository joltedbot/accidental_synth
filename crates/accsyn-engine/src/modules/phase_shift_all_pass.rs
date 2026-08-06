use std::f32::consts::PI;

const NYQUIST_FACTOR: f32 = 0.5;
const BASE_FREQUENCY: f32 = 800.0;
const MIN_FREQUENCY_HZ: f32 = 20.0;
const NYQUIST_SAFETY_FACTOR: f32 = 0.49;
const MODULATION_OCTAVE_RANGE: f32 = 4.0;

/// An all pass filter for use in other modules
pub struct PhaseShiftAllPass {
    sample_rate: f32,
    frequency: f32,
    nyquist: f32,
    previous_input_samples: (f32, f32),
    previous_output_samples: (f32, f32),
}

impl PhaseShiftAllPass {
    /// Creates a new all pass filter instance with the current sample rate
    #[must_use]
    pub fn new(sample_rate: u32) -> Self {
        // Sample rate is always ≤ 192_000, within f32 precision (2²³ = 8_388_608)
        #[allow(clippy::cast_precision_loss)]
        let float_sample_rate = sample_rate as f32;

        Self {
            sample_rate: float_sample_rate,
            nyquist: float_sample_rate * NYQUIST_FACTOR,
            frequency: BASE_FREQUENCY,
            previous_input_samples: (0.0, 0.0),
            previous_output_samples: (0.0, 0.0),
        }
    }

    /// Set the initial frequency mid-point
    pub fn set_frequency(&mut self, frequency: f32) {
        self.frequency = frequency;
    }

    /// Process the input samples through the All Pass filter
    pub fn process(&mut self, samples: (f32, f32), modulation: f32) -> (f32, f32) {
        let current_frequency = (self.frequency * 2f32.powf(modulation * MODULATION_OCTAVE_RANGE))
            .clamp(MIN_FREQUENCY_HZ, self.nyquist * NYQUIST_SAFETY_FACTOR);

        let coefficient = calculate_coefficient(current_frequency, self.sample_rate);

        let left_output_sample = all_pass(
            coefficient,
            samples.0,
            self.previous_input_samples.0,
            self.previous_output_samples.0,
        );

        let right_output_sample = all_pass(
            coefficient,
            samples.1,
            self.previous_input_samples.1,
            self.previous_output_samples.1,
        );

        self.previous_input_samples = (samples.0, samples.1);
        self.previous_output_samples = (left_output_sample, right_output_sample);

        self.previous_output_samples
    }
}

fn calculate_coefficient(frequency: f32, sample_rate: f32) -> f32 {
    ((PI * frequency / sample_rate).tan() - 1.0) / (((PI * frequency / sample_rate).tan()) + 1.0)
}

fn all_pass(
    coefficient: f32,
    input_sample: f32,
    previous_input_sample: f32,
    previous_output_sample: f32,
) -> f32 {
    coefficient * input_sample + previous_input_sample - coefficient * previous_output_sample
}
