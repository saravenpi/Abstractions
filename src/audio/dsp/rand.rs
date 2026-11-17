//! Deterministic random numbers: a seeded generator for composing and white noise for sound.

/// A seeded generator for the choices that shape a piece. The same seed gives the same sequence.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Rand(u64);

impl Rand {
    /// Builds the generator of stream `stream` for a seed.
    pub fn new(seed: u64, stream: u64) -> Self {
        Self(seed ^ stream.wrapping_mul(0xD6E8_FEB8_6659_FD93))
    }

    /// Returns the next 64 random bits.
    pub fn bits(&mut self) -> u64 {
        self.0 = self.0.wrapping_add(0x9E37_79B9_7F4A_7C15);
        let mut z = self.0;
        z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
        z ^ (z >> 31)
    }

    /// Returns the next number in `[0, 1)`.
    pub fn unit(&mut self) -> f32 {
        (self.bits() >> 40) as f32 / 16_777_216.0
    }

    /// Returns the next number in `[low, high)`.
    pub fn range(&mut self, low: f32, high: f32) -> f32 {
        low + (high - low) * self.unit()
    }

    /// Returns the next index below `count`, which must not be zero.
    pub fn below(&mut self, count: usize) -> usize {
        (self.bits() % count as u64) as usize
    }
}

/// White noise for the sample loop.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Noise(u32);

impl Noise {
    /// Builds a noise source. Different seeds give uncorrelated noise.
    pub fn new(seed: u32) -> Self {
        Self(seed | 1)
    }

    /// Returns the next sample in `[-1, 1)`.
    pub fn tick(&mut self) -> f32 {
        let mut x = self.0;
        x ^= x << 13;
        x ^= x >> 17;
        x ^= x << 5;
        self.0 = x;
        (x >> 8) as f32 / 8_388_608.0 - 1.0
    }
}
