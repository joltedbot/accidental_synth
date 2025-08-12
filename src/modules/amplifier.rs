const MAX_SAMPLE_VALUE: f32 = 1.0;
const MIN_SAMPLE_VALUE: f32 = -1.0;
const MAX_AMPLIFIER_VALUE: f32 = 1.0;
const MIN_AMPLIFIER_VALUE: f32 = 0.0;
const DEFAULT_AMPLIFIER_VALUE: f32 = 1.0;

pub fn mono_vca(sample: f32, manual_value: Option<f32>, control_value: Option<f32>) -> f32 {
    let manual = manual_value.unwrap_or(DEFAULT_AMPLIFIER_VALUE);
    let control = control_value.unwrap_or(DEFAULT_AMPLIFIER_VALUE);
    sample.clamp(MIN_SAMPLE_VALUE, MAX_SAMPLE_VALUE)
        * manual.clamp(MIN_AMPLIFIER_VALUE, MAX_AMPLIFIER_VALUE)
        * control.clamp(MIN_AMPLIFIER_VALUE, MAX_AMPLIFIER_VALUE)
}

pub fn stereo_vca(
    left_sample: f32,
    right_sample: f32,
    manual_value: Option<f32>,
    control_value: Option<f32>,
) -> (f32, f32) {
    let left_output_sample = mono_vca(left_sample, manual_value, control_value);
    let right_output_sample = mono_vca(right_sample, manual_value, control_value);
    (left_output_sample, right_output_sample)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn f32_value_equality(value_1: f32, value_2: f32) -> bool {
        (value_1 - value_2).abs() <= f32::EPSILON
    }

    #[test]
    fn vca_returns_correct_value_from_valid_input() {
        let sample = 0.9;
        let manual_value = Some(0.8);
        let control_value = Some(0.03);
        let expected_result = 0.0215999;
        let result = mono_vca(sample, manual_value, control_value);
        assert!(f32_value_equality(result, expected_result));
    }

    #[test]
    fn vca_returns_correct_value_from_valid_input_no_manual_value() {
        let sample = 0.9;
        let manual_value = None;
        let control_value = Some(0.03);
        let expected_result = 0.0269999;
        let result = mono_vca(sample, manual_value, control_value);
        assert!(f32_value_equality(result, expected_result));
    }

    #[test]
    fn vca_returns_correct_value_from_valid_input_no_control_value() {
        let sample = 0.9;
        let manual_value = Some(0.25);
        let control_value = None;
        let expected_result = 0.225;
        let result = mono_vca(sample, manual_value, control_value);
        assert!(f32_value_equality(result, expected_result));
    }

    #[test]
    fn vca_returns_correct_value_from_only_sample() {
        let sample = 0.9;
        let manual_value = None;
        let control_value = None;
        let expected_result = 0.9;
        let result = mono_vca(sample, manual_value, control_value);
        assert!(f32_value_equality(result, expected_result));
    }

    #[test]
    fn vca_returns_correct_value_from_zero_sample_no_values() {
        let sample = 0.0;
        let manual_value = None;
        let control_value = None;
        let expected_result = 0.0;
        let result = mono_vca(sample, manual_value, control_value);
        assert!(f32_value_equality(result, expected_result));
    }

    #[test]
    fn vca_returns_correct_value_from_max_values() {
        let sample = f32::MAX;
        let manual_value = Some(f32::MAX);
        let control_value = Some(f32::MAX);
        let expected_result = 1.0;
        let result = mono_vca(sample, manual_value, control_value);
        assert!(f32_value_equality(result, expected_result));
    }

    #[test]
    fn vca_returns_correct_value_from_min_sample_min_values() {
        let sample = f32::MIN;
        let manual_value = Some(f32::MIN);
        let control_value = Some(f32::MIN);
        let expected_result = -0.0;
        let result = mono_vca(sample, manual_value, control_value);
        assert!(f32_value_equality(result, expected_result));
    }

    #[test]
    fn stereo_vca_returns_identical_values_for_identical_left_right_input() {
        let left_sample = 0.9;
        let right_sample = 0.9;
        let manual_value = None;
        let control_value = None;
        let expected_result = (0.9, 0.9);
        let result = stereo_vca(left_sample, right_sample, manual_value, control_value);
        assert_eq!(result, expected_result);
    }

    #[test]
    fn stereo_vca_returns_different_values_for_different_left_right_input() {
        let left_sample = 0.9;
        let right_sample = 0.2;
        let manual_value = None;
        let control_value = None;
        let expected_result = (0.9, 0.2);
        let result = stereo_vca(left_sample, right_sample, manual_value, control_value);
        assert_eq!(result, expected_result);
    }
}
