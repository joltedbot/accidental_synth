const ENVELOPE_MIN_LEVEL: f32 = 0.0;
const ENVELOPE_MAX_LEVEL: f32 = 1.0;
const DEFAULT_SUSTAIN_LEVEL: f32 = 1.0;
const DEFAULT_ATTACK_LEVEL_INCREMENT: f32 = 0.001;
const DEFAULT_DECAY_LEVEL_INCREMENT: f32 = 0.001;
const DEFAULT_RELEASE_LEVEL_INCREMENT: f32 = 0.001;

#[derive(Default, Debug, Clone, Copy, PartialEq)]
enum State {
    #[default]
    Off,
    Attack,
    Decay,
    Sustain,
    Release,
}

#[derive(Debug, Clone, Copy, PartialEq)]
enum StateAction {
    Start,
    Stop,
    NextState,
}

#[derive(Default, Debug, Clone, Copy, PartialEq)]
pub struct Envelope {
    state: State,
    level: f32,
    sample_rate: u32,
    milliseconds_per_sample: f32,
    attack_level_increment: f32,
    decay_milliseconds: u32,
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
            sustain_level: DEFAULT_SUSTAIN_LEVEL,
            attack_level_increment: DEFAULT_ATTACK_LEVEL_INCREMENT,
            decay_level_increment: DEFAULT_DECAY_LEVEL_INCREMENT,
            release_level_increment: DEFAULT_RELEASE_LEVEL_INCREMENT,
            ..Default::default()
        }
    }

    pub fn next(&mut self) -> f32 {
        self.next_value()
    }

    pub fn start(&mut self) {
        self.state_action(StateAction::Start);
    }

    pub fn stop(&mut self) {
        self.state_action(StateAction::Stop);
    }

    pub fn set_attack_milliseconds(&mut self, milliseconds: u32) {
        self.attack_level_increment = self.level_increments_from_milliseconds(
            ENVELOPE_MIN_LEVEL,
            ENVELOPE_MAX_LEVEL,
            milliseconds,
        );
    }

    pub fn set_decay_milliseconds(&mut self, milliseconds: u32) {
        self.decay_milliseconds = milliseconds;
        self.decay_level_increment = self.level_increments_from_milliseconds(
            ENVELOPE_MAX_LEVEL,
            self.sustain_level,
            milliseconds,
        );
    }

    pub fn set_sustain_level(&mut self, level: f32) {
        if !(0.0..=1.0).contains(&level) {
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

    pub fn set_release_milliseconds(&mut self, milliseconds: u32) {
        self.release_level_increment = self.level_increments_from_milliseconds(
            ENVELOPE_MAX_LEVEL,
            ENVELOPE_MIN_LEVEL,
            milliseconds,
        );
    }

    fn state_action(&mut self, action: StateAction) {
        match (action, self.state) {
            (StateAction::Start, _) => {
                self.state = State::Attack;
            }
            (StateAction::Stop, State::Off) => {}
            (StateAction::Stop, State::Release) => {}
            (StateAction::Stop, _) => {
                self.state = State::Release;
            }
            (StateAction::NextState, State::Attack) => {
                self.state = State::Decay;
            }
            (StateAction::NextState, State::Decay) => {
                self.state = State::Sustain;
            }
            (StateAction::NextState, State::Release) => {
                self.level = ENVELOPE_MIN_LEVEL;
                self.state = State::Off;
            }
            (StateAction::NextState, _) => {}
        }
    }

    fn next_value(&mut self) -> f32 {
        match self.state {
            State::Off => self.level,
            State::Attack => self.attack_next_value(),
            State::Decay => self.decay_next_value(),
            State::Sustain => self.sustain_next_value(),
            State::Release => self.release_next_value(),
        }
    }

    fn attack_next_value(&mut self) -> f32 {
        self.level += self.attack_level_increment;

        if self.level >= ENVELOPE_MAX_LEVEL {
            self.level = ENVELOPE_MAX_LEVEL;
            self.state_action(StateAction::NextState);
        }

        self.level
    }

    fn decay_next_value(&mut self) -> f32 {
        self.level -= self.decay_level_increment;

        if self.level <= self.sustain_level {
            self.level = self.sustain_level;
            self.state_action(StateAction::NextState);
        }

        self.level
    }

    fn sustain_next_value(&mut self) -> f32 {
        self.level
    }

    fn release_next_value(&mut self) -> f32 {
        self.level -= self.attack_level_increment;

        if self.level <= ENVELOPE_MIN_LEVEL {
            self.level = ENVELOPE_MIN_LEVEL;
            self.state_action(StateAction::NextState);
        }

        self.level
    }

    fn level_increments_from_milliseconds(
        &self,
        current_level: f32,
        target_level: f32,
        milliseconds: u32,
    ) -> f32 {
        let range = (target_level - current_level).abs();
        range / (milliseconds as f32 / self.milliseconds_per_sample)
    }
}
