use crate::modules::mixer::MixerInput::Four;

const QUAD_MIX_NUMBER_OF_INPUTS: u8 = 4;
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
pub struct Mixer {
    quad_mix: QuadMix,
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
            left_input_sum /= QUAD_MIX_NUMBER_OF_INPUTS as f32;
            right_input_sum /= QUAD_MIX_NUMBER_OF_INPUTS as f32;
        }

        (left_input_sum, right_input_sum)
    }

    pub fn output_mix(
        &self,
        left_input: f32,
        right_input: f32,
        output_level: f32,
        pan: f32,
    ) -> (f32, f32) {
        let leveled_left_input = left_input * output_level;
        let leveled_right_input = right_input * output_level;
        apply_pan(leveled_left_input, leveled_right_input, pan)
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
}
