#![allow(dead_code)]

const QUAD_MIX_DEFAULT_LEVEL_SUM: f32 = 4.0;
const QUAD_MIX_DEFAULT_MUTE: bool = false;
const QUAD_MIX_DEFAULT_CONSTANT_IS_ENABLED: bool = false;
const QUAD_MIX_DEFAULT_INPUT_LEVEL: f32 = 1.0;
const DEFAULT_OUTPUT_LEVEL: f32 = 0.5;
const DEFAULT_BALANCE: f32 = 0.0;

pub enum MixerInput<T> {
    One(T),
    Two(T),
    Three(T),
    Four(T),
}

#[derive(Copy, Clone, Debug, PartialEq)]
struct Input {
    level: f32,
    balance: f32,
    mute: bool,
}

impl Default for Input {
    fn default() -> Self {
        Self {
            level: QUAD_MIX_DEFAULT_INPUT_LEVEL,
            balance: DEFAULT_BALANCE,
            mute: QUAD_MIX_DEFAULT_MUTE,
        }
    }
}

#[derive(Default, Copy, Clone, Debug, PartialEq)]
pub struct QuadMix {
    level_sum: f32,
    is_constant_level: bool,
    input1: Input,
    input2: Input,
    input3: Input,
    input4: Input,
}

#[derive(Default, Copy, Clone, Debug, PartialEq)]
pub struct OutputMix {
    level: f32,
    balance: f32,
}

#[derive(Default, Clone, Debug, PartialEq)]
pub struct Mixer {
    quad_mix: QuadMix,
    output_mix: OutputMix,
}

impl Mixer {
    pub fn new() -> Self {
        log::info!("Constructing Oscillator Module");
        Self {
            quad_mix: QuadMix {
                level_sum: QUAD_MIX_DEFAULT_LEVEL_SUM,
                is_constant_level: QUAD_MIX_DEFAULT_CONSTANT_IS_ENABLED,
                ..Default::default()
            },
            output_mix: OutputMix {
                level: DEFAULT_OUTPUT_LEVEL,
                balance: DEFAULT_BALANCE,
            },
        }
    }

    pub fn quad_mix(&self, input1: f32, input2: f32, input3: f32, input4: f32) -> (f32, f32) {
        let leveled_input1 = apply_quad_level(
            input1,
            self.quad_mix.input1.level,
            self.quad_mix.input1.mute,
        );
        let leveled_input2 = apply_quad_level(
            input2,
            self.quad_mix.input2.level,
            self.quad_mix.input2.mute,
        );
        let leveled_input3 = apply_quad_level(
            input3,
            self.quad_mix.input3.level,
            self.quad_mix.input3.mute,
        );
        let leveled_input4 = apply_quad_level(
            input4,
            self.quad_mix.input4.level,
            self.quad_mix.input4.mute,
        );

        let (input1_left, input1_right) =
            apply_quad_balance(leveled_input1, self.quad_mix.input1.balance);
        let (input2_left, input2_right) =
            apply_quad_balance(leveled_input2, self.quad_mix.input2.balance);
        let (input3_left, input3_right) =
            apply_quad_balance(leveled_input3, self.quad_mix.input3.balance);
        let (input4_left, input4_right) =
            apply_quad_balance(leveled_input4, self.quad_mix.input4.balance);

        let mut left_input_sum = input1_left + input2_left + input3_left + input4_left;
        let mut right_input_sum = input1_right + input2_right + input3_right + input4_right;

        if self.quad_mix.is_constant_level {
            left_input_sum /= self.quad_mix.level_sum;
            right_input_sum /= self.quad_mix.level_sum;
        } else {
            left_input_sum /= QUAD_MIX_DEFAULT_LEVEL_SUM;
            right_input_sum /= QUAD_MIX_DEFAULT_LEVEL_SUM;
        }

        (left_input_sum, right_input_sum)
    }

    pub fn output_mix(&self, left_input: f32, right_input: f32) -> (f32, f32) {
        let leveled_left_input = left_input * self.output_mix.level;
        let leveled_right_input = right_input * self.output_mix.level;
        apply_balance(
            leveled_left_input,
            leveled_right_input,
            self.output_mix.balance,
        )
    }

    pub fn set_quad_level(&mut self, input: MixerInput<f32>) {
        match input {
            MixerInput::One(level) => self.quad_mix.input1.level = level,
            MixerInput::Two(level) => self.quad_mix.input2.level = level,
            MixerInput::Three(level) => self.quad_mix.input3.level = level,
            MixerInput::Four(level) => self.quad_mix.input4.level = level,
        }
        self.sum_quad_input_levels();
    }

    pub fn set_quad_balance(&mut self, input: MixerInput<f32>) {
        match input {
            MixerInput::One(balance) => self.quad_mix.input1.balance = balance,
            MixerInput::Two(balance) => self.quad_mix.input2.balance = balance,
            MixerInput::Three(balance) => self.quad_mix.input3.balance = balance,
            MixerInput::Four(balance) => self.quad_mix.input4.balance = balance,
        }
    }

    pub fn set_quad_mute(&mut self, input: MixerInput<bool>) {
        match input {
            MixerInput::One(mute) => self.quad_mix.input1.mute = mute,
            MixerInput::Two(mute) => self.quad_mix.input2.mute = mute,
            MixerInput::Three(mute) => self.quad_mix.input3.mute = mute,
            MixerInput::Four(mute) => self.quad_mix.input4.mute = mute,
        }
    }

    pub fn set_constant_level(&mut self, is_enabled: bool) {
        self.quad_mix.is_constant_level = is_enabled;
    }

    pub fn set_output_level(&mut self, level: f32) {
        self.output_mix.level = level;
    }

    pub fn set_output_balance(&mut self, balance: f32) {
        self.output_mix.balance = balance;
    }

    fn sum_quad_input_levels(&mut self) {
        self.quad_mix.level_sum = self.quad_mix.input1.level
            + self.quad_mix.input2.level
            + self.quad_mix.input3.level
            + self.quad_mix.input4.level
    }
}

fn apply_quad_level(input: f32, level: f32, mute: bool) -> f32 {
    if mute {
        return 0.0;
    }

    input * level
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

    fn f32_value_equality(value_1: f32, value_2: f32) -> bool {
        (value_1 - value_2).abs() <= f32::EPSILON
    }

    #[test]
    fn new_returns_mixer_with_correct_default_values() {
        let mixer = Mixer::new();
        assert_eq!(mixer.quad_mix.input1, Input::default());
        assert_eq!(mixer.quad_mix.input2, Input::default());
        assert_eq!(mixer.quad_mix.input3, Input::default());
        assert_eq!(mixer.quad_mix.input4, Input::default());
        assert_eq!(mixer.quad_mix.input1, Input::default());
        assert_eq!(mixer.quad_mix.input2, Input::default());
        assert_eq!(mixer.quad_mix.input3, Input::default());
        assert_eq!(mixer.quad_mix.input4, Input::default());
        assert_eq!(
            mixer.quad_mix.is_constant_level,
            QUAD_MIX_DEFAULT_CONSTANT_IS_ENABLED
        );
    }

    #[test]
    fn quad_mix_returns_correct_values_from_four_inputs() {
        let mixer = Mixer::new();
        let expected_result = 2.5;
        let (left_input, right_input) = mixer.quad_mix(1.0, 2.0, 3.0, 4.0);
        assert!(f32_value_equality(left_input, expected_result));
        assert!(f32_value_equality(right_input, expected_result));
    }

    #[test]
    fn quad_mix_returns_correct_values_from_four_inputs_with_balance() {
        let mut mixer = Mixer::new();
        mixer.set_quad_balance(MixerInput::One(0.5));
        mixer.set_quad_balance(MixerInput::Two(0.5));
        mixer.set_quad_balance(MixerInput::Three(0.5));
        mixer.set_quad_balance(MixerInput::Four(0.5));
        let expected_result_left = 1.25;
        let expected_result_right = 2.5;

        let (left_input, right_input) = mixer.quad_mix(1.0, 2.0, 3.0, 4.0);
        assert!(f32_value_equality(left_input, expected_result_left));
        assert!(f32_value_equality(right_input, expected_result_right));
    }

    #[test]
    fn quad_mix_returns_correct_values_from_four_inputs_with_negative_balance() {
        let mut mixer = Mixer::new();
        mixer.set_quad_balance(MixerInput::One(-0.5));
        mixer.set_quad_balance(MixerInput::Two(-0.5));
        mixer.set_quad_balance(MixerInput::Three(-0.5));
        mixer.set_quad_balance(MixerInput::Four(-0.5));

        let expected_result_left = 2.5;
        let expected_result_right = 1.25;

        let (left_input, right_input) = mixer.quad_mix(1.0, 2.0, 3.0, 4.0);
        println!("{:?}, {:?}", left_input, right_input);
        assert!(f32_value_equality(left_input, expected_result_left));
        assert!(f32_value_equality(right_input, expected_result_right));
    }

    #[test]
    fn quad_mix_returns_correct_values_from_four_inputs_with_level() {
        let mut mixer = Mixer::new();
        mixer.set_quad_level(MixerInput::One(0.5));
        mixer.set_quad_level(MixerInput::Two(0.5));
        mixer.set_quad_level(MixerInput::Three(0.5));
        mixer.set_quad_level(MixerInput::Four(0.5));
        let expected_result_left = 1.25;
        let expected_result_right = 1.25;

        let (left_input, right_input) = mixer.quad_mix(1.0, 2.0, 3.0, 4.0);
        assert!(f32_value_equality(left_input, expected_result_left));
        assert!(f32_value_equality(right_input, expected_result_right));
    }

    #[test]
    fn quad_mix_returns_correct_values_from_four_inputs_with_level_and_balance() {
        let mut mixer = Mixer::new();
        mixer.set_quad_level(MixerInput::One(0.5));
        mixer.set_quad_level(MixerInput::Two(0.5));
        mixer.set_quad_level(MixerInput::Three(0.5));
        mixer.set_quad_level(MixerInput::Four(0.5));
        mixer.set_quad_balance(MixerInput::One(0.8));
        mixer.set_quad_balance(MixerInput::Two(0.8));
        mixer.set_quad_balance(MixerInput::Three(0.8));
        mixer.set_quad_balance(MixerInput::Four(0.8));
        let expected_result_left = 0.2499999;
        let expected_result_right = 1.25;

        let (left_input, right_input) = mixer.quad_mix(1.0, 2.0, 3.0, 4.0);
        assert!(f32_value_equality(left_input, expected_result_left));
        assert!(f32_value_equality(right_input, expected_result_right));
    }

    #[test]
    fn quad_mix_returns_correct_values_from_four_inputs_with_mutes() {
        let mut mixer = Mixer::new();
        let default_expected_result_left = 2.5;
        let default_expected_result_right = 2.5;

        let (left_input, right_input) = mixer.quad_mix(1.0, 2.0, 3.0, 4.0);
        assert!(f32_value_equality(left_input, default_expected_result_left));
        assert!(f32_value_equality(
            right_input,
            default_expected_result_right
        ));

        mixer.set_quad_mute(MixerInput::One(true));
        mixer.set_quad_mute(MixerInput::Two(true));
        mixer.set_quad_mute(MixerInput::Three(true));
        mixer.set_quad_mute(MixerInput::Four(true));
        let muted_expected_result_left = 0.0;
        let muted_expected_result_right = 0.0;

        let (left_input, right_input) = mixer.quad_mix(1.0, 2.0, 3.0, 4.0);
        assert!(f32_value_equality(left_input, muted_expected_result_left));
        assert!(f32_value_equality(right_input, muted_expected_result_right));
    }

    #[test]
    fn quad_mix_returns_correct_values_from_four_one_input_and_constant_level() {
        let mut mixer = Mixer::new();
        mixer.set_quad_level(MixerInput::One(1.0));
        mixer.set_quad_level(MixerInput::Two(0.0));
        mixer.set_quad_level(MixerInput::Three(0.0));
        mixer.set_quad_level(MixerInput::Four(0.0));

        let default_expected_result_left = 0.25;
        let default_expected_result_right = 0.25;
        let (left_input, right_input) = mixer.quad_mix(1.0, 2.0, 3.0, 4.0);
        assert!(f32_value_equality(left_input, default_expected_result_left));
        assert!(f32_value_equality(
            right_input,
            default_expected_result_right
        ));

        mixer.set_constant_level(true);

        let muted_expected_result_left = 1.0;
        let muted_expected_result_right = 1.0;
        let (left_input, right_input) = mixer.quad_mix(1.0, 2.0, 3.0, 4.0);
        assert!(f32_value_equality(left_input, muted_expected_result_left));
        assert!(f32_value_equality(right_input, muted_expected_result_right));
    }

    #[test]
    fn output_mix_returns_correct_values_from_two_different_inputs() {
        let mixer = Mixer::new();
        let left_input = 1.0;
        let right_input = 2.0;
        let expected_result_left = 0.5;
        let expected_result_right = 1.0;

        let (left_result, right_result) = mixer.output_mix(left_input, right_input);
        assert!(f32_value_equality(left_result, expected_result_left));
        assert!(f32_value_equality(right_result, expected_result_right));
    }

    #[test]
    fn output_mix_returns_correct_values_with_level() {
        let mut mixer = Mixer::new();
        let left_input = 1.0;
        let right_input = 2.0;
        let expected_result_left = 0.5;
        let expected_result_right = 1.0;

        mixer.set_output_level(0.5);
        let (left_result, right_result) = mixer.output_mix(left_input, right_input);
        assert!(f32_value_equality(left_result, expected_result_left));
        assert!(f32_value_equality(right_result, expected_result_right));
    }

    #[test]
    fn output_mix_returns_correct_values_with_balance() {
        let mut mixer = Mixer::new();
        let left_input = 1.0;
        let right_input = 2.0;
        let expected_result_left = 0.25;
        let expected_result_right = 1.0;

        mixer.set_output_balance(0.5);
        let (left_result, right_result) = mixer.output_mix(left_input, right_input);
        assert!(f32_value_equality(left_result, expected_result_left));
        assert!(f32_value_equality(right_result, expected_result_right));
    }

    #[test]
    fn output_mix_returns_correct_values_with_negative_balance() {
        let mut mixer = Mixer::new();
        let left_input = 1.0;
        let right_input = 1.0;
        let expected_result_left = 0.5;
        let expected_result_right = 0.25;

        mixer.set_output_balance(-0.5);

        let (left_result, right_result) = mixer.output_mix(left_input, right_input);

        assert!(f32_value_equality(left_result, expected_result_left));
        assert!(f32_value_equality(right_result, expected_result_right));
    }

    #[test]
    fn output_mix_returns_correct_values_with_level_and_balance() {
        let mut mixer = Mixer::new();
        let left_input = 1.0;
        let right_input = 2.0;
        let expected_result_left = 0.25;
        let expected_result_right = 1.0;

        mixer.set_output_balance(0.5);
        mixer.set_output_level(0.5);
        let (left_result, right_result) = mixer.output_mix(left_input, right_input);
        assert!(f32_value_equality(left_result, expected_result_left));
        assert!(f32_value_equality(right_result, expected_result_right));
    }
}
