//! A cheap hasher for the per-thread cell lookups: the keys are a few small integers, read on every ground
//! sample, so a multiply per word beats the default hasher by far and needs no defence against attack.

use std::hash::{BuildHasherDefault, Hasher};

/// The hasher state: every word written is folded in with one multiply.
#[derive(Default)]
pub struct Mix(u64);

/// Builds [`Mix`] hashers.
pub type BuildMix = BuildHasherDefault<Mix>;

impl Hasher for Mix {
    fn write(&mut self, bytes: &[u8]) {
        bytes.iter().for_each(|&b| self.write_u64(u64::from(b)));
    }

    fn write_u64(&mut self, word: u64) {
        self.0 = (self.0 ^ word)
            .wrapping_mul(0x9E37_79B9_7F4A_7C15)
            .rotate_left(23);
    }

    fn write_u32(&mut self, word: u32) {
        self.write_u64(u64::from(word));
    }

    fn write_i32(&mut self, word: i32) {
        self.write_u64(u64::from(word as u32));
    }

    fn write_isize(&mut self, word: isize) {
        self.write_u64(word as u64);
    }

    fn finish(&self) -> u64 {
        self.0
    }
}
