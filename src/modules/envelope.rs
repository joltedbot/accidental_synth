const ENVELOPE_MAX_LEVEL: f32 = 1.0;
const ENVELOPE_MIN_LEVEL: f32 = 0.0;
pub const ENVELOPE_MAX_MILLISECONDS: f32 = 10000.0;
pub const ENVELOPE_MIN_MILLISECONDS: f32 = 0.05;
const MIN_SUSTAIN_LEVEL: f32 = 0.0;
const MAX_SUSTAIN_LEVEL: f32 = 1.0;
const DEFAULT_SUSTAIN_LEVEL: f32 = 0.8;
pub const DEFAULT_ATTACK_LEVEL_INCREMENT: f32 = 0.0002;
pub const DEFAULT_DECAY_LEVEL_INCREMENT: f32 = 0.0002;
pub const DEFAULT_RELEASE_LEVEL_INCREMENT: f32 = 0.0002;
pub const DEFAULT_DECAY_MILLISECONDS: f32 = 100.0;

#[derive(Default, Debug, Clone, Copy, PartialEq)]
enum Stage {
    #[default]
    Off,
    Attack,
    Decay,
    Sustain,
    Release,
}

#[derive(Debug, Clone, Copy, PartialEq)]
enum StageAction {
    Start,
    Stop,
    NextStage,
}

#[derive(Default, Debug, Clone, Copy, PartialEq)]
pub struct Envelope {
    stage: Stage,
    level: f32,
    sample_rate: u32,
    is_inverted: bool,
    milliseconds_per_sample: f32,
    attack_level_increment: f32,
    decay_milliseconds: f32,
    decay_level_increment: f32,
    sustain_level: f32,
    release_level_increment: f32,
}

impl Envelope {
    pub fn new(sample_rate: u32) -> Self {
        log::info!("Constructing Envelope Module");

        let milliseconds_per_sample = 1000.0 / sample_rate as f32;

        Self {
            level: ENVELOPE_MIN_LEVEL,
            sample_rate,
            milliseconds_per_sample,
            is_inverted: false,
            sustain_level: DEFAULT_SUSTAIN_LEVEL,
            attack_level_increment: DEFAULT_ATTACK_LEVEL_INCREMENT,
            decay_level_increment: DEFAULT_DECAY_LEVEL_INCREMENT,
            decay_milliseconds: DEFAULT_DECAY_MILLISECONDS,
            release_level_increment: DEFAULT_RELEASE_LEVEL_INCREMENT,
            stage: Stage::Off,
        }
    }

    pub fn generate(&mut self) -> f32 {
        let mut envelope_output_value = self.next_value();
        if self.is_inverted {
            envelope_output_value = ENVELOPE_MAX_LEVEL - envelope_output_value;
        }
        envelope_output_value
    }

    pub fn gate_on(&mut self) {
        self.state_action(StageAction::Start);
    }

    pub fn gate_off(&mut self) {
        self.state_action(StageAction::Stop);
    }

    pub fn set_attack_milliseconds(&mut self, milliseconds: f32) {
        let clamped_milliseconds = milliseconds.max(self.milliseconds_per_sample);
        self.attack_level_increment = self.level_increments_from_milliseconds(
            ENVELOPE_MIN_LEVEL,
            ENVELOPE_MAX_LEVEL,
            clamped_milliseconds,
        );
    }

    pub fn set_decay_milliseconds(&mut self, milliseconds: f32) {
        self.decay_milliseconds = milliseconds.max(self.milliseconds_per_sample);

        self.decay_level_increment = self.level_increments_from_milliseconds(
            ENVELOPE_MAX_LEVEL,
            self.sustain_level,
            self.decay_milliseconds,
        );
    }

    pub fn set_sustain_level(&mut self, level: f32) {
        if !(MIN_SUSTAIN_LEVEL..=MAX_SUSTAIN_LEVEL).contains(&level) {
            log::debug!(
                "set_sustain_level: level exceeded range (0.0-1.0) but was clamped: level: {level}"
            );
        }
        self.sustain_level = level.clamp(0.0, 1.0);

        self.decay_level_increment = self.level_increments_from_milliseconds(
            ENVELOPE_MAX_LEVEL,
            self.sustain_level,
            self.decay_milliseconds,
        );
    }

    pub fn set_release_milliseconds(&mut self, milliseconds: f32) {
        let clamped_milliseconds = milliseconds.max(self.milliseconds_per_sample);
        self.release_level_increment = self.level_increments_from_milliseconds(
            ENVELOPE_MAX_LEVEL,
            ENVELOPE_MIN_LEVEL,
            clamped_milliseconds,
        );
    }

    pub fn set_is_inverted(&mut self, is_inverted: bool) {
        self.is_inverted = is_inverted
    }

    fn state_action(&mut self, action: StageAction) {
        match (action, self.stage) {
            (StageAction::Start, _) => {
                self.stage = Stage::Attack;
            }
            (StageAction::Stop, Stage::Off) => {}
            (StageAction::Stop, Stage::Release) => {}
            (StageAction::Stop, _) => {
                self.stage = Stage::Release;
            }
            (StageAction::NextStage, Stage::Attack) => {
                self.stage = Stage::Decay;
            }
            (StageAction::NextStage, Stage::Decay) => {
                self.stage = Stage::Sustain;
            }
            (StageAction::NextStage, Stage::Release) => {
                self.level = ENVELOPE_MIN_LEVEL;
                self.stage = Stage::Off;
            }
            (StageAction::NextStage, _) => {
                log::debug!(
                    "state_action(): invalid state transition: state: {:?}, action: {:?}",
                    self.stage,
                    action
                );
            }
        }
    }

    fn next_value(&mut self) -> f32 {
        match self.stage {
            Stage::Off => self.level,
            Stage::Attack => self.attack_next_value(),
            Stage::Decay => self.decay_next_value(),
            Stage::Sustain => self.sustain_next_value(),
            Stage::Release => self.release_next_value(),
        }
    }

    fn attack_next_value(&mut self) -> f32 {
        self.level += self.attack_level_increment;

        if self.level >= ENVELOPE_MAX_LEVEL {
            self.level = ENVELOPE_MAX_LEVEL;
            self.state_action(StageAction::NextStage);
        }

        self.level
    }

    fn decay_next_value(&mut self) -> f32 {
        self.level -= self.decay_level_increment;

        if self.level <= self.sustain_level {
            self.level = self.sustain_level;
            self.state_action(StageAction::NextStage);
        }

        self.level
    }

    fn sustain_next_value(&mut self) -> f32 {
        self.level
    }

    fn release_next_value(&mut self) -> f32 {
        self.level -= self.release_level_increment;

        if self.level <= ENVELOPE_MIN_LEVEL {
            self.level = ENVELOPE_MIN_LEVEL;
            self.state_action(StageAction::NextStage);
        }

        self.level
    }

    fn level_increments_from_milliseconds(
        &self,
        current_level: f32,
        target_level: f32,
        milliseconds: f32,
    ) -> f32 {
        let range = (target_level - current_level).abs();

        if milliseconds <= self.milliseconds_per_sample {
            return range;
        }

        range / (milliseconds / self.milliseconds_per_sample)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn f32_value_equality(value_1: f32, value_2: f32) -> bool {
        (value_1 - value_2).abs() <= f32::EPSILON
    }

    #[test]
    fn new_returns_envelope_with_correct_default_values() {
        let sample_rate = 48000;
        let envelope = Envelope::new(sample_rate);

        assert_eq!(envelope.stage, Stage::Off);
        assert_eq!(envelope.level, ENVELOPE_MIN_LEVEL);
        assert_eq!(envelope.sample_rate, sample_rate);
        assert!(!envelope.is_inverted);
        assert!(f32_value_equality(
            envelope.milliseconds_per_sample,
            0.0208333
        ));
        assert_eq!(envelope.sustain_level, DEFAULT_SUSTAIN_LEVEL);
        assert_eq!(
            envelope.attack_level_increment,
            DEFAULT_ATTACK_LEVEL_INCREMENT
        );
        assert_eq!(
            envelope.decay_level_increment,
            DEFAULT_DECAY_LEVEL_INCREMENT
        );
        assert_eq!(
            envelope.release_level_increment,
            DEFAULT_RELEASE_LEVEL_INCREMENT
        );
    }

    #[test]
    fn set_attack_milliseconds_stores_correct_attack_level_increment() {
        let sample_rate = 44100;
        let attack_ms = 100.0;
        let expected_increment = 0.0002267;

        let mut envelope = Envelope::new(sample_rate);
        envelope.set_attack_milliseconds(attack_ms);

        assert!(f32_value_equality(
            envelope.attack_level_increment,
            expected_increment
        ));
    }

    #[test]
    fn set_decay_milliseconds_stores_correct_level_increment() {
        let sample_rate = 44100;
        let decay_ms = 100.0;
        let sustain_level = 0.5;
        let expected_increment = 0.0001133;

        let mut envelope = Envelope::new(sample_rate);
        envelope.set_sustain_level(sustain_level);
        envelope.set_decay_milliseconds(decay_ms);

        assert!(f32_value_equality(envelope.decay_milliseconds, decay_ms));
        assert!(f32_value_equality(
            envelope.decay_level_increment,
            expected_increment
        ));
    }

    #[test]
    fn sustain_level_stores_correct_level_increment() {
        let sample_rate = 44100;
        let sustain_level = 0.9;
        let decay_ms = 20.0;
        let expected_increment = 0.0001133;

        let mut envelope = Envelope::new(sample_rate);
        envelope.set_decay_milliseconds(decay_ms);
        envelope.set_sustain_level(sustain_level);

        assert!(f32_value_equality(envelope.sustain_level, sustain_level));
        assert!(f32_value_equality(
            envelope.decay_level_increment,
            expected_increment
        ));
    }

    #[test]
    fn set_sustain_level_correctly_clamps_values_to_range() {
        let sample_rate = 192000;
        let mut envelope = Envelope::new(sample_rate);

        envelope.set_sustain_level(f32::MIN);
        assert_eq!(envelope.sustain_level, MIN_SUSTAIN_LEVEL);

        envelope.set_sustain_level(f32::MAX);
        assert_eq!(envelope.sustain_level, MAX_SUSTAIN_LEVEL);
    }

    #[test]
    fn set_release_milliseconds_correctly_sets_level_increment() {
        let sample_rate = 48000;
        let release_ms = 30.0;
        let mut envelope = Envelope::new(sample_rate);
        let expected_increment = 0.0006944;

        envelope.set_release_milliseconds(release_ms);
        assert!(f32_value_equality(
            envelope.release_level_increment,
            expected_increment
        ));
    }

    #[test]
    fn start_correctly_initiates_attack_stage_from_all_stages() {
        let mut envelope = Envelope::new(44100);

        envelope.stage = Stage::Off;
        envelope.gate_on();
        assert_eq!(envelope.stage, Stage::Attack);

        envelope.stage = Stage::Decay;
        envelope.gate_on();
        assert_eq!(envelope.stage, Stage::Attack);

        envelope.stage = Stage::Sustain;
        envelope.gate_on();
        assert_eq!(envelope.stage, Stage::Attack);

        envelope.stage = Stage::Release;
        envelope.gate_on();
        assert_eq!(envelope.stage, Stage::Attack);
    }

    #[test]
    fn stop_correctly_initiates_release_stage_from_all_valid_stages() {
        let mut envelope = Envelope::new(44100);

        envelope.stage = Stage::Attack;
        envelope.gate_off();
        assert_eq!(envelope.stage, Stage::Release);

        envelope.stage = Stage::Decay;
        envelope.gate_off();
        assert_eq!(envelope.stage, Stage::Release);

        envelope.stage = Stage::Sustain;
        envelope.gate_off();
        assert_eq!(envelope.stage, Stage::Release);
    }

    #[test]
    fn stop_correctly_does_not_transition_to_release_from_off_or_release() {
        let mut envelope = Envelope::new(44100);

        envelope.stage = Stage::Off;
        envelope.gate_off();
        assert_eq!(envelope.stage, Stage::Off);

        envelope.stage = Stage::Release;
        envelope.gate_off();
        assert_eq!(envelope.stage, Stage::Release);
    }

    #[test]
    fn attack_value_generation() {
        let sample_rate = 48000;
        let mut envelope = Envelope::new(sample_rate);
        let attack_ms = 100.0;
        envelope.set_attack_milliseconds(attack_ms);
        let expected_release_level_increment = 0.0002083;

        envelope.gate_on();
        assert_eq!(envelope.stage, Stage::Attack);
        assert_eq!(envelope.level, ENVELOPE_MIN_LEVEL);

        let first_value = envelope.generate();
        assert!(f32_value_equality(
            first_value,
            ENVELOPE_MIN_LEVEL + expected_release_level_increment
        ));

        let second_value = envelope.generate();
        assert!(f32_value_equality(
            second_value,
            first_value + expected_release_level_increment
        ));
    }

    #[test]
    fn decay_stage_returns_the_correctly_incremented_level() {
        let sample_rate = 48000;
        let mut envelope = Envelope::new(sample_rate);
        let decay_ms = 100.0;
        let sustain_level = 0.5;
        let expected_release_level_increment = 0.0001041;

        envelope.set_decay_milliseconds(decay_ms);
        envelope.set_sustain_level(sustain_level);
        envelope.stage = Stage::Decay;
        envelope.level = ENVELOPE_MAX_LEVEL;

        let first_value = envelope.generate();
        assert!(f32_value_equality(
            first_value,
            ENVELOPE_MAX_LEVEL - expected_release_level_increment
        ));

        let second_value = envelope.generate();
        assert!(f32_value_equality(
            second_value,
            first_value - expected_release_level_increment
        ));
    }

    #[test]
    fn sustain_stage_correctly_returns_sustain_level_continuously() {
        let mut envelope = Envelope::new(44100);
        let sustain_level = 0.7;
        envelope.set_sustain_level(sustain_level);
        envelope.level = sustain_level;
        envelope.stage = Stage::Sustain;

        for _ in 0..100 {
            let value = envelope.generate();
            assert!(f32_value_equality(value, sustain_level));
        }
    }

    #[test]
    fn release_stage_returns_the_correctly_incremented_level() {
        let sample_rate = 44100;
        let release_ms = 100.0;
        let start_level = 0.8;
        let expected_release_level_increment = 0.0002267;

        let mut envelope = Envelope::new(sample_rate);
        envelope.set_release_milliseconds(release_ms);
        envelope.stage = Stage::Release;
        envelope.level = start_level;

        let first_value = envelope.generate();
        assert!(f32_value_equality(
            first_value,
            start_level - expected_release_level_increment
        ));

        let second_value = envelope.generate();
        assert!(f32_value_equality(
            second_value,
            first_value - expected_release_level_increment
        ));
    }

    #[test]
    fn off_stage_correctly_returns_min_level_continuously() {
        let mut envelope = Envelope::new(44100);
        envelope.stage = Stage::Off;
        envelope.level = ENVELOPE_MIN_LEVEL;

        for _ in 0..100 {
            let value = envelope.generate();
            assert!(f32_value_equality(value, ENVELOPE_MIN_LEVEL));
        }
    }

    #[test]
    fn zero_millisecond_attack_correctly_immediately_transitions_to_decay() {
        let mut envelope = Envelope::new(44100);
        envelope.set_attack_milliseconds(0.0);
        envelope.gate_on();

        let value = envelope.generate();
        assert!(f32_value_equality(value, ENVELOPE_MAX_LEVEL));
        assert_eq!(envelope.stage, Stage::Decay);
    }

    #[test]
    fn zero_millisecond_decay_correctly_immediately_transitions_to_sustain() {
        let mut envelope = Envelope::new(44100);
        let sustain_level = 0.5;
        envelope.set_sustain_level(sustain_level);
        envelope.set_decay_milliseconds(0.0);
        envelope.level = ENVELOPE_MAX_LEVEL;
        envelope.stage = Stage::Decay;

        let value = envelope.generate();
        assert!(f32_value_equality(value, sustain_level));
        assert_eq!(envelope.stage, Stage::Sustain);
    }

    #[test]
    fn zero_millisecond_release_correctly_immediately_transitions_to_off() {
        let mut envelope = Envelope::new(44100);
        envelope.set_release_milliseconds(0.0);
        envelope.stage = Stage::Release;
        envelope.level = 0.8;

        let value = envelope.generate();
        assert!(f32_value_equality(value, ENVELOPE_MIN_LEVEL));
        assert_eq!(envelope.stage, Stage::Off);
    }

    #[test]
    fn level_increments_from_milliseconds_correctly_returns_zero_when_current_and_target_levels_are_equal()
     {
        let envelope = Envelope::new(44100);
        let increment = envelope.level_increments_from_milliseconds(0.5, 0.5, 100.0);

        assert_eq!(increment, 0.0);
    }

    #[test]
    fn envelope_correctly_inverts_levels_when_in_inverted_mode() {
        let mut envelope = Envelope::new(44100);
        envelope.is_inverted = false;

        let pre_invert_expected_result = 0.0;
        let pre_invert_result = envelope.generate();
        assert_eq!(pre_invert_result, pre_invert_expected_result);

        envelope.set_is_inverted(true);

        let post_invert_expected_result = 1.0;
        let post_invert_result = envelope.generate();
        assert_eq!(post_invert_result, post_invert_expected_result);
    }

    #[test]
    fn envelope_correctly_returns_non_inverted_levels_when_switching_back_to_non_inverted_mode() {
        let mut envelope = Envelope::new(44100);
        envelope.is_inverted = true;

        let pre_invert_expected_result = 1.0;
        let pre_invert_result = envelope.generate();
        assert_eq!(pre_invert_result, pre_invert_expected_result);

        envelope.set_is_inverted(false);

        let return_to_not_inverted_expected_result = 0.0;
        let return_to_not_inverted_result = envelope.generate();
        assert_eq!(
            return_to_not_inverted_result,
            return_to_not_inverted_expected_result
        );
    }

    #[test]
    fn envelope_correctly_rejects_invalid_nextstage_transition() {
        let mut envelope = Envelope::new(44100);
        envelope.stage = Stage::Sustain;
        envelope.state_action(StageAction::NextStage);
        assert_eq!(envelope.stage, Stage::Sustain);
    }

    #[test]
    fn envelope_correctly_rejects_invalid_stop_action() {
        let mut envelope = Envelope::new(44100);
        envelope.stage = Stage::Off;
        envelope.state_action(StageAction::Stop);
        assert_eq!(envelope.stage, Stage::Off);
    }

    #[test]
    fn envelope_correctly_transitions_through_all_stages() {
        let sample_rate = 44100;
        let mut envelope = Envelope::new(sample_rate);
        envelope.set_attack_milliseconds(10.0);
        envelope.set_decay_milliseconds(10.0);
        envelope.set_sustain_level(0.5);
        envelope.set_release_milliseconds(10.0);

        // Before the first note
        assert_eq!(envelope.stage, Stage::Off);
        assert_eq!(envelope.level, ENVELOPE_MIN_LEVEL);

        // Midi Note Start
        envelope.gate_on();
        assert_eq!(envelope.stage, Stage::Attack);
        while envelope.stage == Stage::Attack {
            envelope.generate();
        }

        // Transition to decay stage
        assert_eq!(envelope.stage, Stage::Decay);
        assert_eq!(envelope.level, ENVELOPE_MAX_LEVEL);
        while envelope.stage == Stage::Decay {
            envelope.generate();
        }

        // Transition to sustain stage
        assert_eq!(envelope.stage, Stage::Sustain);
        assert_eq!(envelope.level, envelope.sustain_level);
        for _ in 0..100 {
            envelope.generate();
            assert_eq!(envelope.stage, Stage::Sustain);
        }

        // Midi Note Stop
        envelope.gate_off();

        // Transition to release stage
        assert_eq!(envelope.stage, Stage::Release);
        while envelope.stage == Stage::Release {
            envelope.generate();
        }

        // Transition back to the Off stage
        assert_eq!(envelope.stage, Stage::Off);
        assert!(f32_value_equality(envelope.level, ENVELOPE_MIN_LEVEL));
    }
}
