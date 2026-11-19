//! Deterministic hashing: every seed derives from a parent seed and an index or a grid cell.

use bevy::math::IVec3;

/// Scrambles 64 bits with the splitmix64 finalizer.
pub fn mix(x: u64) -> u64 {
    let x = (x ^ (x >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
    let x = (x ^ (x >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
    x ^ (x >> 31)
}

/// Returns the seed of child number `index` of a parent seed.
pub fn child(seed: u64, index: u64) -> u64 {
    mix(seed ^ mix(index.wrapping_add(0x9E37_79B9_7F4A_7C15)))
}

/// Returns the seed of a grid cell under a parent seed.
pub fn cell(seed: u64, at: IVec3) -> u64 {
    let axis = |h: u64, v: i32| child(h, u64::from(v as u32));
    axis(axis(axis(seed, at.x), at.y), at.z)
}

/// A splitmix64 sequence: a stream of numbers that depends only on its seed.
#[derive(Debug, Clone, Copy)]
pub struct Rng(u64);

impl Rng {
    /// Starts the sequence of a seed.
    pub fn new(seed: u64) -> Self {
        Self(seed)
    }

    /// Returns the next 64 bits.
    #[allow(clippy::should_implement_trait)]
    pub fn next(&mut self) -> u64 {
        self.0 = self.0.wrapping_add(0x9E37_79B9_7F4A_7C15);
        mix(self.0)
    }

    /// Returns the next number from 0 included to 1 excluded.
    pub fn unit(&mut self) -> f64 {
        (self.next() >> 11) as f64 / (1u64 << 53) as f64
    }

    /// Returns the next number from `low` included to `high` excluded.
    pub fn range(&mut self, low: f64, high: f64) -> f64 {
        low + (high - low) * self.unit()
    }

    /// Returns the next whole number below `n`, which must not be 0.
    pub fn below(&mut self, n: u32) -> u32 {
        (((self.next() >> 32) * u64::from(n)) >> 32) as u32
    }
}
