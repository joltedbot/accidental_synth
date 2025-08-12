// Derived from https://www.musicdsp.org/en/latest/Filters/253-perfect-lp4-filter.html

const DEFAULT_CUTOFF_FREQUENCY: f32 = 20000.0;
const NATURAL_LOG_OF_4: f32 = 1.386294361;
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
        let pole_coefficient =
            calculate_pole_coefficient(calculate_gain_coefficient(gain_coefficient));
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
        self.cutoff_frequency = cut_off_frequency;
        self.calculate_coefficients_on_cutoff_update(cut_off_frequency);
    }

    pub fn set_resonance(&mut self, resonance: f32) {
        self.resonance = resonance;
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

        self.stage1_output = input * self.coefficients.gain_coefficient
            + self.input_unit_delay * self.coefficients.gain_coefficient
            - self.coefficients.pole_coefficient * self.stage1_output;
        self.stage2_output = self.stage1_output * self.coefficients.gain_coefficient
            + self.stage1_unit_delay * self.coefficients.gain_coefficient
            - self.coefficients.pole_coefficient * self.stage2_output;
        self.stage3_output = self.stage2_output * self.coefficients.gain_coefficient
            + self.stage2_unit_delay * self.coefficients.gain_coefficient
            - self.coefficients.pole_coefficient * self.stage3_output;
        self.stage4_output = self.stage3_output * self.coefficients.gain_coefficient
            + self.stage3_unit_delay * self.coefficients.gain_coefficient
            - self.coefficients.pole_coefficient * self.stage4_output;

        self.stage4_output = self.stage4_output
            - ((self.stage4_output * self.stage4_output * self.stage4_output) / 6.0);

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
