/// Default values and constants for all synthesizer parameters.
pub struct Defaults {}

impl Defaults {
    // Audio Defaults
    /// Default audio output device index.
    pub const AUDIO_DEVICE_INDEX: i32 = 0;
    /// Default left output channel index.
    pub const LEFT_CHANNEL_INDEX: i32 = 0;
    /// Default right output channel index.
    pub const RIGHT_CHANNEL_INDEX: i32 = 1;
    /// Channel count for mono output.
    pub const MONO_CHANNEL_COUNT: u16 = 1;
    /// Sentinel value indicating a disabled output channel.
    pub const OUTPUT_CHANNEL_DISABLED_VALUE: i32 = -1;
    /// Default index into [`Defaults::SUPPORTED_SAMPLE_RATES`].
    pub const SAMPLE_RATE_INDEX: usize = 1;
    /// Available sample rates in Hz.
    pub const SUPPORTED_SAMPLE_RATES: [u32; 3] = [44100, 48000, 96000];
    /// Default index into [`Defaults::SUPPORTED_BUFFER_SIZES`].
    pub const BUFFER_SIZE_INDEX: usize = 2;
    /// Available audio buffer sizes in samples.
    pub const SUPPORTED_BUFFER_SIZES: [u32; 5] = [64, 128, 256, 512, 1024];

    // Mixer Defaults
    /// Default output mixer level (0.0–1.0).
    pub const OUTPUT_MIXER_LEVEL: f32 = 0.5;
    /// Default output mixer stereo balance (-1.0 left to 1.0 right).
    pub const OUTPUT_MIXER_BALANCE: f32 = 0.0;
    /// Default output mixer mute state.
    pub const OUTPUT_MIXER_IS_MUTED: bool = false;
    /// Default per-oscillator mixer level (0.0–1.0).
    pub const QUAD_MIXER_LEVEL: f32 = 1.0;
    /// Default per-oscillator sub-oscillator level (0.0–1.0).
    pub const QUAD_MIXER_SUB_LEVEL: f32 = 0.0;
    /// Default per-oscillator stereo balance (-1.0 left to 1.0 right).
    pub const QUAD_MIXER_BALANCE: f32 = 0.0;
    /// Maximum balance range for output and sub-oscillator mixers.
    pub const MAXIMUM_BALANCE_RANGE: f32 = 1.0;
    /// Minimum balance range for output and sub-oscillator mixers.
    pub const MINIMUM_BALANCE_RANGE: f32 = -1.0;
    /// Default per-oscillator mute state.
    pub const QUAD_MIXER_IS_MUTED: bool = false;

    // Global Panel Constants
    /// Default pitch bend range in semitones.
    pub const PITCH_BEND_RANGE: u8 = 12;
    /// Default portamento time as a normalized value (0.0–1.0).
    pub const PORTAMENTO_TIME_NORMAL_VALUE: f32 = 0.0;
    /// Default velocity curve as a normalized value (0.0–1.0).
    pub const VELOCITY_CURVE_NORMAL_VALUE: f32 = 0.5;
    /// Default velocity curve value.
    pub const LINEAR_VELOCITY_CURVE_EXPONENT: f32 = 1.0;
    /// Minimum allowed pitch bend range in semitones.
    pub const MINIMUM_PITCH_BEND_RANGE: u32 = 2;
    /// Maximum allowed pitch bend range in semitones.
    pub const MAXIMUM_PITCH_BEND_RANGE: u32 = 12;
}

/// Default delay effect parameters: [time, feedback, mix, unused].
pub const DELAY_DEFAULT_PARAMETERS: [f32; 4] = [0.5, 0.5, 0.5, 0.0];
/// Default autopan effect parameters: [rate, depth, unused, unused].
pub const AUTOPAN_DEFAULT_PARAMETERS: [f32; 4] = [0.1, 1.0, 0.0, 0.0];
/// Default tremolo effect parameters: [rate, depth, unused, unused].
pub const TREMOLO_DEFAULT_PARAMETERS: [f32; 4] = [0.1, 1.0, 0.0, 0.0];
/// Default Saturation effect parameters: [Amount, Cut, unused, unused].
pub const SATURATION_DEFAULT_PARAMETERS: [f32; 4] = [0.0, 0.0, 1.0, 0.0];
/// Default compressor effect parameters: [threshold, ratio, unused, unused].
pub const COMPRESSOR_DEFAULT_PARAMETERS: [f32; 4] = [0.0, 1.0, 0.0, 0.0];
/// Default clipper effect parameters: [threshold, unused, unused, unused].
pub const CLIPPER_DEFAULT_PARAMETERS: [f32; 4] = [1.0, 0.0, 0.0, 0.0];
/// Default gate effect parameters: [threshold, ratio, unused, unused].
pub const GATE_DEFAULT_PARAMETERS: [f32; 4] = [0.0, 1.0, 0.0, 0.0];

impl Defaults {
    /// MIDI note number to frequency (Hz) and note name lookup table.
    pub const MIDI_NOTE_FREQUENCIES: [(f32, &str); 128] = [
        (8.175, "C-1"),
        (8.662, "Db-1"),
        (9.177, "D-1"),
        (9.722, "Eb-1"),
        (10.300, "E-1"),
        (10.913, "F-1"),
        (11.562, "Gb-1"),
        (12.249, "G-1"),
        (12.978, "Ab-1"),
        (13.750, "A-1"),
        (14.567, "Bb-1"),
        (15.433, "B-1"),
        (16.351, "C0"),
        (17.323, "Db0"),
        (18.354, "D0"),
        (19.445, "Eb0"),
        (20.601, "E0"),
        (21.826, "F0"),
        (23.124, "Gb0"),
        (24.499, "G0"),
        (25.956, "Ab0"),
        (27.500, "A0"),
        (29.135, "Bb0"),
        (30.867, "B0"),
        (32.703, "C1"),
        (34.647, "Db1"),
        (36.708, "D1"),
        (38.890, "Eb1"),
        (41.203, "E1"),
        (43.653, "F1"),
        (46.249, "Gb1"),
        (48.999, "G1"),
        (51.913, "Ab1"),
        (55.000, "A1"),
        (58.270, "Bb1"),
        (61.735, "B1"),
        (65.406, "C2"),
        (69.295, "Db2"),
        (73.416, "D2"),
        (77.781, "Eb2"),
        (82.406, "E2"),
        (87.307, "F2"),
        (92.498, "Gb2"),
        (97.998, "G2"),
        (103.826, "Ab2"),
        (110.000, "A2"),
        (116.540, "Bb2"),
        (123.470, "B2"),
        (130.812, "C3"), // 48
        (138.591, "Db3"),
        (146.832, "D3"),
        (155.563, "Eb3"),
        (164.813, "E3"),
        (174.614, "F3"),
        (184.997, "Gb3"),
        (195.997, "G3"),
        (207.652, "Ab3"),
        (220.000, "A3"),
        (233.081, "Bb3"),
        (246.941, "B3"),
        (261.625, "C4"), // 60
        (277.182, "Db4"),
        (293.664, "D4"),
        (311.127, "Eb4"),
        (329.627, "E4"),
        (349.228, "F4"),
        (369.994, "Gb4"),
        (391.995, "G4"),
        (415.304, "Ab4"),
        (440.000, "A4"),
        (466.163, "Bb4"),
        (493.883, "B4"),
        (523.251, "C5"), // 72
        (554.365, "Db5"),
        (587.329, "D5"),
        (622.254, "Eb5"),
        (659.255, "E5"),
        (698.456, "F5"),
        (739.988, "Gb5"),
        (783.990, "G5"),
        (830.609, "Ab5"),
        (880.000, "A5"),
        (932.327, "Bb5"),
        (987.766, "B5"),
        (1046.502, "C6"), // 84
        (1_108.73, "Db6"),
        (1174.659, "D6"),
        (1244.507, "Eb6"),
        (1_318.51, "E6"),
        (1396.912, "F6"),
        (1479.977, "Gb6"),
        (1567.981, "G6"),
        (1661.218, "Ab6"),
        (1760.000, "A6"),
        (1864.655, "Bb6"),
        (1975.533, "B6"),
        (2093.004, "C7"), // 96
        (2217.461, "Db7"),
        (2349.318, "D7"),
        (2489.015, "Eb7"),
        (2_637.02, "E7"),
        (2793.825, "F7"),
        (2959.955, "Gb7"),
        (3135.963, "G7"),
        (3322.437, "Ab7"),
        (3520.000, "A7"),
        (3_729.31, "Bb7"),
        (3951.066, "B7"),
        (4186.009, "C8"),
        (4434.922, "Db8"),
        (4698.636, "D8"),
        (4978.031, "Eb8"),
        (5_274.04, "E8"),
        (5587.651, "F8"),
        (5_919.91, "Gb8"),
        (6271.927, "G8"),
        (6644.875, "Ab8"),
        (7040.000, "A8"),
        (7_458.62, "Bb8"),
        (7902.132, "B8"),
        (8372.018, "C9"),
        (8869.844, "Db9"),
        (9397.272, "D9"),
        (9956.063, "Eb9"),
        (10548.081, "E9"),
        (11175.303, "F9"),
        (11839.821, "Gb9"),
        (12543.854, "G9"),
    ];
}

/// Maximum absolute sample value before clipping.
pub const MAX_SAMPLE_VALUE: f32 = 1.0;
/// Maximum oscillator fine-tune offset in cents.
pub const OSCILLATOR_FINE_TUNE_MAX_CENTS: i8 = 63;
/// Minimum oscillator fine-tune offset in cents.
pub const OSCILLATOR_FINE_TUNE_MIN_CENTS: i8 = -63;
/// Maximum oscillator coarse-tune interval in semitones.
pub const OSCILLATOR_COURSE_TUNE_MAX_INTERVAL: i8 = 12;
/// Minimum oscillator coarse-tune interval in semitones.
pub const OSCILLATOR_COURSE_TUNE_MIN_INTERVAL: i8 = -12;
/// Maximum filter cutoff frequency in Hz.
pub const MAX_FILTER_CUTOFF: f32 = 20000.0;
/// Minimum filter cutoff frequency in Hz.
pub const MIN_FILTER_CUTOFF: f32 = 0.0;
/// Minimum filter resonance (0.0–1.0).
pub const MIN_FILTER_RESONANCE: f32 = 0.0;
/// Maximum filter resonance (0.0–1.0).
pub const MAX_FILTER_RESONANCE: f32 = 0.90;
