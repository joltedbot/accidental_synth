const QUAD_MIX_DEFAULT_LEVEL_SUM: f32 = 4.0;

#[derive(Copy, Clone, Debug, PartialEq)]
pub struct MixerInput {
    pub sample: f32,
    pub level: f32,
    pub balance: f32,
    pub mute: bool,
}

pub fn quad_mix(
    input1: MixerInput,
    input2: MixerInput,
    input3: MixerInput,
    input4: MixerInput,
) -> (f32, f32) {
    let leveled_input1 = apply_quad_level(&input1);
    let leveled_input2 = apply_quad_level(&input2);
    let leveled_input3 = apply_quad_level(&input3);
    let leveled_input4 = apply_quad_level(&input4);

    let (input1_left, input1_right) = apply_quad_balance(leveled_input1, input1.balance);
    let (input2_left, input2_right) = apply_quad_balance(leveled_input2, input2.balance);
    let (input3_left, input3_right) = apply_quad_balance(leveled_input3, input3.balance);
    let (input4_left, input4_right) = apply_quad_balance(leveled_input4, input4.balance);

    let mut left_input_sum = input1_left + input2_left + input3_left + input4_left;
    let mut right_input_sum = input1_right + input2_right + input3_right + input4_right;

    left_input_sum /= QUAD_MIX_DEFAULT_LEVEL_SUM;
    right_input_sum /= QUAD_MIX_DEFAULT_LEVEL_SUM;

    (left_input_sum, right_input_sum)
}

pub fn output_mix(left_input: f32, right_input: f32, level: f32, balance: f32) -> (f32, f32) {
    let leveled_left_input = left_input * level;
    let leveled_right_input = right_input * level;
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
