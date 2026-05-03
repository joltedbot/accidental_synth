// SHARED CONSTANTS
/// Radians in one full waveform cycle (2 * PI).
pub const RADS_PER_CYCLE: f32 = std::f32::consts::TAU;
/// Minimum phase value (0.0).
pub const MIN_PHASE: f32 = 0.0;
/// Maximum phase value (1.0).
pub const MAX_PHASE: f32 = 1.0;
/// Default starting phase for oscillators.
pub const DEFAULT_PHASE: f32 = 0.0;
/// Default X coordinate for waveform generators.
pub const DEFAULT_X_COORDINATE: f32 = 0.0;
/// Default X increment per sample for waveform generators.
pub const DEFAULT_X_INCREMENT: f32 = 1.0;
/// Default state for key sync (phase reset on note-on).
pub const DEFAULT_KEY_SYNC_ENABLED: bool = false;
/// Default state for hard sync between oscillators.
pub const DEFAULT_HARD_SYNC_ENABLED: bool = false;
/// Default state for waveform polarity flip.
pub const DEFAULT_POLARITY_FLIPPED: bool = false;
/// Default state for sustain pedal.
pub const DEFAULT_SUSTAIN_PEDAL_FLIPPED: bool = false;
/// Minimum clipper boost value in dB.
pub const MIN_CLIP_BOOST: u8 = 0;
/// Maximum clipper boost value in dB.
pub const MAX_CLIP_BOOST: u8 = 30;

// Oscillator Shape Specific Constants
/// Default modulation depth for AM synthesis.
pub const DEFAULT_AMPLITUDE_MODULATION_AMOUNT: f32 = 4.0;
/// Default AM tone amount (0.0 = ring mod, 1.0 = proper AM).
pub const DEFAULT_AM_TONE_AMOUNT: f32 = 1.0;
/// Default pulse width for the pulse wave oscillator.
pub const DEFAULT_PULSE_WIDTH_ADJUSTMENT: f32 = 0.5;
/// Scaling factor for converting oscillator modulation to pulse width modulation.
pub const OSCILLATOR_MOD_TO_PWM_ADJUSTMENT_FACTOR: f32 = 0.5;

// Oscillator Tuning Constants
/// Maximum MIDI note number (127).
pub const MAX_MIDI_NOTE_NUMBER: i16 = 127;
/// Minimum MIDI note number (0).
pub const MIN_MIDI_NOTE_NUMBER: i16 = 0;
/// Maximum oscillator frequency in Hz (MIDI note 127).
pub const MAX_NOTE_FREQUENCY: f32 = 12543.854;
/// Minimum oscillator frequency in Hz (MIDI note 0).
pub const MIN_NOTE_FREQUENCY: f32 = 8.175;
/// Default oscillator frequency in Hz (middle C).
pub const DEFAULT_NOTE_FREQUENCY: f32 = 261.625;
/// Default portamento glide time measured in buffer increments.
pub const DEFAULT_PORTAMENTO_TIME_IN_BUFFERS: u16 = 7;
/// Default state for portamento (pitch glide).
pub const DEFAULT_PORTAMENTO_ENABLED: bool = false;
