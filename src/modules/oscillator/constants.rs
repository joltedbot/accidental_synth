// SHARED CONSTANTS
pub const RADS_PER_CYCLE: f32 = std::f32::consts::TAU;
pub const MIN_PHASE: f32 = 0.0;
pub const MAX_PHASE: f32 = 1.0;
pub const DEFAULT_PHASE: f32 = 0.0;
pub const DEFAULT_X_COORDINATE: f32 = 0.0;
pub const DEFAULT_X_INCREMENT: f32 = 1.0;
pub const DEFAULT_KEY_SYNC_ENABLED: bool = false;
pub const DEFAULT_HARD_SYNC_ENABLED: bool = false;
pub const DEFAULT_POLARITY_FLIPPED: bool = false;
pub const MIN_CLIP_BOOST: u8 = 0;
pub const MAX_CLIP_BOOST: u8 = 30;

// Oscillator Shape Specific Constants
pub const DEFAULT_AMPLITUDE_MODULATION_AMOUNT: f32 = 4.0;
pub const DEFAULT_AM_TONE_AMOUNT: f32 = 1.0;
pub const DEFAULT_PULSE_WIDTH_ADJUSTMENT: f32 = 0.5;
pub const OSCILLATOR_MOD_TO_PWM_ADJUSTMENT_FACTOR: f32 = 0.5;

// Oscillator Tuning Constants
pub const MAX_MIDI_NOTE_NUMBER: i16 = 127;
pub const MIN_MIDI_NOTE_NUMBER: i16 = 0;
pub const MAX_NOTE_FREQUENCY: f32 = 12543.854;
pub const MIN_NOTE_FREQUENCY: f32 = 8.175;
pub const DEFAULT_NOTE_FREQUENCY: f32 = 261.625;
pub const DEFAULT_PORTAMENTO_TIME_IN_BUFFERS: u16 = 7;
pub const DEFAULT_PORTAMENTO_ENABLED: bool = false;
