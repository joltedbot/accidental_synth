use midir::Ignore;

/// Default MIDI input port index when no port is explicitly selected.
pub const DEFAULT_MIDI_PORT_INDEX: usize = 0;
/// Byte index of the CC number within a Control Change message.
pub const CC_MESSAGE_NUMBER_BYTE_INDEX: usize = 1;
/// Byte index of the CC value within a Control Change message.
pub const CC_MESSAGE_VALUE_BYTE_INDEX: usize = 2;
/// Byte index of the pressure value within a Channel Pressure message.
pub const CHANNEL_PRESSURE_VALUE_BYTE_INDEX: usize = 1;
/// Polling interval in milliseconds for checking MIDI device list changes.
pub const DEVICE_LIST_POLLING_INTERVAL: u64 = 2000;
/// MIDI message types to ignore (SysEx and timing messages).
pub const MESSAGE_TYPE_IGNORE_LIST: Ignore = Ignore::SysexAndTime;
/// Bitmask to extract the MIDI channel from a status byte.
pub const MESSAGE_STATUS_BYTE_CHANNEL_MASK: u8 = 0x0F;
/// Byte index of the status byte within a MIDI message.
pub const MESSAGE_STATUS_BYTE_INDEX: usize = 0;
/// Bitmask to extract the message type from a status byte.
pub const MESSAGE_STATUS_BYTE_TYPE_MASK: u8 = 0xF0;
/// Client name used when creating midir MIDI input instances.
pub const MIDI_INPUT_CLIENT_NAME: &str = "Accidental Synth MIDI Input";
/// Connection name used when connecting to a MIDI input port.
pub const MIDI_INPUT_CONNECTION_NAME: &str = "AccSyn MIDI Input";
/// Bounded channel capacity for MIDI message and device update senders.
pub const MIDI_MESSAGE_SENDER_CAPACITY: usize = 16;
/// Byte index of the note number within a Note On/Off message.
pub const NOTE_MESSAGE_NUMBER_BYTE_INDEX: usize = 1;
/// Byte index of the velocity within a Note On/Off message.
pub const NOTE_MESSAGE_VELOCITY_BYTE_INDEX: usize = 2;
/// Byte index of the most significant byte in a Pitch Bend message.
pub const PITCH_BEND_MESSAGE_MSB_BYTE_INDEX: usize = 2;
/// Byte index of the least significant byte in a Pitch Bend message.
pub const PITCH_BEND_MESSAGE_LSB_BYTE_INDEX: usize = 1;
/// Offset to convert zero-indexed MIDI channels to user-readable (1-based) channels.
pub const RAW_CHANNEL_TO_USER_READABLE_CHANNEL_OFFSET: u8 = 1;
/// Fallback name displayed when a MIDI port name cannot be determined.
pub const UNKNOWN_MIDI_PORT_NAME_MESSAGE: &str = "Unknown";
