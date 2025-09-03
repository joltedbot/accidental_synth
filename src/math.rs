#![allow(dead_code)]
use std::sync::atomic::AtomicU32;
use std::sync::atomic::Ordering::Relaxed;

const ALTERNATE_EPSILON: f32 = 1e-7; // try 1e-7 if there are precision issues

// Convert a dbfs value to an f32 value that can be used to adjust the level of audio samples
pub fn dbfs_to_sample(dbfs: f32) -> f32 {
    10.0_f32.powf(dbfs / 20.0)
}

// Converts an f32 sample value to a dbfs value
pub fn sample_to_dbfs(sample: f32) -> f32 {
    let sample_absolute_value = sample.abs();
    if sample_absolute_value <= ALTERNATE_EPSILON {
        return f32::NEG_INFINITY;
    }
    20.0 * sample_absolute_value.log10()
}

// Determine if 2 f32 values are equal to within standard floating point precision
pub fn f32s_are_equal(value_1: f32, value_2: f32) -> bool {
    (value_1 - value_2).abs() <= ALTERNATE_EPSILON
}

//
pub fn store_f32_as_atomic_u32(atomic: &AtomicU32, value: f32) {
    atomic.store(value.to_bits(), Relaxed);
}

pub fn load_f32_from_atomic_u32(atomic: &AtomicU32) -> f32 {
    f32::from_bits(atomic.load(Relaxed))
}

pub fn frequency_from_cents(frequency: f32, cents: i16) -> f32 {
    let cents_per_octave = 1200.0;
    frequency * (2.0f32.powf(f32::from(cents) / cents_per_octave))
}

// The exponential function using natural log, output normalized to the range [0, 1]
fn exponential_function_natural_log(
    input_value: f32,
    input_scale_min: f32,
    input_scale_max: f32,
    output_scale_min: f32,
    output_scale_max: f32,
) -> f32 {
    let b = (output_scale_max.ln() - output_scale_min.ln()) / (input_scale_max - input_scale_min);
    let a = output_scale_min / (b * input_scale_min).exp();
    let result = a * (b * input_value).exp();
    (result - output_scale_min) / (output_scale_max - output_scale_min)
}
