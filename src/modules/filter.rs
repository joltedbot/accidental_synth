// Derived from https://www.musicdsp.org/en/latest/Filters/253-perfect-lp4-filter.html

use crate::math::load_f32_from_atomic_u32;
use std::sync::atomic::Ordering::Relaxed;
use std::sync::atomic::{AtomicU8, AtomicU32};

pub const NUMBER_OF_FILER_POLES: f32 = 4.0;
const MAX_FILTER_CUTOFF: f32 = 20000.0;
const DEFAULT_RESONANCE: f32 = 0.0;
const NATURAL_LOG_OF_4: f32 = 1.386_294_3;
const DENORMAL_GUARD: f32 = 1e-25_f32;
const MIDI_CENTER_NOTE_NUMBER: u8 = 64;
const NOTES_PER_OCTAVE: u8 = 12;
pub const DEFAULT_KEY_TRACKING_AMOUNT: f32 = 0.5;
pub const DEFAULT_KEY_TRACKING_FREQUENCY_OFFSET: f32 = 1.0;
const DEFAULT_FILTER_POLES: u8 = 4;
const MAX_FILTER_PERCENT_OF_NYQUIST: f32 = 0.35;

#[derive(Default, Debug)]
pub struct FilterParameters {
    pub cutoff_frequency: AtomicU32,
    pub resonance: AtomicU32,
    pub filter_poles: AtomicU8,
    pub key_tracking_amount: AtomicU32,
    pub current_note_number: AtomicU8,
}

#[derive(Default, Copy, Clone, Debug, PartialEq)]
struct Coefficients {
    cutoff_coefficient: f32,
    gain_coefficient: f32,
    pole_coefficient: f32,
    resonance_factor: f32,
    adjusted_resonance_factor: f32,
    feedback_gain: f32,
}

#[derive(Default, Copy, Clone, Debug, PartialEq)]
struct LadderState {
    stage1_output: f32,
    stage2_output: f32,
    stage3_output: f32,
    stage4_output: f32,
    input_unit_delay: f32,
    stage1_unit_delay: f32,
    stage2_unit_delay: f32,
    stage3_unit_delay: f32,
}

#[derive(Default, Debug)]
pub struct Filter {
    sample_rate: u32,
    max_frequency: f32,
    cutoff_frequency: f32,
    resonance: f32,
    poles: u8,
    key_tracking_amount: f32,
    key_tracking_frequency_offset: f32,
    cutoff_modulation_amount: f32,
    coefficients: Coefficients,
    left_ladder_state: LadderState,
    right_ladder_state: LadderState,
}

impl Filter {
    pub fn new(sample_rate: u32) -> Self {
        log::debug!("Constructing Filter Module");

        let max_frequency =
            (sample_rate as f32 * MAX_FILTER_PERCENT_OF_NYQUIST).min(MAX_FILTER_CUTOFF);

        let cutoff_coefficient = calculate_cutoff_coefficient(max_frequency, sample_rate);
        let gain_coefficient = calculate_gain_coefficient(cutoff_coefficient);
        let pole_coefficient = calculate_pole_coefficient(gain_coefficient);
        let resonance_factor = calculate_resonance_factor(gain_coefficient);
        let adjusted_resonance_factor = calculate_adjusted_resonance_factor(resonance_factor);
        let feedback_gain = calculate_feedback_gain(
            DEFAULT_RESONANCE,
            resonance_factor,
            adjusted_resonance_factor,
        );

        Self {
            sample_rate,
            cutoff_frequency: max_frequency,
            max_frequency,
            resonance: DEFAULT_RESONANCE,
            poles: DEFAULT_FILTER_POLES,
            key_tracking_amount: DEFAULT_KEY_TRACKING_AMOUNT,
            key_tracking_frequency_offset: DEFAULT_KEY_TRACKING_FREQUENCY_OFFSET,
            coefficients: Coefficients {
                cutoff_coefficient,
                gain_coefficient,
                pole_coefficient,
                resonance_factor,
                adjusted_resonance_factor,
                feedback_gain,
            },
            ..Default::default()
        }
    }

    pub fn set_parameters(&mut self, filter_parameters: &FilterParameters) {
        self.store_filter_parameters(filter_parameters);
        self.calculate_coefficients();
    }

    pub fn process(
        &mut self,
        left_sample: f32,
        right_sample: f32,
        modulation: Option<f32>,
    ) -> (f32, f32) {
        let modulation_amount = modulation.unwrap_or(0.0);
        self.modulate_cutoff_frequency(modulation_amount);
        self.cutoff_frequency = self.calculate_filter_cutoff_frequency();

        let left_output = self.left_ladder_filter(left_sample);
        let right_output = self.right_ladder_filter(right_sample);

        (left_output, right_output)
    }

    pub fn modulate_cutoff_frequency(&mut self, modulation: f32) {
        if modulation == 0.0 {
            return;
        }
        self.cutoff_modulation_amount = modulation * self.max_frequency;
    }

    fn calculate_coefficients(&mut self) {
        let cutoff_frequency = self.calculate_filter_cutoff_frequency();
        self.coefficients.cutoff_coefficient =
            calculate_cutoff_coefficient(cutoff_frequency, self.sample_rate);
        self.coefficients.gain_coefficient =
            calculate_gain_coefficient(self.coefficients.cutoff_coefficient);
        self.coefficients.pole_coefficient =
            calculate_pole_coefficient(self.coefficients.gain_coefficient);
        self.coefficients.resonance_factor =
            calculate_resonance_factor(self.coefficients.gain_coefficient);
        self.coefficients.adjusted_resonance_factor =
            calculate_adjusted_resonance_factor(self.coefficients.resonance_factor);
        self.coefficients.feedback_gain = calculate_feedback_gain(
            self.resonance,
            self.coefficients.resonance_factor,
            self.coefficients.adjusted_resonance_factor,
        );
    }

    fn left_ladder_filter(&mut self, sample: f32) -> f32 {
        let input = sample - self.coefficients.feedback_gain * self.left_ladder_state.stage4_output;

        self.left_ladder_state.stage1_output =
            self.calculate_ladder_stage1(input, self.left_ladder_state);
        self.left_ladder_state.stage2_output = self.calculate_ladder_stage2(self.left_ladder_state);
        self.left_ladder_state.stage3_output = self.calculate_ladder_stage3(self.left_ladder_state);
        self.left_ladder_state.stage4_output = self.calculate_ladder_stage4(self.left_ladder_state);

        self.left_ladder_state.stage4_output =
            calculate_non_linear_saturation(self.left_ladder_state.stage4_output);

        self.left_ladder_state.input_unit_delay = input;
        self.left_ladder_state.stage1_unit_delay =
            self.left_ladder_state.stage1_output + DENORMAL_GUARD;
        self.left_ladder_state.stage2_unit_delay =
            self.left_ladder_state.stage2_output + DENORMAL_GUARD;
        self.left_ladder_state.stage3_unit_delay =
            self.left_ladder_state.stage3_output + DENORMAL_GUARD;

        match self.poles {
            1 => self.left_ladder_state.stage1_output,
            2 => self.left_ladder_state.stage2_output,
            3 => self.left_ladder_state.stage3_output,
            _ => self.left_ladder_state.stage4_output,
        }
    }

    fn right_ladder_filter(&mut self, sample: f32) -> f32 {
        let input =
            sample - self.coefficients.feedback_gain * self.right_ladder_state.stage4_output;

        self.right_ladder_state.stage1_output =
            self.calculate_ladder_stage1(input, self.right_ladder_state);
        self.right_ladder_state.stage2_output =
            self.calculate_ladder_stage2(self.right_ladder_state);
        self.right_ladder_state.stage3_output =
            self.calculate_ladder_stage3(self.right_ladder_state);
        self.right_ladder_state.stage4_output =
            self.calculate_ladder_stage4(self.right_ladder_state);

        self.right_ladder_state.stage4_output =
            calculate_non_linear_saturation(self.right_ladder_state.stage4_output);

        self.right_ladder_state.input_unit_delay = input;
        self.right_ladder_state.stage1_unit_delay =
            self.right_ladder_state.stage1_output + DENORMAL_GUARD;
        self.right_ladder_state.stage2_unit_delay =
            self.right_ladder_state.stage2_output + DENORMAL_GUARD;
        self.right_ladder_state.stage3_unit_delay =
            self.right_ladder_state.stage3_output + DENORMAL_GUARD;

        match self.poles {
            1 => self.right_ladder_state.stage1_output,
            2 => self.right_ladder_state.stage2_output,
            3 => self.right_ladder_state.stage3_output,
            _ => self.right_ladder_state.stage4_output,
        }
    }

    fn store_filter_parameters(&mut self, parameters: &FilterParameters) {
        self.cutoff_frequency = load_f32_from_atomic_u32(&parameters.cutoff_frequency);
        self.resonance = load_f32_from_atomic_u32(&parameters.resonance);
        self.poles = parameters.filter_poles.load(Relaxed);
        self.key_tracking_amount = load_f32_from_atomic_u32(&parameters.key_tracking_amount);
        self.key_tracking_frequency_offset = get_tracking_offset_from_midi_note_number(
            parameters.current_note_number.load(Relaxed),
            self.key_tracking_amount,
        );
    }

    fn calculate_ladder_stage4(&mut self, ladder_state: LadderState) -> f32 {
        ladder_state.stage3_output * self.coefficients.gain_coefficient
            + ladder_state.stage3_unit_delay * self.coefficients.gain_coefficient
            - self.coefficients.pole_coefficient * ladder_state.stage4_output
    }

    fn calculate_ladder_stage3(&mut self, ladder_state: LadderState) -> f32 {
        ladder_state.stage2_output * self.coefficients.gain_coefficient
            + ladder_state.stage2_unit_delay * self.coefficients.gain_coefficient
            - self.coefficients.pole_coefficient * ladder_state.stage3_output
    }

    fn calculate_ladder_stage2(&mut self, ladder_state: LadderState) -> f32 {
        ladder_state.stage1_output * self.coefficients.gain_coefficient
            + ladder_state.stage1_unit_delay * self.coefficients.gain_coefficient
            - self.coefficients.pole_coefficient * ladder_state.stage2_output
    }

    fn calculate_ladder_stage1(&mut self, input: f32, ladder_state: LadderState) -> f32 {
        input * self.coefficients.gain_coefficient
            + ladder_state.input_unit_delay * self.coefficients.gain_coefficient
            - self.coefficients.pole_coefficient * ladder_state.stage1_output
    }

    fn calculate_filter_cutoff_frequency(&mut self) -> f32 {
        ((self.cutoff_frequency * self.key_tracking_frequency_offset)
            + self.cutoff_modulation_amount)
            .clamp(0.0, self.max_frequency)
    }
}

fn get_tracking_offset_from_midi_note_number(midi_note: u8, key_tracking_amount: f32) -> f32 {
    let delta_from_tracking_reference_note =
        f32::from(midi_note) - f32::from(MIDI_CENTER_NOTE_NUMBER);
    let key_tracking_bipolar = (key_tracking_amount - DEFAULT_KEY_TRACKING_AMOUNT) * 2.0;
    2.0_f32.powf(
        key_tracking_bipolar * delta_from_tracking_reference_note / f32::from(NOTES_PER_OCTAVE),
    )
}

fn calculate_non_linear_saturation(stage4_output: f32) -> f32 {
    stage4_output - (stage4_output * stage4_output * stage4_output / 6.0)
}

fn calculate_cutoff_coefficient(frequency: f32, sample_rate: u32) -> f32 {
    (frequency + frequency) / sample_rate as f32
}

fn calculate_pole_coefficient(gain_coefficient: f32) -> f32 {
    gain_coefficient + gain_coefficient - 1.0
}

fn calculate_gain_coefficient(cutoff_coefficient: f32) -> f32 {
    cutoff_coefficient * (1.8 - 0.8 * cutoff_coefficient)
}

fn calculate_resonance_factor(gain_coefficient: f32) -> f32 {
    (1.0 - gain_coefficient) * NATURAL_LOG_OF_4
}
fn calculate_adjusted_resonance_factor(resonance_factor: f32) -> f32 {
    12.0 + resonance_factor * resonance_factor
}

fn calculate_feedback_gain(
    resonance: f32,
    resonance_factor: f32,
    adjusted_resonance_factor: f32,
) -> f32 {
    resonance * (adjusted_resonance_factor + 6.0 * resonance_factor)
        / (adjusted_resonance_factor - 6.0 * resonance_factor)
}

pub fn max_frequency_from_sample_rate(sample_rate: u32) -> f32 {
    (sample_rate as f32 * MAX_FILTER_PERCENT_OF_NYQUIST).min(MAX_FILTER_CUTOFF)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::math::f32s_are_equal;

    #[test]
    fn new_returns_filter_with_correct_default_values() {
        let sample_rate = 48000;
        let max_frequency = sample_rate as f32 * 0.35;
        let filter = Filter::new(sample_rate);

        assert_eq!(filter.sample_rate, sample_rate);
        assert!(f32s_are_equal(filter.cutoff_frequency, max_frequency));
        assert!(f32s_are_equal(filter.resonance, DEFAULT_RESONANCE));
        assert_eq!(filter.poles, 4);
        assert!(f32s_are_equal(filter.left_ladder_state.stage1_output, 0.0));
        assert!(f32s_are_equal(filter.left_ladder_state.stage2_output, 0.0));
        assert!(f32s_are_equal(filter.left_ladder_state.stage3_output, 0.0));
        assert!(f32s_are_equal(filter.left_ladder_state.stage4_output, 0.0));
        assert!(f32s_are_equal(
            filter.left_ladder_state.input_unit_delay,
            0.0
        ));
        assert!(f32s_are_equal(
            filter.left_ladder_state.stage1_unit_delay,
            0.0
        ));
        assert!(f32s_are_equal(
            filter.left_ladder_state.stage2_unit_delay,
            0.0
        ));
        assert!(f32s_are_equal(
            filter.left_ladder_state.stage3_unit_delay,
            0.0
        ));
        assert!(f32s_are_equal(filter.right_ladder_state.stage1_output, 0.0));
        assert!(f32s_are_equal(filter.right_ladder_state.stage2_output, 0.0));
        assert!(f32s_are_equal(filter.right_ladder_state.stage3_output, 0.0));
        assert!(f32s_are_equal(filter.right_ladder_state.stage4_output, 0.0));
        assert!(f32s_are_equal(
            filter.right_ladder_state.input_unit_delay,
            0.0
        ));
        assert!(f32s_are_equal(
            filter.right_ladder_state.stage1_unit_delay,
            0.0
        ));
        assert!(f32s_are_equal(
            filter.right_ladder_state.stage2_unit_delay,
            0.0
        ));
        assert!(f32s_are_equal(
            filter.right_ladder_state.stage3_unit_delay,
            0.0
        ));

        let expected_cutoff = 0.7;
        let expected_gain = 0.868;
        let expected_pole = 0.735_999_9;
        let expected_res_factor = 0.182_990_88;
        let expected_adj_res_factor = 12.033_485;
        let expected_feedback = 0.0;

        assert!(f32s_are_equal(
            filter.coefficients.cutoff_coefficient,
            expected_cutoff
        ));
        assert!(f32s_are_equal(
            filter.coefficients.gain_coefficient,
            expected_gain
        ));
        assert!(f32s_are_equal(
            filter.coefficients.pole_coefficient,
            expected_pole
        ));
        assert!(f32s_are_equal(
            filter.coefficients.resonance_factor,
            expected_res_factor
        ));
        assert!(f32s_are_equal(
            filter.coefficients.adjusted_resonance_factor,
            expected_adj_res_factor
        ));
        assert!(f32s_are_equal(
            filter.coefficients.feedback_gain,
            expected_feedback
        ));
    }

    #[test]
    fn calculate_cutoff_coefficient_returns_expected_values() {
        let result = calculate_cutoff_coefficient(24000.0, 48000);
        assert!(f32s_are_equal(result, 1.0));

        let result_zero = calculate_cutoff_coefficient(0.0, 48000);
        assert!(f32s_are_equal(result_zero, 0.0));

        let expected_result = 0.833_333_3;
        let result_common = calculate_cutoff_coefficient(20000.0, 48000);
        assert!(f32s_are_equal(result_common, expected_result));
    }

    #[test]
    fn calculate_gain_coefficient_returns_expected_values() {
        let result = calculate_gain_coefficient(1.0);
        assert!(f32s_are_equal(result, 1.0));

        let result_zero = calculate_gain_coefficient(0.0);
        assert!(f32s_are_equal(result_zero, 0.0));

        let cutoff = 0.833_333_3;
        let expected_result = 0.944_444_4;
        let result_common = calculate_gain_coefficient(cutoff);
        assert!(f32s_are_equal(result_common, expected_result));
    }

    #[test]
    fn calculate_pole_coefficient_returns_expected_values() {
        assert!(f32s_are_equal(calculate_pole_coefficient(1.0), 1.0));
        assert!(f32s_are_equal(calculate_pole_coefficient(0.0), -1.0));

        let gain = 0.944_444_4;
        let expected_result = 0.888_888_7;
        assert!(f32s_are_equal(
            calculate_pole_coefficient(gain),
            expected_result
        ));
    }

    #[test]
    fn calculate_resonance_factor_returns_expected_values() {
        assert!(f32s_are_equal(calculate_resonance_factor(1.0), 0.0));

        assert!(f32s_are_equal(
            calculate_resonance_factor(0.0),
            NATURAL_LOG_OF_4
        ));

        let gain = 0.944_444_4;
        let expected_result = 0.077_016_39;
        assert!(f32s_are_equal(
            calculate_resonance_factor(gain),
            expected_result
        ));
    }

    #[test]
    fn calculate_adjusted_resonance_factor_returns_expected_values() {
        assert!(f32s_are_equal(
            calculate_adjusted_resonance_factor(0.0),
            12.0
        ));

        let rf = NATURAL_LOG_OF_4;
        let result = calculate_adjusted_resonance_factor(rf);
        let expected_result = 13.921_812;
        assert!(f32s_are_equal(result, expected_result));
    }

    #[test]
    fn calculate_feedback_gain_returns_expected_values() {
        let resonance = std::f32::consts::PI;
        assert!(f32s_are_equal(
            calculate_feedback_gain(0.0, resonance, resonance),
            0.0
        ));

        let result = calculate_feedback_gain(0.5, 0.0, 12.0);
        assert!(f32s_are_equal(result, 0.5));

        let resonance = 0.7;
        let rf = 0.05;
        let arf = 12.0025;
        let expected_result = 0.735_889_7;
        let result = calculate_feedback_gain(resonance, rf, arf);
        assert!(f32s_are_equal(result, expected_result));
    }

    #[test]
    fn calculate_ladder_stage1_returns_expected_value() {
        let mut filter = Filter::new(48000);
        filter.coefficients.gain_coefficient = 0.5;
        filter.coefficients.pole_coefficient = -0.2;
        let state = LadderState {
            input_unit_delay: 0.3,
            stage1_output: -0.4,
            ..LadderState::default()
        };

        let input = 0.8;
        let expected_result = 0.47;
        let result = filter.calculate_ladder_stage1(input, state);
        assert!(f32s_are_equal(result, expected_result));
    }

    #[test]
    fn calculate_ladder_stage2_returns_expected_value() {
        let mut filter = Filter::new(48000);
        filter.coefficients.gain_coefficient = 0.6;
        filter.coefficients.pole_coefficient = 0.1;
        let state = LadderState {
            stage1_output: -0.25,
            stage1_unit_delay: 0.15,
            stage2_output: 0.05,
            ..LadderState::default()
        };

        let expected_result = -0.065;
        let result = filter.calculate_ladder_stage2(state);
        assert!(f32s_are_equal(result, expected_result));
    }

    #[test]
    fn calculate_ladder_stage3_returns_expected_value() {
        let mut filter = Filter::new(48000);
        filter.coefficients.gain_coefficient = 0.7;
        filter.coefficients.pole_coefficient = -0.3;
        let state = LadderState {
            stage2_output: 0.2,
            stage2_unit_delay: -0.1,
            stage3_output: 0.05,
            ..LadderState::default()
        };

        let expected_result = 0.085;
        let result = filter.calculate_ladder_stage3(state);
        assert!(f32s_are_equal(result, expected_result));
    }

    #[test]
    fn calculate_ladder_stage4_returns_expected_value() {
        let mut filter = Filter::new(48000);
        filter.coefficients.gain_coefficient = 0.4;
        filter.coefficients.pole_coefficient = 0.25;
        let state = LadderState {
            stage3_output: -0.35,
            stage3_unit_delay: 0.22,
            stage4_output: -0.12,
            ..LadderState::default()
        };

        let expected_result = -0.022;
        let result = filter.calculate_ladder_stage4(state);
        assert!(f32s_are_equal(result, expected_result));
    }

    #[test]
    fn calculate_non_linear_saturation_returns_expected_values() {
        let expected_result = 0.0;
        let result = calculate_non_linear_saturation(0.0);
        assert!(f32s_are_equal(result, expected_result));

        let expected_result = 0.7785;
        let result = calculate_non_linear_saturation(0.9);
        assert!(f32s_are_equal(result, expected_result));

        let expected_result = -0.479_166_6;
        let result = calculate_non_linear_saturation(-0.5);
        assert!(f32s_are_equal(result, expected_result));
    }
}
