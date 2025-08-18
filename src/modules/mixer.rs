#![allow(dead_code)]

const QUAD_MIX_DEFAULT_LEVEL_SUM: f32 = 4.0;
const QUAD_MIX_DEFAULT_MUTE: bool = false;
const QUAD_MIX_DEFAULT_CONSTANT_IS_ENABLED: bool = false;
const DEFAULT_LEVEL: f32 = 1.0;
const DEFAULT_PAN: f32 = 0.0;

pub enum MixerInput {
    One,
    Two,
    Three,
    Four,
}

#[derive(Copy, Clone, Debug, PartialEq)]
struct Input {
    level: f32,
    pan: f32,
    mute: bool,
}

impl Default for Input {
    fn default() -> Self {
        Self {
            level: DEFAULT_LEVEL,
            pan: DEFAULT_PAN,
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
    pan: f32,
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
                level: DEFAULT_LEVEL,
                pan: DEFAULT_PAN,
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

        let (input1_left, input1_right) = apply_quad_pan(leveled_input1, self.quad_mix.input1.pan);
        let (input2_left, input2_right) = apply_quad_pan(leveled_input2, self.quad_mix.input2.pan);
        let (input3_left, input3_right) = apply_quad_pan(leveled_input3, self.quad_mix.input3.pan);
        let (input4_left, input4_right) = apply_quad_pan(leveled_input4, self.quad_mix.input4.pan);

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
        apply_pan(leveled_left_input, leveled_right_input, self.output_mix.pan)
    }

    pub fn set_quad_level(&mut self, level: f32, input: MixerInput) {
        match input {
            MixerInput::One => self.quad_mix.input1.level = level,
            MixerInput::Two => self.quad_mix.input2.level = level,
            MixerInput::Three => self.quad_mix.input3.level = level,
            MixerInput::Four => self.quad_mix.input4.level = level,
        }
        self.sum_quad_input_levels();
    }

    pub fn set_quad_pan(&mut self, pan: f32, input: MixerInput) {
        match input {
            MixerInput::One => self.quad_mix.input1.pan = pan,
            MixerInput::Two => self.quad_mix.input2.pan = pan,
            MixerInput::Three => self.quad_mix.input3.pan = pan,
            MixerInput::Four => self.quad_mix.input4.pan = pan,
        }
    }

    pub fn set_quad_mute(&mut self, mute: bool, input: MixerInput) {
        match input {
            MixerInput::One => self.quad_mix.input1.mute = mute,
            MixerInput::Two => self.quad_mix.input2.mute = mute,
            MixerInput::Three => self.quad_mix.input3.mute = mute,
            MixerInput::Four => self.quad_mix.input4.mute = mute,
        }
    }

    pub fn set_constant_level(&mut self, is_enabled: bool) {
        self.quad_mix.is_constant_level = is_enabled;
    }

    pub fn set_output_level(&mut self, level: f32) {
        self.output_mix.level = level;
    }

    pub fn set_output_pan(&mut self, pan: f32) {
        self.output_mix.pan = pan;
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

fn apply_quad_pan(input: f32, pan: f32) -> (f32, f32) {
    apply_pan(input, input, pan)
}

fn apply_pan(mut input_left: f32, mut input_right: f32, pan: f32) -> (f32, f32) {
    if pan == 0.0 {
        return (input_left, input_right);
    }

    if pan.is_sign_positive() {
        input_left *= 1.0 - pan;
        (input_left, input_right)
    } else {
        input_right *= 1.0 - pan.abs();
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
    fn quad_mix_returns_correct_values_from_four_inputs_with_pan() {
        let mut mixer = Mixer::new();
        mixer.set_quad_pan(0.5, MixerInput::One);
        mixer.set_quad_pan(0.5, MixerInput::Two);
        mixer.set_quad_pan(0.5, MixerInput::Three);
        mixer.set_quad_pan(0.5, MixerInput::Four);
        let expected_result_left = 1.25;
        let expected_result_right = 2.5;

        let (left_input, right_input) = mixer.quad_mix(1.0, 2.0, 3.0, 4.0);
        assert!(f32_value_equality(left_input, expected_result_left));
        assert!(f32_value_equality(right_input, expected_result_right));
    }

    #[test]
    fn quad_mix_returns_correct_values_from_four_inputs_with_negative_pan() {
        let mut mixer = Mixer::new();
        mixer.set_quad_pan(-0.5, MixerInput::One);
        mixer.set_quad_pan(-0.5, MixerInput::Two);
        mixer.set_quad_pan(-0.5, MixerInput::Three);
        mixer.set_quad_pan(-0.5, MixerInput::Four);

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
        mixer.set_quad_level(0.5, MixerInput::One);
        mixer.set_quad_level(0.5, MixerInput::Two);
        mixer.set_quad_level(0.5, MixerInput::Three);
        mixer.set_quad_level(0.5, MixerInput::Four);
        let expected_result_left = 1.25;
        let expected_result_right = 1.25;

        let (left_input, right_input) = mixer.quad_mix(1.0, 2.0, 3.0, 4.0);
        assert!(f32_value_equality(left_input, expected_result_left));
        assert!(f32_value_equality(right_input, expected_result_right));
    }

    #[test]
    fn quad_mix_returns_correct_values_from_four_inputs_with_level_and_pan() {
        let mut mixer = Mixer::new();
        mixer.set_quad_level(0.5, MixerInput::One);
        mixer.set_quad_level(0.5, MixerInput::Two);
        mixer.set_quad_level(0.5, MixerInput::Three);
        mixer.set_quad_level(0.5, MixerInput::Four);
        mixer.set_quad_pan(0.8, MixerInput::One);
        mixer.set_quad_pan(0.8, MixerInput::Two);
        mixer.set_quad_pan(0.8, MixerInput::Three);
        mixer.set_quad_pan(0.8, MixerInput::Four);
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

        mixer.set_quad_mute(true, MixerInput::One);
        mixer.set_quad_mute(true, MixerInput::Two);
        mixer.set_quad_mute(true, MixerInput::Three);
        mixer.set_quad_mute(true, MixerInput::Four);
        let muted_expected_result_left = 0.0;
        let muted_expected_result_right = 0.0;

        let (left_input, right_input) = mixer.quad_mix(1.0, 2.0, 3.0, 4.0);
        assert!(f32_value_equality(left_input, muted_expected_result_left));
        assert!(f32_value_equality(right_input, muted_expected_result_right));
    }

    #[test]
    fn quad_mix_returns_correct_values_from_four_one_input_and_constant_level() {
        let mut mixer = Mixer::new();
        mixer.set_quad_level(1.0, MixerInput::One);
        mixer.set_quad_level(0.0, MixerInput::Two);
        mixer.set_quad_level(0.0, MixerInput::Three);
        mixer.set_quad_level(0.0, MixerInput::Four);

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
        let expected_result_left = 1.0;
        let expected_result_right = 2.0;

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
    fn output_mix_returns_correct_values_with_pan() {
        let mut mixer = Mixer::new();
        let left_input = 1.0;
        let right_input = 2.0;
        let expected_result_left = 0.5;
        let expected_result_right = 2.0;

        mixer.set_output_pan(0.5);
        let (left_result, right_result) = mixer.output_mix(left_input, right_input);
        assert!(f32_value_equality(left_result, expected_result_left));
        assert!(f32_value_equality(right_result, expected_result_right));
    }

    #[test]
    fn output_mix_returns_correct_values_with_negative_pan() {
        let mut mixer = Mixer::new();
        let left_input = 1.0;
        let right_input = 1.0;
        let expected_result_left = 1.0;
        let expected_result_right = 0.5;

        mixer.set_output_pan(-0.5);

        let (left_result, right_result) = mixer.output_mix(left_input, right_input);
        println!("{:?}, {:?}", left_result, right_result);
        assert!(f32_value_equality(left_result, expected_result_left));
        assert!(f32_value_equality(right_result, expected_result_right));
    }

    #[test]
    fn output_mix_returns_correct_values_with_level_and_pan() {
        let mut mixer = Mixer::new();
        let left_input = 1.0;
        let right_input = 2.0;
        let expected_result_left = 0.25;
        let expected_result_right = 1.0;

        mixer.set_output_pan(0.5);
        mixer.set_output_level(0.5);
        let (left_result, right_result) = mixer.output_mix(left_input, right_input);
        assert!(f32_value_equality(left_result, expected_result_left));
        assert!(f32_value_equality(right_result, expected_result_right));
    }
}
