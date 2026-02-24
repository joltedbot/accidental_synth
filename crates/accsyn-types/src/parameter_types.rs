use std::sync::atomic::{AtomicI8, AtomicU8, AtomicU16, AtomicU32, Ordering};

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
