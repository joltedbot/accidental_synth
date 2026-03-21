/// Final gain stage with velocity sensitivity.
pub mod amplifier;
/// Audio effects processing chain (wavefolder, clipper, delay, etc.).
pub mod effects;
/// ADSR envelope generators for amplitude and filter modulation.
pub mod envelope;
/// Resonant lowpass ladder filter with key tracking and envelope modulation.
pub mod filter;
/// Low-frequency oscillators for parameter modulation.
pub mod lfo;
/// Level and stereo balance mixing for oscillator and output stages.
pub mod mixer;
/// Waveform generation oscillators supporting multiple wave shapes.
pub mod oscillator;
