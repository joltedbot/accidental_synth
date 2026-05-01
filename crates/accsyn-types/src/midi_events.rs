use strum_macros::FromRepr;

/// MIDI events received from input devices and forwarded to the synthesizer.
#[derive(Debug, PartialEq, Clone, Copy)]
pub enum MidiEvent {
    /// Note on event (note number, velocity).
    NoteOn(u8, u8),
    /// Note off event.
    NoteOff,
    /// Control change message.
    ControlChange(CC),
    /// Pitch bend event (14-bit unsigned value, center at 8192).
    PitchBend(u16),
    /// Channel pressure (aftertouch) event (pressure value).
    ChannelPressure(u8),
    /// Program change event (program number).
    ProgramChange(u8),
}

/// MIDI Control Change message types mapped to synthesizer parameters.
#[derive(Debug, PartialEq, Clone, Copy)]
pub enum CC {
    /// Mod wheel position (CC 1).
    ModWheel(u8),
    /// Velocity curve adjustment.
    VelocityCurve(u8),
    /// Pitch bend range adjustment.
    PitchBendRange(u8),
    /// Master volume (CC 7).
    Volume(u8),
    /// Master mute toggle.
    Mute(u8),
    /// Master stereo balance.
    Balance(u8),
    /// Sub-oscillator waveform parameter 1.
    SubOscillatorShapeParameter1(u8),
    /// Sub-oscillator waveform parameter 2.
    SubOscillatorShapeParameter2(u8),
    /// Oscillator 1 waveform parameter 1.
    Oscillator1ShapeParameter1(u8),
    /// Oscillator 1 waveform parameter 2.
    Oscillator1ShapeParameter2(u8),
    /// Oscillator 2 waveform parameter 1.
    Oscillator2ShapeParameter1(u8),
    /// Oscillator 2 waveform parameter 2.
    Oscillator2ShapeParameter2(u8),
    /// Oscillator 3 waveform parameter 1.
    Oscillator3ShapeParameter1(u8),
    /// Oscillator 3 waveform parameter 2.
    Oscillator3ShapeParameter2(u8),
    /// Toggle oscillator key sync.
    OscillatorKeySyncEnabled(u8),
    /// Sub-oscillator pitch envelope amount.
    SubOscillatorPitchEnvelopeAmount(u8),
    /// Oscillator 1 pitch envelope amount.
    Oscillator1PitchEnvelopeAmount(u8),
    /// Oscillator 2 pitch envelope amount.
    Oscillator2PitchEnvelopeAmount(u8),
    /// Oscillator 3 pitch envelope amount.
    Oscillator3PitchEnvelopeAmount(u8),
    /// Pitch envelope attack time
    PitchEnvelopeAttackTime(u8),
    /// Pitch envelope decay time
    PitchEnvelopeDecayTime(u8),
    /// Pitch envelope sustain level
    PitchEnvelopeSustainLevel(u8),
    /// Pitch envelope release time
    PitchEnvelopeReleaseTime(u8),
    /// Pitch envelope inversion toggle
    PitchEnvelopeInverted(u8),
    /// Portamento glide time.
    PortamentoTime(u8),
    /// Toggle oscillator hard sync.
    OscillatorHardSync(u8),
    /// Sub-oscillator waveform shape selection.
    SubOscillatorShape(u8),
    /// Oscillator 1 waveform shape selection.
    Oscillator1Shape(u8),
    /// Oscillator 2 waveform shape selection.
    Oscillator2Shape(u8),
    /// Oscillator 3 waveform shape selection.
    Oscillator3Shape(u8),
    /// Sub-oscillator coarse tune.
    SubOscillatorCourseTune(u8),
    /// Oscillator 1 coarse tune.
    Oscillator1CourseTune(u8),
    /// Oscillator 2 coarse tune.
    Oscillator2CourseTune(u8),
    /// Oscillator 3 coarse tune.
    Oscillator3CourseTune(u8),
    /// Sub-oscillator fine tune.
    SubOscillatorFineTune(u8),
    /// Oscillator 1 fine tune.
    Oscillator1FineTune(u8),
    /// Oscillator 2 fine tune.
    Oscillator2FineTune(u8),
    /// Oscillator 3 fine tune.
    Oscillator3FineTune(u8),
    /// Sub-oscillator mixer level.
    SubOscillatorLevel(u8),
    /// Oscillator 1 mixer level.
    Oscillator1Level(u8),
    /// Oscillator 2 mixer level.
    Oscillator2Level(u8),
    /// Oscillator 3 mixer level.
    Oscillator3Level(u8),
    /// Sub-oscillator mute toggle.
    SubOscillatorMute(u8),
    /// Oscillator 1 mute toggle.
    Oscillator1Mute(u8),
    /// Oscillator 2 mute toggle.
    Oscillator2Mute(u8),
    /// Oscillator 3 mute toggle.
    Oscillator3Mute(u8),
    /// Sub-oscillator stereo balance.
    SubOscillatorBalance(u8),
    /// Oscillator 1 stereo balance.
    Oscillator1Balance(u8),
    /// Oscillator 2 stereo balance.
    Oscillator2Balance(u8),
    /// Oscillator 3 stereo balance.
    Oscillator3Balance(u8),
    /// Sustain pedal (CC 64).
    Sustain(u8),
    /// Toggle portamento on/off.
    PortamentoEnabled(u8),
    /// Sub-oscillator clipper boost amount.
    SubOscillatorClipBoost(u8),
    /// Oscillator 1 clipper boost amount.
    Oscillator1ClipBoost(u8),
    /// Oscillator 2 clipper boost amount.
    Oscillator2ClipBoost(u8),
    /// Oscillator 3 clipper boost amount.
    Oscillator3ClipBoost(u8),
    /// Filter pole count selection.
    FilterPoles(u8),
    /// Filter resonance amount.
    FilterResonance(u8),
    /// Filter cutoff frequency.
    FilterCutoff(u8),
    /// Amplitude envelope release time.
    AmpEGReleaseTime(u8),
    /// Amplitude envelope attack time.
    AmpEGAttackTime(u8),
    /// Amplitude envelope decay time.
    AmpEGDecayTime(u8),
    /// Amplitude envelope sustain level.
    AmpEGSustainLevel(u8),
    /// Amplitude envelope inversion toggle.
    AmpEGInverted(u8),
    /// Filter envelope attack time.
    FilterEnvelopeAttackTime(u8),
    /// Filter envelope decay time.
    FilterEnvelopeDecayTime(u8),
    /// Filter envelope sustain level.
    FilterEnvelopeSustainLevel(u8),
    /// Filter envelope release time.
    FilterEnvelopeReleaseTime(u8),
    /// Filter envelope inversion toggle.
    FilterEnvelopeInverted(u8),
    /// Filter envelope modulation amount.
    FilterEnvelopeAmount(u8),
    /// Filter key tracking amount.
    KeyTrackingAmount(u8),
    /// Mod wheel LFO frequency.
    ModWheelLFOFrequency(u8),
    /// Mod wheel LFO center value.
    ModWheelLFOCenterValue(u8),
    /// Mod wheel LFO range.
    ModWheelLFORange(u8),
    /// Mod wheel LFO waveform shape.
    ModWheelLFOWaveShape(u8),
    /// Mod wheel LFO phase offset.
    ModWheelLFOPhase(u8),
    /// Reset mod wheel LFO phase to zero.
    ModWheelLFOReset,
    /// Filter modulation LFO frequency.
    FilterModLFOFrequency(u8),
    /// Filter modulation LFO amount.
    FilterModLFOAmount(u8),
    /// Filter modulation LFO waveform shape.
    FilterModLFOWaveShape(u8),
    /// Filter modulation LFO phase offset.
    FilterModLFOPhase(u8),
    /// Reset filter modulation LFO phase to zero.
    FilterModLFOReset,
    /// Turn off all currently sounding notes.
    AllNotesOff,
}

/// MIDI device update events sent between UI and MIDI module.
///
/// Note: `MidiInputPort` is a midir-specific type. To avoid pulling midir into
/// the types crate, the port is represented as an opaque index + name pair.
#[derive(Debug, Clone, PartialEq)]
pub enum MidiDeviceUpdateEvents {
    /// Updated list of available MIDI input port names.
    InputPortList(Vec<String>),
    /// Request to connect to a MIDI input port by name.
    InputPortByName(String),
    /// A MIDI input port was selected (index and port name).
    InputPortSelected {
        /// Index of the selected port in the port list.
        index: usize,
        /// Name of the selected port.
        port_name: String,
    },
    /// The current MIDI input port was disconnected.
    InputPortCleared,
    /// User selected a MIDI input port from the UI.
    UIMidiInputPort(String),
    /// User changed the MIDI input channel filter from the UI.
    UIMidiInputChannelIndex(String),
}

/// MIDI channel filter index.
#[derive(Debug, Clone, Copy, PartialEq, Eq, FromRepr)]
#[repr(i32)]
pub enum MidiChannelIndex {
    /// Receive on all MIDI channels.
    Omni = 0,
}
