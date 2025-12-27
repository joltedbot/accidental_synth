const QUAD_MIX_DEFAULT_LEVEL_SUM: f32 = 4.0;

#[derive(Copy, Clone, Debug, PartialEq)]
pub struct MixerInput {
    pub sample: f32,
    pub level: f32,
    pub balance: f32,
    pub mute: bool,
}

pub fn quad_mix(inputs: [MixerInput; 4]) -> (f32, f32) {
    let (mut left_input_sum, mut right_input_sum): (f32, f32) = inputs
        .iter()
        .map(|input| apply_quad_balance(apply_quad_level(input), input.balance))
        .fold(
            (0.0, 0.0),
            |(left_sum, right_sum), (left_input, right_input)| {
                (left_sum + left_input, right_sum + right_input)
            },
        );

    left_input_sum /= QUAD_MIX_DEFAULT_LEVEL_SUM;
    right_input_sum /= QUAD_MIX_DEFAULT_LEVEL_SUM;

    (left_input_sum, right_input_sum)
}

pub fn output_mix(
    stereo_input: (f32, f32),
    level: f32,
    balance: f32,
    is_muted: bool,
) -> (f32, f32) {
    if is_muted {
        return (0.0, 0.0);
    }

    let leveled_left_input = stereo_input.0 * level;
    let leveled_right_input = stereo_input.1 * level;
    apply_balance(leveled_left_input, leveled_right_input, balance)
}

fn apply_quad_level(input: &MixerInput) -> f32 {
    if input.mute {
        return 0.0;
    }

    input.sample * input.level
}

fn apply_quad_balance(input: f32, balance: f32) -> (f32, f32) {
    apply_balance(input, input, balance)
}

fn apply_balance(mut input_left: f32, mut input_right: f32, balance: f32) -> (f32, f32) {
    if balance == 0.0 {
        return (input_left, input_right);
    }

    if balance.is_sign_positive() {
        input_left *= 1.0 - balance;
        (input_left, input_right)
    } else {
        input_right *= 1.0 - balance.abs();
        (input_left, input_right)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::math::f32s_are_equal;

    fn assert_stereo_eq(actual: (f32, f32), expected: (f32, f32)) {
        assert!(
            f32s_are_equal(actual.0, expected.0) && f32s_are_equal(actual.1, expected.1),
            "Expected ({}, {}), got ({}, {})",
            expected.0,
            expected.1,
            actual.0,
            actual.1
        );
    }

    // Tests for apply_quad_level - only test branching logic (mute)
    #[test]
    fn test_apply_quad_level_muted() {
        let input = MixerInput {
            sample: 0.5,
            level: 0.8,
            balance: 0.0,
            mute: true,
        };

        let actual = apply_quad_level(&input);
        let expected = 0.0;

        assert!(
            f32s_are_equal(actual, expected),
            "Expected {expected}, got {actual}"
        );
    }

    // Tests for apply_balance - test branching and edge cases
    #[test]
    fn test_apply_balance_center() {
        let input_left = 0.5;
        let input_right = 0.5;
        let balance = 0.0;

        let actual = apply_balance(input_left, input_right, balance);
        let expected = (0.5, 0.5);

        assert_stereo_eq(actual, expected);
    }

    #[test]
    fn test_apply_balance_full_left() {
        let input_left = 0.5;
        let input_right = 0.5;
        let balance = -1.0;

        let actual = apply_balance(input_left, input_right, balance);
        let expected = (0.5, 0.0);

        assert_stereo_eq(actual, expected);
    }

    #[test]
    fn test_apply_balance_full_right() {
        let input_left = 0.5;
        let input_right = 0.5;
        let balance = 1.0;

        let actual = apply_balance(input_left, input_right, balance);
        let expected = (0.0, 0.5);

        assert_stereo_eq(actual, expected);
    }

    // Tests for quad_mix - test edge cases and integration of branching logic
    #[test]
    fn test_quad_mix_all_muted() {
        let inputs = [
            MixerInput {
                sample: 1.0,
                level: 1.0,
                balance: 0.0,
                mute: true,
            },
            MixerInput {
                sample: 1.0,
                level: 1.0,
                balance: 0.0,
                mute: true,
            },
            MixerInput {
                sample: 1.0,
                level: 1.0,
                balance: 0.0,
                mute: true,
            },
            MixerInput {
                sample: 1.0,
                level: 1.0,
                balance: 0.0,
                mute: true,
            },
        ];

        let actual = quad_mix(inputs);
        let expected = (0.0, 0.0);

        assert_stereo_eq(actual, expected);
    }

    #[test]
    fn test_quad_mix_mixed_panning() {
        let inputs = [
            MixerInput {
                sample: 1.0,
                level: 1.0,
                balance: -1.0, // Full left
                mute: false,
            },
            MixerInput {
                sample: 1.0,
                level: 1.0,
                balance: 1.0, // Full right
                mute: false,
            },
            MixerInput {
                sample: 1.0,
                level: 1.0,
                balance: 0.0, // Center
                mute: false,
            },
            MixerInput {
                sample: 1.0,
                level: 1.0,
                balance: 0.0, // Center
                mute: false,
            },
        ];

        let actual = quad_mix(inputs);
        let expected = (0.75, 0.75); // Left: (1+0+1+1)/4, Right: (0+1+1+1)/4

        assert_stereo_eq(actual, expected);
    }

    #[test]
    fn test_quad_mix_partial_mute() {
        let inputs = [
            MixerInput {
                sample: 1.0,
                level: 1.0,
                balance: 0.0,
                mute: false,
            },
            MixerInput {
                sample: 1.0,
                level: 1.0,
                balance: 0.0,
                mute: true,
            },
            MixerInput {
                sample: 1.0,
                level: 1.0,
                balance: 0.0,
                mute: false,
            },
            MixerInput {
                sample: 1.0,
                level: 1.0,
                balance: 0.0,
                mute: true,
            },
        ];

        let actual = quad_mix(inputs);
        let expected = (0.5, 0.5); // (1.0 + 0.0 + 1.0 + 0.0) / 4.0

        assert_stereo_eq(actual, expected);
    }

    // Tests for output_mix - test branching and integration
    #[test]
    fn test_output_mix_muted() {
        let stereo_input = (0.5, 0.4);
        let level = 1.0;
        let balance = 0.0;
        let is_muted = true;

        let actual = output_mix(stereo_input, level, balance, is_muted);
        let expected = (0.0, 0.0);

        assert_stereo_eq(actual, expected);
    }

    #[test]
    fn test_output_mix_level_and_balance() {
        let stereo_input = (1.0, 1.0);
        let level = 0.5;
        let balance = 0.5;
        let is_muted = false;

        let actual = output_mix(stereo_input, level, balance, is_muted);
        let expected = (0.25, 0.5); // Level: (0.5, 0.5), then balance right

        assert_stereo_eq(actual, expected);
    }
}
