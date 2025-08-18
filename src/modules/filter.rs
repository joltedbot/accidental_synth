// Derived from https://www.musicdsp.org/en/latest/Filters/253-perfect-lp4-filter.html
#![allow(dead_code)]

const DEFAULT_CUTOFF_FREQUENCY: f32 = 20000.0;
const MAX_FILTER_FREQUENCY: f32 = 20000.0;
const MIN_FILTER_FREQUENCY: f32 = 0.0;
const MIN_RESONANCE: f32 = 0.0;
const MAX_RESONANCE: f32 = 1.0;

const NATURAL_LOG_OF_4: f32 = 1.3862943;
const DEFAULT_RESONANCE: f32 = 0.0;
const DENORMAL_GUARD: f32 = 1e-25_f32;

#[derive(Default, Copy, Clone, Debug, PartialEq)]
pub enum FilterSlope {
    Db6,  // 1-pole
    Db12, // 2-pole
    Db18, // 3-pole
    #[default]
    Db24, // 4-pole
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
pub struct Filter {
    sample_rate: u32,
    cutoff_frequency: f32,
    cutoff_modulation_amount: f32,
    resonance: f32,
    filter_slope: FilterSlope,
    coefficients: Coefficients,
    stage1_output: f32,
    stage2_output: f32,
    stage3_output: f32,
    stage4_output: f32,
    input_unit_delay: f32,
    stage1_unit_delay: f32,
    stage2_unit_delay: f32,
    stage3_unit_delay: f32,
}

impl Filter {
    pub fn new(sample_rate: u32) -> Self {
        log::info!("Constructing Filter Module");

        let cutoff_coefficient =
            calculate_cutoff_coefficient(DEFAULT_CUTOFF_FREQUENCY, sample_rate);
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
            cutoff_frequency: DEFAULT_CUTOFF_FREQUENCY,
            resonance: DEFAULT_RESONANCE,
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

    pub fn filter(&mut self, left_sample: f32, right_sample: f32) -> (f32, f32) {
        let left_output = self.ladder_filter(left_sample);
        let right_output = self.ladder_filter(right_sample);
        (left_output, right_output)
    }

    pub fn set_cutoff_frequency(&mut self, cut_off_frequency: f32) {
        self.cutoff_frequency = cut_off_frequency.clamp(MIN_FILTER_FREQUENCY, MAX_FILTER_FREQUENCY);
        let frequency = (self.cutoff_frequency + self.cutoff_modulation_amount)
            .clamp(MIN_FILTER_FREQUENCY, MAX_FILTER_FREQUENCY);
        self.calculate_coefficients_on_cutoff_update(frequency);
    }

    pub fn modulate_cutoff_frequency(&mut self, cutoff_modulation: f32) {
        self.cutoff_modulation_amount = cutoff_modulation * MAX_FILTER_FREQUENCY;
        let frequency = (self.cutoff_frequency + self.cutoff_modulation_amount)
            .clamp(MIN_FILTER_FREQUENCY, MAX_FILTER_FREQUENCY);
        self.calculate_coefficients_on_cutoff_update(frequency);
    }

    pub fn set_resonance(&mut self, resonance: f32) {
        self.resonance = resonance.clamp(MIN_RESONANCE, MAX_RESONANCE);
        self.coefficients.feedback_gain = calculate_feedback_gain(
            self.resonance,
            self.coefficients.resonance_factor,
            self.coefficients.adjusted_resonance_factor,
        );
    }

    pub fn set_filter_slope(&mut self, slope: FilterSlope) {
        self.filter_slope = slope;
    }

    fn calculate_coefficients_on_cutoff_update(&mut self, frequency: f32) {
        self.coefficients.cutoff_coefficient =
            calculate_cutoff_coefficient(frequency, self.sample_rate);
        self.coefficients.gain_coefficient =
            calculate_gain_coefficient(self.coefficients.cutoff_coefficient);
        self.coefficients.pole_coefficient =
            calculate_pole_coefficient(self.coefficients.gain_coefficient);
        self.coefficients.resonance_factor =
            calculate_resonance_factor(self.coefficients.gain_coefficient);
        self.coefficients.adjusted_resonance_factor =
            calculate_adjusted_resonance_factor(self.coefficients.resonance_factor);
    }

    fn ladder_filter(&mut self, sample: f32) -> f32 {
        let input = sample - self.coefficients.feedback_gain * self.stage4_output;

        self.stage1_output = self.calculate_ladder_stage1(input);
        self.stage2_output = self.calculate_ladder_stage2();
        self.stage3_output = self.calculate_ladder_stage3();
        self.stage4_output = self.calculate_ladder_stage4();

        self.stage4_output = self.calculate_non_linear_saturation();

        self.input_unit_delay = input;
        self.stage1_unit_delay = self.stage1_output + DENORMAL_GUARD;
        self.stage2_unit_delay = self.stage2_output + DENORMAL_GUARD;
        self.stage3_unit_delay = self.stage3_output + DENORMAL_GUARD;

        match self.filter_slope {
            FilterSlope::Db6 => self.stage1_output,
            FilterSlope::Db12 => self.stage2_output,
            FilterSlope::Db18 => self.stage3_output,
            FilterSlope::Db24 => self.stage4_output,
        }
    }

    fn calculate_non_linear_saturation(&mut self) -> f32 {
        self.stage4_output - ((self.stage4_output * self.stage4_output * self.stage4_output) / 6.0)
    }

    fn calculate_ladder_stage4(&mut self) -> f32 {
        self.stage3_output * self.coefficients.gain_coefficient
            + self.stage3_unit_delay * self.coefficients.gain_coefficient
            - self.coefficients.pole_coefficient * self.stage4_output
    }

    fn calculate_ladder_stage3(&mut self) -> f32 {
        self.stage2_output * self.coefficients.gain_coefficient
            + self.stage2_unit_delay * self.coefficients.gain_coefficient
            - self.coefficients.pole_coefficient * self.stage3_output
    }

    fn calculate_ladder_stage2(&mut self) -> f32 {
        self.stage1_output * self.coefficients.gain_coefficient
            + self.stage1_unit_delay * self.coefficients.gain_coefficient
            - self.coefficients.pole_coefficient * self.stage2_output
    }

    fn calculate_ladder_stage1(&mut self, input: f32) -> f32 {
        input * self.coefficients.gain_coefficient
            + self.input_unit_delay * self.coefficients.gain_coefficient
            - self.coefficients.pole_coefficient * self.stage1_output
    }
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

#[cfg(test)]
mod tests {
    use super::*;

    fn f32_value_equality(value_1: f32, value_2: f32) -> bool {
        (value_1 - value_2).abs() <= f32::EPSILON
    }

    #[test]
    fn new_returns_filter_with_correct_default_values() {
        let sample_rate = 48000;
        let filter = Filter::new(sample_rate);

        assert_eq!(filter.sample_rate, sample_rate);
        assert_eq!(filter.cutoff_frequency, DEFAULT_CUTOFF_FREQUENCY);
        assert_eq!(filter.resonance, DEFAULT_RESONANCE);
        assert_eq!(filter.filter_slope, FilterSlope::Db24);

        assert_eq!(filter.stage1_output, 0.0);
        assert_eq!(filter.stage2_output, 0.0);
        assert_eq!(filter.stage3_output, 0.0);
        assert_eq!(filter.stage4_output, 0.0);
        assert_eq!(filter.input_unit_delay, 0.0);
        assert_eq!(filter.stage1_unit_delay, 0.0);
        assert_eq!(filter.stage2_unit_delay, 0.0);
        assert_eq!(filter.stage3_unit_delay, 0.0);

        let expected_cutoff = 0.8333333;
        let expected_gain = 0.9444442;
        let expected_pole = 0.8888885;
        let expected_res_factor = 0.0770165;
        let expected_adj_res_factor = 12.005932;
        let expected_feedback = 0.0;

        assert!(f32_value_equality(
            filter.coefficients.cutoff_coefficient,
            expected_cutoff
        ));
        assert!(f32_value_equality(
            filter.coefficients.gain_coefficient,
            expected_gain
        ));
        assert!(f32_value_equality(
            filter.coefficients.pole_coefficient,
            expected_pole
        ));
        assert!(f32_value_equality(
            filter.coefficients.resonance_factor,
            expected_res_factor
        ));
        assert!(f32_value_equality(
            filter.coefficients.adjusted_resonance_factor,
            expected_adj_res_factor
        ));
        assert!(f32_value_equality(
            filter.coefficients.feedback_gain,
            expected_feedback
        ));
    }

    #[test]
    fn calculate_cutoff_coefficient_returns_expected_values() {
        let result = calculate_cutoff_coefficient(24000.0, 48000);
        assert_eq!(result, 1.0);

        let result_zero = calculate_cutoff_coefficient(0.0, 48000);
        assert_eq!(result_zero, 0.0);

        let expected_result = 0.8333333;
        let result_common = calculate_cutoff_coefficient(20000.0, 48000);
        assert!(f32_value_equality(result_common, expected_result));
    }

    #[test]
    fn calculate_gain_coefficient_returns_expected_values() {
        let result = calculate_gain_coefficient(1.0);
        assert!(f32_value_equality(result, 1.0));

        let result_zero = calculate_gain_coefficient(0.0);
        assert_eq!(result_zero, 0.0);

        let cutoff = 0.8333333;
        let expected_result = 0.9444444;
        let result_common = calculate_gain_coefficient(cutoff);
        assert!(f32_value_equality(result_common, expected_result));
    }

    #[test]
    fn calculate_pole_coefficient_returns_expected_values() {
        assert_eq!(calculate_pole_coefficient(1.0), 1.0);
        assert_eq!(calculate_pole_coefficient(0.0), -1.0);

        let gain = 0.9444444;
        let expected_result = 0.8888887;
        assert!(f32_value_equality(
            calculate_pole_coefficient(gain),
            expected_result
        ));
    }

    #[test]
    fn calculate_resonance_factor_returns_expected_values() {
        assert_eq!(calculate_resonance_factor(1.0), 0.0);

        assert!(f32_value_equality(
            calculate_resonance_factor(0.0),
            NATURAL_LOG_OF_4
        ));

        let gain = 0.9444444;
        let expected_result = 0.07701639;
        assert!(f32_value_equality(
            calculate_resonance_factor(gain),
            expected_result
        ));
    }

    #[test]
    fn calculate_adjusted_resonance_factor_returns_expected_values() {
        assert_eq!(calculate_adjusted_resonance_factor(0.0), 12.0);

        let rf = NATURAL_LOG_OF_4;
        let result = calculate_adjusted_resonance_factor(rf);
        let expected_result = 13.921812;
        assert!(f32_value_equality(result, expected_result));
    }

    #[test]
    fn calculate_feedback_gain_returns_expected_values() {
        let any = std::f32::consts::PI;
        assert_eq!(calculate_feedback_gain(0.0, any, any), 0.0);

        let result = calculate_feedback_gain(0.5, 0.0, 12.0);
        assert_eq!(result, 0.5);

        let resonance = 0.7;
        let rf = 0.05;
        let arf = 12.0025;
        let expected_result = 0.7358897;
        let result = calculate_feedback_gain(resonance, rf, arf);
        assert!(f32_value_equality(result, expected_result));
    }

    #[test]
    fn calculate_ladder_stage1_returns_expected_value() {
        let mut filter = Filter::new(48000);
        filter.coefficients.gain_coefficient = 0.5;
        filter.coefficients.pole_coefficient = -0.2;
        filter.input_unit_delay = 0.3;
        filter.stage1_output = -0.4;

        let input = 0.8;
        let expected_result = 0.47;
        let result = filter.calculate_ladder_stage1(input);
        assert!(f32_value_equality(result, expected_result));
    }

    #[test]
    fn calculate_ladder_stage2_returns_expected_value() {
        let mut filter = Filter::new(48000);
        filter.coefficients.gain_coefficient = 0.6;
        filter.coefficients.pole_coefficient = 0.1;
        filter.stage1_output = -0.25;
        filter.stage1_unit_delay = 0.15;
        filter.stage2_output = 0.05;

        let expected_result = -0.065;
        let result = filter.calculate_ladder_stage2();
        assert!(f32_value_equality(result, expected_result));
    }

    #[test]
    fn calculate_ladder_stage3_returns_expected_value() {
        let mut filter = Filter::new(48000);
        filter.coefficients.gain_coefficient = 0.7;
        filter.coefficients.pole_coefficient = -0.3;
        filter.stage2_output = 0.2;
        filter.stage2_unit_delay = -0.1;
        filter.stage3_output = 0.05;

        let expected_result = 0.085;
        let result = filter.calculate_ladder_stage3();
        assert!(f32_value_equality(result, expected_result));
    }

    #[test]
    fn calculate_ladder_stage4_returns_expected_value() {
        let mut filter = Filter::new(48000);
        filter.coefficients.gain_coefficient = 0.4;
        filter.coefficients.pole_coefficient = 0.25;
        filter.stage3_output = -0.35;
        filter.stage3_unit_delay = 0.22;
        filter.stage4_output = -0.12;

        let expected_result = -0.022;
        let result = filter.calculate_ladder_stage4();
        assert!(f32_value_equality(result, expected_result));
    }

    #[test]
    fn calculate_non_linear_saturation_returns_expected_values() {
        let mut filter = Filter::new(48000);

        filter.stage4_output = 0.0;
        let expected_result = 0.0;
        let result = filter.calculate_non_linear_saturation();
        assert!(f32_value_equality(result, expected_result));

        filter.stage4_output = 0.9;
        let expected_result = 0.7785;
        let result = filter.calculate_non_linear_saturation();
        assert!(f32_value_equality(result, expected_result));

        filter.stage4_output = -0.5;
        let expected_result = -0.4791666;
        let result = filter.calculate_non_linear_saturation();
        assert!(f32_value_equality(result, expected_result));
    }
}
