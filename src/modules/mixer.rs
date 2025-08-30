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
