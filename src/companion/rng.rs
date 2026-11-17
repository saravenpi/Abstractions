//! A tiny seeded random number generator, so every choice of the companion can be replayed in a test.

/// SplitMix64: the same seed always gives the same sequence.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Rng(u64);

impl Rng {
    /// Returns a generator for `seed`.
    pub fn new(seed: u64) -> Self {
        Self(seed)
    }

    /// Returns the next number in `[0, 1)`.
    pub fn next(&mut self) -> f32 {
        self.0 = self.0.wrapping_add(0x9E37_79B9_7F4A_7C15);
        let mut x = self.0;
        x = (x ^ (x >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
        x = (x ^ (x >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
        x ^= x >> 31;
        (x >> 40) as f32 / 16_777_216.0
    }

    /// Returns the next number in `[low, high)`.
    pub fn range(&mut self, low: f32, high: f32) -> f32 {
        low + (high - low) * self.next()
    }
}
