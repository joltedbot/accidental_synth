use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::sync::atomic::{AtomicI8, AtomicI16, AtomicU8, AtomicU16, AtomicU32, Ordering};

/// Thread-safe normalized float parameter (0.0–1.0) stored as atomic bits.
#[derive(Debug)]
pub struct NormalizedValue {
    value: AtomicU32,
}

impl NormalizedValue {
    /// Creates a new normalized value from an f32.
    #[inline]
    pub fn new(normalized: f32) -> Self {
        Self {
            value: AtomicU32::new(normalized.to_bits()),
        }
    }

    /// Loads the current value as an f32.
    #[inline]
    pub fn load(&self) -> f32 {
        f32::from_bits(self.value.load(Ordering::Relaxed))
    }

    /// Stores a new f32 value.
    #[inline]
    pub fn store(&self, normalized: f32) {
        self.value.store(normalized.to_bits(), Ordering::Relaxed);
    }
}

impl Default for NormalizedValue {
    fn default() -> Self {
        Self {
            value: AtomicU32::new(0.0_f32.to_bits()),
        }
    }
}

impl Serialize for NormalizedValue {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_f32(self.load())
    }
}

impl<'de> Deserialize<'de> for NormalizedValue {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let value = f32::deserialize(deserializer)?;
        Ok(Self::new(value))
    }
}

/// Thread-safe frequency parameter in Hz stored as atomic bits.
#[derive(Debug)]
pub struct Hertz {
    value: AtomicU32,
}

impl Hertz {
    /// Creates a new frequency value from Hz.
    #[inline]
    pub fn new(hz: f32) -> Self {
        Self {
            value: AtomicU32::new(hz.to_bits()),
        }
    }

    /// Loads the current frequency in Hz.
    #[inline]
    pub fn load(&self) -> f32 {
        f32::from_bits(self.value.load(Ordering::Relaxed))
    }

    /// Stores a new frequency in Hz.
    #[inline]
    pub fn store(&self, hz: f32) {
        self.value.store(hz.to_bits(), Ordering::Relaxed);
    }
}

impl Default for Hertz {
    fn default() -> Self {
        Self {
            value: AtomicU32::new(0.0_f32.to_bits()),
        }
    }
}

impl Serialize for Hertz {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_f32(self.load())
    }
}

impl<'de> Deserialize<'de> for Hertz {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let value = f32::deserialize(deserializer)?;
        Ok(Self::new(value))
    }
}

/// Thread-safe LFO modulation range parameter stored as atomic bits.
#[derive(Debug)]
pub struct LfoRange {
    value: AtomicU32,
}

impl LfoRange {
    /// Creates a new LFO range value.
    #[inline]
    pub fn new(range: f32) -> Self {
        Self {
            value: AtomicU32::new(range.to_bits()),
        }
    }

    /// Loads the current LFO range.
    #[inline]
    pub fn load(&self) -> f32 {
        f32::from_bits(self.value.load(Ordering::Relaxed))
    }

    /// Stores a new LFO range.
    #[inline]
    pub fn store(&self, range: f32) {
        self.value.store(range.to_bits(), Ordering::Relaxed);
    }
}

impl Default for LfoRange {
    fn default() -> Self {
        Self {
            value: AtomicU32::new(0.0_f32.to_bits()),
        }
    }
}

impl Serialize for LfoRange {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_f32(self.load())
    }
}

impl<'de> Deserialize<'de> for LfoRange {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let range = f32::deserialize(deserializer)?;
        Ok(Self::new(range))
    }
}

/// Thread-safe time duration parameter in milliseconds stored atomically.
#[derive(Debug)]
pub struct Milliseconds {
    value: AtomicU32,
}

impl Milliseconds {
    /// Creates a new milliseconds value.
    #[inline]
    pub fn new(ms: u32) -> Self {
        Self {
            value: AtomicU32::new(ms),
        }
    }

    /// Loads the current value in milliseconds.
    #[inline]
    pub fn load(&self) -> u32 {
        self.value.load(Ordering::Relaxed)
    }

    /// Stores a new value in milliseconds.
    #[inline]
    pub fn store(&self, ms: u32) {
        self.value.store(ms, Ordering::Relaxed);
    }
}

impl Default for Milliseconds {
    fn default() -> Self {
        Self {
            value: AtomicU32::new(0),
        }
    }
}

impl Serialize for Milliseconds {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_u32(self.load())
    }
}

impl<'de> Deserialize<'de> for Milliseconds {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let ms = u32::deserialize(deserializer)?;
        Ok(Self::new(ms))
    }
}

/// Thread-safe fine-tune offset parameter in cents stored atomically.
#[derive(Debug)]
pub struct Cents {
    value: AtomicI8,
}

impl Cents {
    /// Creates a new cents value.
    #[inline]
    pub fn new(cents: i8) -> Self {
        Self {
            value: AtomicI8::new(cents),
        }
    }

    /// Loads the current value in cents.
    #[inline]
    pub fn load(&self) -> i8 {
        self.value.load(Ordering::Relaxed)
    }

    /// Stores a new value in cents.
    #[inline]
    pub fn store(&self, cents: i8) {
        self.value.store(cents, Ordering::Relaxed);
    }
}

impl Default for Cents {
    fn default() -> Self {
        Self {
            value: AtomicI8::new(0),
        }
    }
}

impl Serialize for Cents {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_i8(self.load())
    }
}

impl<'de> Deserialize<'de> for Cents {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let cents = i8::deserialize(deserializer)?;
        Ok(Self::new(cents))
    }
}

/// Thread-safe coarse-tune offset parameter in semitones stored atomically.
#[derive(Debug)]
pub struct Semitones {
    value: AtomicI8,
}

impl Semitones {
    /// Creates a new semitones value.
    #[inline]
    pub fn new(semitones: i8) -> Self {
        Self {
            value: AtomicI8::new(semitones),
        }
    }

    /// Loads the current value in semitones.
    #[inline]
    pub fn load(&self) -> i8 {
        self.value.load(Ordering::Relaxed)
    }

    /// Stores a new value in semitones.
    #[inline]
    pub fn store(&self, semitones: i8) {
        self.value.store(semitones, Ordering::Relaxed);
    }
}

impl Default for Semitones {
    fn default() -> Self {
        Self {
            value: AtomicI8::new(0),
        }
    }
}

impl Serialize for Semitones {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_i8(self.load())
    }
}

impl<'de> Deserialize<'de> for Semitones {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let semitones = i8::deserialize(deserializer)?;
        Ok(Self::new(semitones))
    }
}

/// Thread-safe pitch bend parameter stored as a signed 14-bit MIDI value.
#[derive(Debug)]
pub struct PitchBend {
    value: AtomicI16,
}

impl PitchBend {
    /// Creates a new pitch bend value.
    #[inline]
    pub fn new(bend: i16) -> Self {
        Self {
            value: AtomicI16::new(bend),
        }
    }

    /// Loads the current pitch bend value.
    #[inline]
    pub fn load(&self) -> i16 {
        self.value.load(Ordering::Relaxed)
    }

    /// Stores a new pitch bend value.
    #[inline]
    pub fn store(&self, bend: i16) {
        self.value.store(bend, Ordering::Relaxed);
    }
}

impl Default for PitchBend {
    fn default() -> Self {
        Self {
            value: AtomicI16::new(0),
        }
    }
}

impl Serialize for PitchBend {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_i16(self.load())
    }
}

impl<'de> Deserialize<'de> for PitchBend {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let bend = i16::deserialize(deserializer)?;
        Ok(Self::new(bend))
    }
}

/// Thread-safe filter pole count parameter (e.g., 2 for 12dB/oct, 4 for 24dB/oct).
#[derive(Debug)]
pub struct FilterPoles {
    value: AtomicU8,
}

impl FilterPoles {
    /// Creates a new filter poles value.
    #[inline]
    pub fn new(poles: u8) -> Self {
        Self {
            value: AtomicU8::new(poles),
        }
    }

    /// Loads the current pole count.
    #[inline]
    pub fn load(&self) -> u8 {
        self.value.load(Ordering::Relaxed)
    }

    /// Stores a new pole count.
    #[inline]
    pub fn store(&self, poles: u8) {
        self.value.store(poles, Ordering::Relaxed);
    }
}

impl Default for FilterPoles {
    fn default() -> Self {
        Self {
            value: AtomicU8::new(4),
        }
    }
}

impl Serialize for FilterPoles {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_u8(self.load())
    }
}

impl<'de> Deserialize<'de> for FilterPoles {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let poles = u8::deserialize(deserializer)?;
        Ok(Self::new(poles))
    }
}

/// Thread-safe stereo balance parameter (-1.0 left to 1.0 right) stored as atomic bits.
#[derive(Debug)]
pub struct Balance {
    value: AtomicU32,
}

impl Balance {
    /// Creates a new balance value.
    #[inline]
    pub fn new(balance: f32) -> Self {
        Self {
            value: AtomicU32::new(balance.to_bits()),
        }
    }

    /// Loads the current balance value.
    #[inline]
    pub fn load(&self) -> f32 {
        f32::from_bits(self.value.load(Ordering::Relaxed))
    }

    /// Stores a new balance value.
    #[inline]
    pub fn store(&self, balance: f32) {
        self.value.store(balance.to_bits(), Ordering::Relaxed);
    }
}

impl Default for Balance {
    fn default() -> Self {
        Self {
            value: AtomicU32::new(0.0_f32.to_bits()),
        }
    }
}

impl Serialize for Balance {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_f32(self.load())
    }
}

impl<'de> Deserialize<'de> for Balance {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let value = f32::deserialize(deserializer)?;
        Ok(Self::new(value))
    }
}

/// Thread-safe portamento buffer count parameter stored atomically.
#[derive(Debug)]
pub struct PortamentoBuffers {
    value: AtomicU16,
}

impl PortamentoBuffers {
    /// Creates a new portamento buffers value.
    #[inline]
    pub fn new(buffers: u16) -> Self {
        Self {
            value: AtomicU16::new(buffers),
        }
    }

    /// Loads the current buffer count.
    #[inline]
    pub fn load(&self) -> u16 {
        self.value.load(Ordering::Relaxed)
    }

    /// Stores a new buffer count.
    #[inline]
    pub fn store(&self, buffers: u16) {
        self.value.store(buffers, Ordering::Relaxed);
    }
}

impl Default for PortamentoBuffers {
    fn default() -> Self {
        Self {
            value: AtomicU16::new(0),
        }
    }
}

impl Serialize for PortamentoBuffers {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_u16(self.load())
    }
}

impl<'de> Deserialize<'de> for PortamentoBuffers {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let buffers = u16::deserialize(deserializer)?;
        Ok(Self::new(buffers))
    }
}
