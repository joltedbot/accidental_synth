const NUMBER_OF_INPUTS: u8 = 4;
const DEFAULT_LEVEL: f32 = 1.0;
const DEFAULT_PAN: f32 = 0.0;
const DEFAULT_CONSTANT_IS_ENABLED: bool = false;

pub enum MixerInput {
    One,
    Two,
    Three,
    Four,
}

#[derive(Default, Copy, Clone, Debug, PartialEq)]
pub struct Mixer {
    input_1_level: f32,
    input_2_level: f32,
    input_3_level: f32,
    input_4_level: f32,
    input_1_pan: f32,
    input_2_pan: f32,
    input_3_pan: f32,
    input_4_pan: f32,
    is_constant_level: bool,
}

impl Mixer {
    pub fn new() -> Self {
        log::info!("Constructing Oscillator Module");
        Self {
            is_constant_level: DEFAULT_CONSTANT_IS_ENABLED,
            input_1_level: DEFAULT_LEVEL,
            input_2_level: DEFAULT_LEVEL,
            input_3_level: DEFAULT_LEVEL,
            input_4_level: DEFAULT_LEVEL,
            input_1_pan: DEFAULT_PAN,
            input_2_pan: DEFAULT_PAN,
            input_3_pan: DEFAULT_PAN,
            input_4_pan: DEFAULT_PAN,
        }
    }

    pub fn mix(&self, input1: f32, input2: f32, input3: f32, input4: f32) -> (f32, f32) {
        let (input1_left, input1_right) = self.apply_pan(input1, self.input_1_pan);
        let (input2_left, input2_right) = self.apply_pan(input2, self.input_2_pan);
        let (input3_left, input3_right) = self.apply_pan(input3, self.input_3_pan);
        let (input4_left, input4_right) = self.apply_pan(input4, self.input_4_pan);

        let mut left_input_sum = input1_left + input2_left + input3_left + input4_left;
        let mut right_input_sum = input1_right + input2_right + input3_right + input4_right;

        if self.is_constant_level {
            left_input_sum /=
                (self.input_1_level + self.input_2_level + self.input_3_level + self.input_4_level);
            right_input_sum /=
                (self.input_1_level + self.input_2_level + self.input_3_level + self.input_4_level);
        } else {
            left_input_sum /= NUMBER_OF_INPUTS as f32;
            right_input_sum /= NUMBER_OF_INPUTS as f32;
        }

        (left_input_sum, right_input_sum)
    }

    pub fn set_level(&mut self, level: f32, input: MixerInput) {
        match input {
            MixerInput::One => self.input_1_level = level,
            MixerInput::Two => self.input_2_level = level,
            MixerInput::Three => self.input_3_level = level,
            MixerInput::Four => self.input_4_level = level,
        }
    }

    pub fn set_pan(&mut self, pan: f32, input: MixerInput) {
        match input {
            MixerInput::One => self.input_1_pan = pan,
            MixerInput::Two => self.input_2_pan = pan,
            MixerInput::Three => self.input_3_pan = pan,
            MixerInput::Four => self.input_4_pan = pan,
        }
    }

    pub fn constant_level(&mut self, is_enabled: bool) {
        self.is_constant_level = is_enabled;
    }

    fn apply_pan(&self, input: f32, pan: f32) -> (f32, f32) {
        if pan == 0.0 {
            return (input, input);
        }

        let mut left = input;
        let mut right = input;

        if pan.is_sign_positive() {
            right *= pan;
            (left, right)
        } else {
            left *= pan;
            (left, right)
        }
    }
}
