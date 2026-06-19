//! Safe casting helpers for bounded conversions used across all AccSyn crates.

/// Converts `f32` to `u8` by clamping to `[0.0, 255.0]` before truncating.
/// Use when the caller guarantees the value represents a small index or count.
#[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
#[must_use]
pub fn f32_to_u8_clamped(value: f32) -> u8 {
    value.clamp(0.0, f32::from(u8::MAX)) as u8
}

/// Converts `f32` to `u16` by clamping to `[0.0, f16::MAX]` before truncating.
/// Use when the caller guarantees the value represents a moderate sized index or count.
#[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
#[must_use]
pub fn f32_to_u16_clamped(value: f32) -> u16 {
    value.clamp(0.0, f32::from(u16::MAX)) as u16
}

/// Converts `f32` to `u32` by clamping to `[0.0, u32::MAX]` before truncating.
/// Use when the caller guarantees the value is a non-negative index or enum repr.
#[allow(
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss,
    clippy::cast_precision_loss
)]
#[must_use]
pub fn f32_to_u32_clamped(value: f32) -> u32 {
    // u32::MAX as f32 rounds up to 2^32 due to f32 precision; as-cast from f32 to u32
    // saturates at u32::MAX (Rust 1.45+ guaranteed), so the clamp is still correct.
    value.clamp(0.0, u32::MAX as f32) as u32
}

/// Converts `f32` to `usize` by clamping to `[0.0, usize::MAX]` before truncating.
/// Use for delay samples or other audio buffer offsets derived from f32 math.
#[allow(
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss,
    clippy::cast_precision_loss
)]
#[must_use]
pub fn f32_to_usize_clamped(value: f32) -> usize {
    // usize::MAX as f32 rounds up to 2^64 on 64-bit targets due to f32 precision; as-cast from f32
    // to usize saturates at usize::MAX (Rust 1.45+ guaranteed), so the clamp is still correct.
    value.clamp(0.0, usize::MAX as f32) as usize
}

/// Converts `f64` to `f32` by clamping to `[f32::MIN, f32::MAX]` before truncating.
/// Use when the caller guarantees the value can not exceed `f32::MAX`
#[allow(clippy::cast_possible_truncation)]
#[must_use]
pub fn f64_to_f32_clamped(value: f64) -> f32 {
    value.clamp(f64::from(f32::MIN), f64::from(f32::MAX)) as f32
}

/// Converts `f64` to `u32` by clamping to `[0.0, u32::MAX]` before truncating.
/// Use when the caller guarantees the value is a non-negative audio parameter (e.g., sample rate).
#[allow(
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss,
    clippy::cast_precision_loss
)]
#[must_use]
pub fn f64_to_u32_clamped(value: f64) -> u32 {
    // f64 can represent u32::MAX exactly (unlike f32), so the clamp bound is precise.
    value.clamp(0.0, f64::from(u32::MAX)) as u32
}

/// Converts `i32` to `u8` by clamping to `[0, 255]` before converting.
/// Use for wave shape indices and similar small UI-sourced integers.
#[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
#[must_use]
pub fn i32_to_u8_clamped(value: i32) -> u8 {
    value.clamp(0, i32::from(u8::MAX)) as u8
}
