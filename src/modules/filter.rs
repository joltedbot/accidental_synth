use std::f32::consts::PI;
const DEFAULT_CUTOFF_FREQUENCY: f32 = 20000.0;
const DEFAULT_RESONANCE: f32 = 1.0;
const DEFAULT_IS_FOUR_POLE: bool = true;

#[derive(Default, Copy, Clone, Debug, PartialEq)]
struct FilterParameters {
    low_pass: f32,
    high_pass: f32,
    band_pass: f32,
}

#[derive(Default, Copy, Clone, Debug, PartialEq)]
pub struct Filter {
    sample_rate: u32,
    cutoff_frequency: f32,
    resonance: f32,
    coefficient: f32,
    is_four_pole: bool,
    left_2_pole: FilterParameters,
    right_2_pole: FilterParameters,
    left_4_pole: FilterParameters,
    right_4_pole: FilterParameters,
}

impl Filter {
    pub fn new(sample_rate: u32) -> Self {
        log::info!("Constructing Filter Module");
        Self {
            sample_rate,
            cutoff_frequency: DEFAULT_CUTOFF_FREQUENCY,
            resonance: DEFAULT_RESONANCE,
            is_four_pole: DEFAULT_IS_FOUR_POLE,
            coefficient: calculate_coefficient(DEFAULT_CUTOFF_FREQUENCY, sample_rate),
            ..Default::default()
        }
    }

    pub fn filter(&mut self, left_sample: f32, right_sample: f32) -> (f32, f32) {
        let mut left_filtered_sample = chamberlin_2_pole_low_pass(
            left_sample,
            self.coefficient,
            self.resonance,
            &mut self.left_2_pole,
        );
        let mut right_filtered_sample = chamberlin_2_pole_low_pass(
            right_sample,
            self.coefficient,
            self.resonance,
            &mut self.right_2_pole,
        );

        if self.is_four_pole {
            left_filtered_sample = chamberlin_2_pole_low_pass(
                left_sample,
                self.coefficient,
                self.resonance,
                &mut self.left_4_pole,
            );
            right_filtered_sample = chamberlin_2_pole_low_pass(
                right_sample,
                self.coefficient,
                self.resonance,
                &mut self.right_4_pole,
            );
        }

        (left_filtered_sample, right_filtered_sample)
    }

    pub fn set_cutoff_frequency(&mut self, cut_off_frequency: f32) {
        self.cutoff_frequency = cut_off_frequency; // * CUTOFF_FREQUENCY_POINT_ADJUSTMENT;
        self.coefficient = calculate_coefficient(cut_off_frequency, self.sample_rate);
    }

    pub fn set_resonance(&mut self, resonance: f32) {
        self.resonance = resonance;
    }

    pub fn set_is_four_pole(&mut self, is_four_pole: bool) {
        self.is_four_pole = is_four_pole;
    }
}

fn chamberlin_2_pole_low_pass(
    sample: f32,
    coefficient: f32,
    resonance: f32,
    parameters: &mut FilterParameters,
) -> f32 {
    parameters.low_pass += coefficient * parameters.band_pass;
    parameters.high_pass =
        (resonance * sample) - parameters.low_pass - (resonance * parameters.band_pass);
    parameters.band_pass += coefficient * parameters.high_pass;
    parameters.low_pass
}

fn calculate_coefficient(frequency: f32, sample_rate: u32) -> f32 {
    2.0 * (PI * (frequency / (sample_rate as f32 * 2.0))).sin()
}
