//! Noise fields: each one reads the `noise` crate on the plane and the fast gradient noise in space.

use noise::{NoiseFn, Perlin, Seedable};

use super::solid::gradient;

/// A noise field that can be read on the legacy plane and in space.
///
/// Both readings follow the same recipe (octaves, frequency, gain), so a sphere shows the same
/// kind of world as the plane. Only the plane reading is frozen bit for bit.
pub trait Field {
    /// Returns the value at point `p` of the legacy plane.
    fn flat(&self, p: [f64; 2]) -> f64;

    /// Returns the value at point `p` of space.
    fn solid(&self, p: [f64; 3]) -> f64;
}

impl Field for Perlin {
    fn flat(&self, p: [f64; 2]) -> f64 {
        self.get(p)
    }

    fn solid(&self, p: [f64; 3]) -> f64 {
        gradient(self.seed(), p)
    }
}
