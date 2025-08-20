use crate::modules::oscillator::WaveShape;

// MIDI Constants
pub const MAX_MIDI_VALUE: u8 = 127;
pub const MIDI_SWITCH_MAX_OFF_VALUE: u8 = 63;
pub const DEFAULT_FIXED_VELOCITY_STATE: bool = false;
pub const PITCH_BEND_AMOUNT_ZERO_POINT: i16 = 8192;
pub const PITCH_BEND_AMOUNT_MAX_VALUE: i16 = 16383;
pub const PITCH_BEND_AMOUNT_CENTS: i16 = 1200;
pub const EXPONENTIAL_FILTER_COEFFICIENT: f32 = 9.903487; // filter range 20,000hz so log(20000) = 9.903487
pub const EXPONENTIAL_LEVEL_COEFFICIENT: f32 = 6.908; // Level linear range is 1000x so log(1000) = 6.908
pub const EXPONENTIAL_LFO_COEFFICIENT: f32 = 13.81551; // log(100_000) = 13.81551
pub const LEVEL_CURVE_LINEAR_RANGE: f32 = 1000.0; // Level range is -60 to 0 = 60dbfs so 10^(60/20) = 1000x
pub const DEFAULT_VELOCITY_CURVE: u8 = 80;
pub const DEFAULT_VELOCITY: f32 = 1.0;
pub const DEFAULT_MIDI_NOTE: u8 = 60;

// Tuner Constants
pub const OSCILLATOR_FINE_TUNE_MAX_CENTS: i8 = 99;
pub const OSCILLATOR_FINE_TUNE_MIN_CENTS: i8 = -99;
pub const OSCILLATOR_COURSE_TUNE_MAX_INTERVAL: i8 = 12;
pub const OSCILLATOR_COURSE_TUNE_MIN_INTERVAL: i8 = -12;

// Oscillator Constants
pub const DEFAULT_OSCILLATOR_OUTPUT_LEVEL: f32 = 1.0;
pub const DEFAULT_OSCILLATOR_OUTPUT_PAN: f32 = 0.0;
pub const DEFAULT_OSCILLATOR_WAVE_SHAPE: WaveShape = WaveShape::Saw;
pub const DEFAULT_SUB_OSCILLATOR_INTERVAL: i8 = -12;
pub const DEFAULT_SUB_OSCILLATOR_LEVEL: f32 = 0.0;
pub const DEFAULT_OSCILLATOR_KEY_SYNC_STATE: bool = true;

// Envelope Constants
pub const DEFAULT_FILTER_ENVELOPE_AMOUNT: f32 = 0.0;
pub const ENVELOPE_INDEX_AMP_ENVELOPE: usize = 0;
pub const ENVELOPE_INDEX_FILTER_ENVELOPE: usize = 1;

// Mixer Constants
pub const DEFAULT_OUTPUT_LEVEL: f32 = 0.5;
pub const DEFAULT_OUTPUT_PAN: f32 = 0.0;

// LFO constants
pub const LFO_INDEX_FILTER_MODULATION: usize = 0;
pub const LFO_INDEX_GENERAL_LFO1: usize = 1;
