use midir::Ignore;
pub const PANIC_MESSAGE_MIDI_SENDER_FAILURE: &str =
    "Could not send MIDI message to the synthesizer engine.";
pub const PANIC_MESSAGE_PORT_LIST_SENDER_FAILURE: &str =
    "Could not send MIDI ports list to the midi engine.";
pub const MIDI_INPUT_CLIENT_NAME: &str = "Accidental Synth MIDI Input";
pub const DEFAULT_MIDI_PORT_INDEX: usize = 0;
pub const MIDI_INPUT_CONNECTION_NAME: &str = "Accidental Synth MIDI Input Connection";
pub const MIDI_STATUS_BYTE_INDEX: usize = 0;
pub const MIDI_NOTE_NUMBER_BYTE_INDEX: usize = 1;
pub const MIDI_VELOCITY_BYTE_INDEX: usize = 2;
pub const MIDI_CC_NUMBER_BYTE_INDEX: usize = 1;
pub const MIDI_CC_VALUE_BYTE_INDEX: usize = 2;
pub const MIDI_CHANNEL_PRESSURE_VALUE_BYTE_INDEX: usize = 1;
pub const MIDI_PITCH_BEND_MSB_BYTE_INDEX: usize = 2;
pub const MIDI_PITCH_BEND_LSB_BYTE_INDEX: usize = 1;
pub const MIDI_MESSAGE_SENDER_CAPACITY: usize = 16;
pub const STATUS_BYTE_CHANNEL_MASK: u8 = 0x0F;
pub const STATUS_BYTE_MESSAGE_TYPE_MASK: u8 = 0xF0;
pub const RAW_CHANNEL_TO_USER_READABLE_CHANNEL_OFFSET: u8 = 1;
pub const DEFAULT_NAME_FOR_UNNAMED_MIDI_PORTS: &str = "Name Not Available";
pub const MIDI_MESSAGE_IGNORE_LIST: Ignore = Ignore::SysexAndTime;
pub const DEVICE_LIST_POLLING_INTERVAL: u64 = 2000;
