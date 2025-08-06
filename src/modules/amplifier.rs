const DEFAULT_AMPLIFIER_CY_VALUE: f32 = 0.0;
const DEFAULT_AMPLIFIER_MANUAL_VALUE: f32 = 1.0;

pub fn controllable_amplifier(
    sample: f32,
    control_value: Option<f32>,
    manual_value: Option<f32>,
) -> f32 {
    sample
        * control_value.unwrap_or(DEFAULT_AMPLIFIER_CY_VALUE)
        * manual_value.unwrap_or(DEFAULT_AMPLIFIER_MANUAL_VALUE)
}
