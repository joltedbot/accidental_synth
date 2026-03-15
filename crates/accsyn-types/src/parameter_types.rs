use std::sync::atomic::{AtomicI8, AtomicI16, AtomicU8, AtomicU16, AtomicU32, Ordering};
use serde::{Deserialize, Deserializer, Serialize, Serializer};

#[derive(Debug)]
pub struct NormalizedValue {
    value: AtomicU32,
}

impl NormalizedValue {
    #[inline]
    pub fn new(normalized: f32) -> Self {
        Self {
            value: AtomicU32::new(normalized.to_bits()),
        }
    }

    #[inline]
    pub fn load(&self) -> f32 {
        f32::from_bits(self.value.load(Ordering::Relaxed))
    }

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

#[derive(Debug)]
pub struct Hertz {
    value: AtomicU32,
}

impl Hertz {
    #[inline]
    pub fn new(hz: f32) -> Self {
        Self {
            value: AtomicU32::new(hz.to_bits()),
        }
    }

    #[inline]
    pub fn load(&self) -> f32 {
        f32::from_bits(self.value.load(Ordering::Relaxed))
    }

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

#[derive(Debug)]
pub struct LfoRange {
    value: AtomicU32,
}

impl LfoRange {
    #[inline]
    pub fn new(range: f32) -> Self {
        Self {
            value: AtomicU32::new(range.to_bits()),
        }
    }

    #[inline]
    pub fn load(&self) -> f32 {
        f32::from_bits(self.value.load(Ordering::Relaxed))
    }

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

#[derive(Debug)]
pub struct Milliseconds {
    value: AtomicU32,
}

impl Milliseconds {
    #[inline]
    pub fn new(ms: u32) -> Self {
        Self {
            value: AtomicU32::new(ms),
        }
    }

    #[inline]
    pub fn load(&self) -> u32 {
        self.value.load(Ordering::Relaxed)
    }

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

#[derive(Debug)]
pub struct Cents {
    value: AtomicI8,
}

impl Cents {
    #[inline]
    pub fn new(cents: i8) -> Self {
        Self {
            value: AtomicI8::new(cents),
        }
    }

    #[inline]
    pub fn load(&self) -> i8 {
        self.value.load(Ordering::Relaxed)
    }

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

#[derive(Debug)]
pub struct Semitones {
    value: AtomicI8,
}

impl Semitones {
    #[inline]
    pub fn new(semitones: i8) -> Self {
        Self {
            value: AtomicI8::new(semitones),
        }
    }

    #[inline]
    pub fn load(&self) -> i8 {
        self.value.load(Ordering::Relaxed)
    }

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

#[derive(Debug)]
pub struct PitchBend {
    value: AtomicI16,
}

impl PitchBend {
    #[inline]
    pub fn new(bend: i16) -> Self {
        Self {
            value: AtomicI16::new(bend),
        }
    }

    #[inline]
    pub fn load(&self) -> i16 {
        self.value.load(Ordering::Relaxed)
    }

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

#[derive(Debug)]
pub struct FilterPoles {
    value: AtomicU8,
}

impl FilterPoles {
    #[inline]
    pub fn new(poles: u8) -> Self {
        Self {
            value: AtomicU8::new(poles),
        }
    }

    #[inline]
    pub fn load(&self) -> u8 {
        self.value.load(Ordering::Relaxed)
    }

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

#[derive(Debug)]
pub struct Balance {
    value: AtomicU32,
}

impl Balance {
    #[inline]
    pub fn new(balance: f32) -> Self {
        Self {
            value: AtomicU32::new(balance.to_bits()),
        }
    }

    #[inline]
    pub fn load(&self) -> f32 {
        f32::from_bits(self.value.load(Ordering::Relaxed))
    }

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

#[derive(Debug)]
pub struct PortamentoBuffers {
    value: AtomicU16,
}

impl PortamentoBuffers {
    #[inline]
    pub fn new(buffers: u16) -> Self {
        Self {
            value: AtomicU16::new(buffers),
        }
    }

    #[inline]
    pub fn load(&self) -> u16 {
        self.value.load(Ordering::Relaxed)
    }

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
