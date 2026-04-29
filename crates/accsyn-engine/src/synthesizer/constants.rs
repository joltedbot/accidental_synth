// MIDI Constants
pub const NORMAL_TO_BOOL_SWITCH_ON_VALUE: f32 = 0.5;
pub const PITCH_BEND_AMOUNT_ZERO_POINT: u16 = 8192;
pub const PITCH_BEND_AMOUNT_MAX_VALUE: u16 = 16383;
pub const CENTS_PER_SEMITONE: u16 = 100;
pub const MIN_PITCH_BEND_RANGE: u8 = 2;
pub const MAX_PITCH_BEND_RANGE: u8 = 12;
pub const MAX_MIDI_KEY_VELOCITY: f32 = 1.0;
pub const MIN_VELOCITY_CURVE_EXPONENT: f32 = 0.25;
pub const MAX_VELOCITY_CURVE_EXPONENT: f32 = 4.0;

// Envelope Constants
pub const ENVELOPE_INDEX_AMP: i32 = 0;
pub const ENVELOPE_INDEX_FILTER: i32 = 1;
pub const ENVELOPE_INDEX_PITCH: i32 = 2;

// LFO Constants
pub const LFO_INDEX_MOD_WHEEL: i32 = 0;
pub const LFO_INDEX_FILTER: i32 = 1;

// Audio Constants
pub const SAMPLE_PRODUCER_LOOP_SLEEP_DURATION_MICROSECONDS: u64 = 100;

// MISC Constants
pub const SYNTHESIZER_MESSAGE_SENDER_CAPACITY: usize = 10;
pub const MAX_PATCH_NAME_LENGTH: usize = 24;
pub const MAX_PATCH_FILE_SIZE: u64 = 5120;

// Patch Save Status Messages
pub const PATCH_SAVE_SUCCESS: &str = "Patch saved successfully!";
pub const PATCH_SAVE_ALREADY_EXISTS: &str = "Patch name already exists!";
pub const PATCH_SAVE_FAILURE: &str = "Failed to save patch!";
pub const PATCH_DELETE_SUCCESS: &str = "Patch deleted successfully!";
pub const PATCH_DELETE_FILE_DOES_NOT_EXIST: &str = "Invalid patch, file does not exist!";
pub const PATCH_DELETE_FAILURE: &str = "Failed to delete patch!";
