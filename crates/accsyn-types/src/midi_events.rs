use strum_macros::FromRepr;

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum MidiEvent {
    NoteOn(u8, u8),
    NoteOff,
    ControlChange(CC),
    PitchBend(u16),
    ChannelPressure(u8),
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum CC {
    ModWheel(u8),
    VelocityCurve(u8),
    PitchBendRange(u8),
    Volume(u8),
    Mute(u8),
    Balance(u8),
    SubOscillatorShapeParameter1(u8),
    SubOscillatorShapeParameter2(u8),
    Oscillator1ShapeParameter1(u8),
    Oscillator1ShapeParameter2(u8),
    Oscillator2ShapeParameter1(u8),
    Oscillator2ShapeParameter2(u8),
    Oscillator3ShapeParameter1(u8),
    Oscillator3ShapeParameter2(u8),
    OscillatorKeySyncEnabled(u8),
    PortamentoTime(u8),
    OscillatorHardSync(u8),
    SubOscillatorShape(u8),
    Oscillator1Shape(u8),
    Oscillator2Shape(u8),
    Oscillator3Shape(u8),
    SubOscillatorCourseTune(u8),
    Oscillator1CourseTune(u8),
    Oscillator2CourseTune(u8),
    Oscillator3CourseTune(u8),
    SubOscillatorFineTune(u8),
    Oscillator1FineTune(u8),
    Oscillator2FineTune(u8),
    Oscillator3FineTune(u8),
    SubOscillatorLevel(u8),
    Oscillator1Level(u8),
    Oscillator2Level(u8),
    Oscillator3Level(u8),
    SubOscillatorMute(u8),
    Oscillator1Mute(u8),
    Oscillator2Mute(u8),
    Oscillator3Mute(u8),
    SubOscillatorBalance(u8),
    Oscillator1Balance(u8),
    Oscillator2Balance(u8),
    Oscillator3Balance(u8),
    Sustain(u8),
    PortamentoEnabled(u8),
    SubOscillatorClipBoost(u8),
    Oscillator1ClipBoost(u8),
    Oscillator2ClipBoost(u8),
    Oscillator3ClipBoost(u8),
    FilterPoles(u8),
    FilterResonance(u8),
    FilterCutoff(u8),
    AmpEGReleaseTime(u8),
    AmpEGAttackTime(u8),
    AmpEGDecayTime(u8),
    AmpEGSustainLevel(u8),
    AmpEGInverted(u8),
    FilterEnvelopeAttackTime(u8),
    FilterEnvelopeDecayTime(u8),
    FilterEnvelopeSustainLevel(u8),
    FilterEnvelopeReleaseTime(u8),
    FilterEnvelopeInverted(u8),
    FilterEnvelopeAmount(u8),
    KeyTrackingAmount(u8),
    ModWheelLFOFrequency(u8),
    ModWheelLFOCenterValue(u8),
    ModWheelLFORange(u8),
    ModWheelLFOWaveShape(u8),
    ModWheelLFOPhase(u8),
    ModWheelLFOReset,
    FilterModLFOFrequency(u8),
    FilterModLFOAmount(u8),
    FilterModLFOWaveShape(u8),
    FilterModLFOPhase(u8),
    FilterModLFOReset,
    AllNotesOff,
}

/// MIDI device update events sent between UI and MIDI module.
///
/// Note: `MidiInputPort` is a midir-specific type. To avoid pulling midir into
/// the types crate, the port is represented as an opaque index + name pair.
#[derive(Debug, Clone, PartialEq)]
pub enum MidiDeviceUpdateEvents {
    InputPortList(Vec<String>),
    InputPortByName(String),
    InputPortSelected { index: usize, port_name: String },
    InputPortCleared,
    UIMidiInputPort(String),
    UIMidiInputChannelIndex(String),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, FromRepr)]
#[repr(i32)]
pub enum MidiChannelIndex {
    Omni = 0,
}
