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
pub const SYSTEM_PATCH_INIT_PARAMETERS: &str = SYSTEM_PATCHES[0].1;
pub const SYSTEM_PATCHES: &[(&str, &str)] = &[
    ("Init*", include_str!("patches/init.json")),
    (
        "A Slightly Warmer FM*",
        include_str!("patches/a-slightly-warmer-fm.json"),
    ),
    ("Acid Squelch*", include_str!("patches/acid-squelch.json")),
    ("Acid Time*", include_str!("patches/acid-time.json")),
    (
        "Alien Invasion*",
        include_str!("patches/alien-invasion.json"),
    ),
    ("Ambient Drone*", include_str!("patches/ambient-drone.json")),
    (
        "Arpable Dirty FM*",
        include_str!("patches/arpable-dirty-fm.json"),
    ),
    ("Bright Lead*", include_str!("patches/bright-lead.json")),
    ("Buzz Brass*", include_str!("patches/buzz-brass.json")),
    ("Deep Bass*", include_str!("patches/deep-bass.json")),
    (
        "Dirty Bass Echo*",
        include_str!("patches/dirty-bass-echo.json"),
    ),
    ("Drifting Pad*", include_str!("patches/drifting-pad.json")),
    (
        "Electric Piano*",
        include_str!("patches/electric-piano.json"),
    ),
    ("FM Bells*", include_str!("patches/fm-bells.json")),
    ("Guys I Thing I Broke It*", include_str!("patches/guys-I-think-I-Broke-It.json")),
    ("House Hits*", include_str!("patches/house-hits.json")),
    ("Kick (Long)*", include_str!("patches/kick-long.json")),
    ("Kick (Short)*", include_str!("patches/kick-short.json")),
    ("Plucky Bass*", include_str!("patches/plucky-bass.json")),
    ("Plucky Keys*", include_str!("patches/plucky-keys.json")),
    ("Sci-Fi*", include_str!("patches/sci-fi.json")),
    (
        "Reverse Bass Swells*",
        include_str!("patches/reverse-bass-swells.json"),
    ),
    (
        "Reverse Pulse Lead*",
        include_str!("patches/reverse-pulse-lead.json"),
    ),
    ("Shred Auto-Arp*", include_str!("patches/shred-auto-arp.json")),
    ("Singing Bowls*", include_str!("patches/singing-bowls.json")),
    ("Slide Bass*", include_str!("patches/slide-bass.json")),
    (
        "Supersaw 5ths Repeater*",
        include_str!("patches/supersaw-5ths-repeater.json"),
    ),
    (
        "Supersaw Swirl*",
        include_str!("patches/supersaw-swirl.json"),
    ),
    (
        "Triangles and Claves*",
        include_str!("patches/triangles-and-claves.json"),
    ),
    (
        "Wandering Saws*",
        include_str!("patches/wandering-saws.json"),
    ),
];
