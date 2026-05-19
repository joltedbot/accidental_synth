// These functions are only used in the main real-time audio hot loop where maximum performance is required.
#![allow(clippy::inline_always)]
const I24_ALIGNED_HIGH_I32_MAX_VALUE: f32 = 2_147_483_392.0;
const I16_MAX_F32: f32 = 32767.0;
const I8_MAX_F32: f32 = 127.0;

#[inline(always)]
#[allow(clippy::cast_possible_truncation)]
pub fn f32_to_int24_aligned_high(sample: f32) -> i32 {
    let scaled = sample * I24_ALIGNED_HIGH_I32_MAX_VALUE;
    let clamped = scaled.clamp(
        -I24_ALIGNED_HIGH_I32_MAX_VALUE,
        I24_ALIGNED_HIGH_I32_MAX_VALUE,
    );
    (clamped as i32) & !0xFF
}

#[inline(always)]
#[allow(clippy::cast_possible_truncation)]
pub fn f32_to_i16(sample: f32) -> i16 {
    let scaled = sample * I16_MAX_F32;
    scaled.clamp(-I16_MAX_F32, I16_MAX_F32) as i16
}

#[inline(always)]
#[allow(clippy::cast_possible_truncation)]
pub fn f32_to_i8(sample: f32) -> i8 {
    let scaled = sample * I8_MAX_F32;
    scaled.clamp(-I8_MAX_F32, I8_MAX_F32) as i8
}

/// Converts a normalized f32 audio sample into a concrete device sample type
///
/// Implemented for the four sample types supported by `CoreAudio's` render callback:
/// `f32` (pass-through), `i32` (24-bit high-aligned in 32-bit container), `i16`, and `i8`.
pub trait SampleConvert: 'static {
    fn from_f32_sample(sample: f32) -> Self;
}

impl SampleConvert for f32 {
    #[inline(always)]
    fn from_f32_sample(sample: f32) -> Self {
        sample
    }
}

impl SampleConvert for i32 {
    #[inline(always)]
    fn from_f32_sample(sample: f32) -> Self {
        f32_to_int24_aligned_high(sample)
    }
}

impl SampleConvert for i16 {
    #[inline(always)]
    fn from_f32_sample(sample: f32) -> Self {
        f32_to_i16(sample)
    }
}

impl SampleConvert for i8 {
    #[inline(always)]
    fn from_f32_sample(sample: f32) -> Self {
        f32_to_i8(sample)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn f32_to_int24_aligned_high_converts_positive_full_scale() {
        let result = f32_to_int24_aligned_high(1.0);
        let expected = 0x7FFF_FF00_i32;
        assert_eq!(
            result, expected,
            "Expected: {expected:#010X}, got: {result:#010X}"
        );
    }

    #[test]
    fn f32_to_int24_aligned_high_converts_negative_full_scale() {
        let result = f32_to_int24_aligned_high(-1.0);
        let expected = (-2_147_483_392_i32) & !0xFF;
        assert_eq!(result, expected, "Expected: {expected}, got: {result}");
    }

    #[test]
    fn f32_to_int24_aligned_high_converts_zero() {
        let result = f32_to_int24_aligned_high(0.0);
        assert_eq!(result, 0, "Expected: 0, got: {result}");
    }

    #[test]
    fn f32_to_int24_aligned_high_low_byte_is_always_zero() {
        for sample in [-1.0_f32, -0.5, -0.1, 0.0, 0.1, 0.5, 1.0] {
            let result = f32_to_int24_aligned_high(sample);
            assert_eq!(
                result & 0xFF,
                0,
                "Low byte should be zero for sample {sample}, got: {result:#010X}"
            );
        }
    }

    #[test]
    fn f32_to_i16_converts_positive_full_scale() {
        let expected = 32767_i16;
        let result = f32_to_i16(1.0);
        assert_eq!(result, expected, "Expected: {expected}, got: {result}");
    }

    #[test]
    fn f32_to_i16_converts_negative_full_scale() {
        let expected = -32767_i16;
        let result = f32_to_i16(-1.0);
        assert_eq!(result, expected, "Expected: {expected}, got: {result}");
    }

    #[test]
    fn f32_to_i16_converts_zero() {
        assert_eq!(f32_to_i16(0.0), 0_i16);
    }

    #[test]
    fn f32_to_i16_clamps_above_positive_full_scale() {
        assert_eq!(f32_to_i16(2.0), 32767_i16);
    }

    #[test]
    fn f32_to_i16_clamps_below_negative_full_scale() {
        assert_eq!(f32_to_i16(-2.0), -32767_i16);
    }

    #[test]
    fn f32_to_i8_converts_positive_full_scale() {
        assert_eq!(f32_to_i8(1.0), 127_i8);
    }

    #[test]
    fn f32_to_i8_converts_negative_full_scale() {
        assert_eq!(f32_to_i8(-1.0), -127_i8);
    }

    #[test]
    fn f32_to_i8_converts_zero() {
        assert_eq!(f32_to_i8(0.0), 0_i8);
    }

    #[test]
    fn f32_to_i8_clamps_above_positive_full_scale() {
        assert_eq!(f32_to_i8(2.0), 127_i8);
    }

    #[test]
    fn f32_to_i8_clamps_below_negative_full_scale() {
        assert_eq!(f32_to_i8(-2.0), -127_i8);
    }
}
