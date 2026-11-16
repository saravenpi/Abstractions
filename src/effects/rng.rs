//! A tiny deterministic hash random number generator for particles.

use bevy::prelude::*;

/// Counter based random numbers: the same sequence on every run.
#[derive(Resource, Debug, Clone, Copy)]
pub struct Rng(u32);

impl Default for Rng {
    /// Starts from a fixed seed.
    fn default() -> Self {
        Self(0x1234_5678)
    }
}

impl Rng {
    /// Returns the next number in `[0, 1)`.
    pub fn next(&mut self) -> f32 {
        self.0 = self.0.wrapping_add(0x9E37_79B9);
        let mut x = self.0;
        x ^= x >> 16;
        x = x.wrapping_mul(0x7feb_352d);
        x ^= x >> 15;
        x = x.wrapping_mul(0x846c_a68b);
        x ^= x >> 16;
        (x >> 8) as f32 / 16_777_216.0
    }

    /// Returns the next number in `[low, high)`.
    pub fn range(&mut self, low: f32, high: f32) -> f32 {
        low + (high - low) * self.next()
    }
}
