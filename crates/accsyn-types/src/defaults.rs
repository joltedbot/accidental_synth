pub struct Defaults {}

impl Defaults {
    // Audio Defaults
    pub const AUDIO_DEVICE_INDEX: i32 = 0;
    pub const LEFT_CHANNEL_INDEX: i32 = 0;
    pub const RIGHT_CHANNEL_INDEX: i32 = 1;
    pub const MONO_CHANNEL_COUNT: u16 = 1;
    pub const OUTPUT_CHANNEL_DISABLED_VALUE: i32 = -1;
    pub const SAMPLE_RATE_INDEX: usize = 1;
    pub const SUPPORTED_SAMPLE_RATES: [u32; 3] = [44100, 48000, 96000];
    pub const BUFFER_SIZE_INDEX: usize = 2;
    pub const SUPPORTED_BUFFER_SIZES: [u32; 5] = [64, 128, 256, 512, 1024];

    // Mixer Defaults
    pub const OUTPUT_MIXER_LEVEL: f32 = 0.5;
    pub const OUTPUT_MIXER_BALANCE: f32 = 0.0;
    pub const OUTPUT_MIXER_IS_MUTED: bool = false;
    pub const QUAD_MIXER_LEVEL: f32 = 1.0;
    pub const QUAD_MIXER_SUB_LEVEL: f32 = 0.0;
    pub const QUAD_MIXER_BALANCE: f32 = 0.0;
    pub const QUAD_MIXER_IS_MUTED: bool = false;

    // Global Panel Constants
    pub const PITCH_BEND_RANGE: u8 = 12;
    pub const PORTAMENTO_TIME_NORMAL_VALUE: f32 = 0.0;
    pub const VELOCITY_CURVE_NORMAL_VALUE: f32 = 0.5;
    pub const MINIMUM_PITCH_BEND_RANGE: u32 = 2;
    pub const MAXIMUM_PITCH_BEND_RANGE: u32 = 12;
}

// Effects Defaults
pub const DELAY_DEFAULT_PARAMETERS: [f32; 4] = [0.5, 0.5, 0.5, 0.0];
pub const AUTOPAN_DEFAULT_PARAMETERS: [f32; 4] = [0.1, 1.0, 0.0, 0.0];
pub const TREMOLO_DEFAULT_PARAMETERS: [f32; 4] = [0.1, 1.0, 0.0, 0.0];
pub const COMPRESSOR_DEFAULT_PARAMETERS: [f32; 4] = [0.0, 1.0, 0.0, 0.0];
pub const CLIPPER_DEFAULT_PARAMETERS: [f32; 4] = [1.0, 0.0, 0.0, 0.0];
pub const GATE_DEFAULT_PARAMETERS: [f32; 4] = [0.0, 1.0, 0.0, 0.0];

impl Defaults {
    pub const MIDI_NOTE_FREQUENCIES: [(f32, &str); 128] = [
        (8.175, "C-1"),
        (8.662, "C#(Db)-1"),
        (9.177, "D-1"),
        (9.722, "D#(Eb)-1"),
        (10.300, "E-1"),
        (10.913, "F-1"),
        (11.562, "F#(Gb)-1"),
        (12.249, "G-1"),
        (12.978, "G#(Ab)-1"),
        (13.750, "A-1"),
        (14.567, "A#(Bb)-1"),
        (15.433, "B-1"),
        (16.351, "C0"),
        (17.323, "C#(Db)0"),
        (18.354, "D0"),
        (19.445, "D#(Eb)0"),
        (20.601, "E0"),
        (21.826, "F0"),
        (23.124, "F#(Gb)0"),
        (24.499, "G0"),
        (25.956, "G#(Ab)0"),
        (27.500, "A0"),
        (29.135, "A#(Bb)0"),
        (30.867, "B0"),
        (32.703, "C1"),
        (34.647, "C#(Db)1"),
        (36.708, "D1"),
        (38.890, "D#(Eb)1"),
        (41.203, "E1"),
        (43.653, "F1"),
        (46.249, "F#(Gb)1"),
        (48.999, "G1"),
        (51.913, "G#(Ab)1"),
        (55.000, "A1"),
        (58.270, "A#(Bb)1"),
        (61.735, "B1"),
        (65.406, "C2"),
        (69.295, "C#(Db)2"),
        (73.416, "D2"),
        (77.781, "D#(Eb)2"),
        (82.406, "E2"),
        (87.307, "F2"),
        (92.498, "F#(Gb)2"),
        (97.998, "G2"),
        (103.826, "G#(Ab)2"),
        (110.000, "A2"),
        (116.540, "A#(Bb)2"),
        (123.470, "B2"),
        (130.812, "C3"), // 48
        (138.591, "C#(Db)3"),
        (146.832, "D3"),
        (155.563, "D#(Eb)3"),
        (164.813, "E3"),
        (174.614, "F3"),
        (184.997, "F#(Gb)3"),
        (195.997, "G3"),
        (207.652, "G#(Ab)3"),
        (220.000, "A3"),
        (233.081, "A#(Bb)3"),
        (246.941, "B3"),
        (261.625, "C4"), // 60
        (277.182, "C#(Db)4"),
        (293.664, "D4"),
        (311.127, "D#(Eb)4"),
        (329.627, "E4"),
        (349.228, "F4"),
        (369.994, "F#(Gb)4"),
        (391.995, "G4"),
        (415.304, "G#(Ab)4"),
        (440.000, "A4"),
        (466.163, "A#(Bb)4"),
        (493.883, "B4"),
        (523.251, "C5"), // 72
        (554.365, "C#(Db)5"),
        (587.329, "D5"),
        (622.254, "D#(Eb)5"),
        (659.255, "E5"),
        (698.456, "F5"),
        (739.988, "F#(Gb)5"),
        (783.990, "G5"),
        (830.609, "G#(Ab)5"),
        (880.000, "A5"),
        (932.327, "A#(Bb)5"),
        (987.766, "B5"),
        (1046.502, "C6"), // 84
        (1_108.73, "C#(Db)6"),
        (1174.659, "D6"),
        (1244.507, "D#(Eb)6"),
        (1_318.51, "E6"),
        (1396.912, "F6"),
        (1479.977, "F#(Gb)6"),
        (1567.981, "G6"),
        (1661.218, "G#(Ab)6"),
        (1760.000, "A6"),
        (1864.655, "A#(Bb)6"),
        (1975.533, "B6"),
        (2093.004, "C7"), // 96
        (2217.461, "C#(Db)7"),
        (2349.318, "D7"),
        (2489.015, "D#(Eb)7"),
        (2_637.02, "E7"),
        (2793.825, "F7"),
        (2959.955, "F#(Gb)7"),
        (3135.963, "G7"),
        (3322.437, "G#(Ab)7"),
        (3520.000, "A7"),
        (3_729.31, "A#(Bb)7"),
        (3951.066, "B7"),
        (4186.009, "C8"),
        (4434.922, "C#(Db)8"),
        (4698.636, "D8"),
        (4978.031, "D#(Eb)8"),
        (5_274.04, "E8"),
        (5587.651, "F8"),
        (5_919.91, "F#(Gb)8"),
        (6271.927, "G8"),
        (6644.875, "G#(Ab)8"),
        (7040.000, "A8"),
        (7_458.62, "A#(Bb)8"),
        (7902.132, "B8"),
        (8372.018, "C9"),
        (8869.844, "C#(Db)9"),
        (9397.272, "D9"),
        (9956.063, "D#(Eb)9"),
        (10548.081, "E9"),
        (11175.303, "F9"),
        (11839.821, "F#(Gb)9"),
        (12543.854, "G9"),
    ];
}

pub const MAX_SAMPLE_VALUE: f32 = 1.0;
