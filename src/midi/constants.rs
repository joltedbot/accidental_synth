use midir::Ignore;

pub const DEFAULT_MIDI_PORT_INDEX: usize = 0;
pub const CC_MESSAGE_NUMBER_BYTE_INDEX: usize = 1;
pub const CC_MESSAGE_VALUE_BYTE_INDEX: usize = 2;
pub const CHANNEL_PRESSURE_VALUE_BYTE_INDEX: usize = 1;
pub const DEVICE_LIST_POLLING_INTERVAL: u64 = 2000;
pub const INPUT_PORT_SENDER_CAPACITY: usize = 5;
pub const MESSAGE_TYPE_IGNORE_LIST: Ignore = Ignore::SysexAndTime;
pub const MESSAGE_STATUS_BYTE_CHANNEL_MASK: u8 = 0x0F;
pub const MESSAGE_STATUS_BYTE_INDEX: usize = 0;
pub const MESSAGE_STATUS_BYTE_TYPE_MASK: u8 = 0xF0;
pub const MIDI_INPUT_CLIENT_NAME: &str = "Accidental Synth MIDI Input";
pub const MIDI_INPUT_CONNECTION_NAME: &str = "Accidental Synth MIDI Input Connection";
pub const MIDI_MESSAGE_SENDER_CAPACITY: usize = 16;
pub const NOTE_MESSAGE_NUMBER_BYTE_INDEX: usize = 1;
pub const NOTE_MESSAGE_VELOCITY_BYTE_INDEX: usize = 2;
pub const PANIC_MESSAGE_MIDI_SENDER_FAILURE: &str =
    "Could not send MIDI message to the synthesizer engine.";
pub const PITCH_BEND_MESSAGE_MSB_BYTE_INDEX: usize = 2;
pub const PITCH_BEND_MESSAGE_LSB_BYTE_INDEX: usize = 1;
pub const RAW_CHANNEL_TO_USER_READABLE_CHANNEL_OFFSET: u8 = 1;
pub const UNKNOWN_MIDI_PORT_NAME_MESSAGE: &str = "Unknown";

