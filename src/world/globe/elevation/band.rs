//! The regional band: noise between the plates and the local ground, from waves 400 km long down to 5 km,
//! the long waves that move the shore off the plate edges, and the crest lines that break a range into
//! ridges and valleys.

use bevy::math::DVec3;

use super::tune::{
    BAND_FINEST, BAND_HEIGHT, BAND_HURST, BAND_LONGEST, BAND_SIZE, BAND_STEP, COAST, OCTAVES,
    RANGE_BODY, RANGE_CRESTS,
};
use crate::world::globe::plates::RANGE;
use crate::world::islands::{gradient, sub_seed};

/// The regional noise of one planet.
#[derive(Debug, Clone, PartialEq)]
pub struct Band {
    /// Seed of the first octave.
    seed: u32,
    /// One over the length of each wave in metres, and its height in metres.
    waves: [(f64, f32); OCTAVES],
    /// How many waves the planet has room for.
    count: usize,
    /// One over the metres between two crest lines of a range.
    crests: f64,
}

impl Band {
    /// Returns the band of planet `seed` of `radius` metres: waves [`BAND_STEP`] times shorter each, from the longest the
    /// planet has room for down to [`BAND_FINEST`], each as high as real ground of that length is rough.
    pub fn new(seed: u64, radius: f64) -> Self {
        let longest = BAND_LONGEST.0.min(BAND_LONGEST.1 * radius);
        let wave = |k: usize| {
            let length = longest / BAND_STEP.powi(k as i32);
            let height = BAND_HEIGHT * (length / BAND_SIZE).powf(BAND_HURST);
            (1.0 / length, height as f32)
        };
        let waves: [(f64, f32); OCTAVES] = std::array::from_fn(wave);
        Self {
            seed: sub_seed(seed, 31),
            count: waves.iter().filter(|w| w.0 <= 1.0 / BAND_FINEST).count(),
            waves,
            crests: RANGE_CRESTS / (RANGE * radius),
        }
    }

    /// Returns the height of the band in metres at sphere point `p`.
    pub fn at(&self, p: DVec3) -> f32 {
        let wave = |(k, (rate, height)): (usize, &(f64, f32))| {
            height * gradient(self.seed.wrapping_add(k as u32), (p * *rate).to_array()) as f32
        };
        self.waves[..self.count].iter().enumerate().map(wave).sum()
    }

    /// Returns what moves the shore in unit direction `dir`, in units of the crust blend: two long waves
    /// read by direction, so they keep their angular size on every planet as the plates do.
    pub fn coast(&self, dir: DVec3) -> f32 {
        let wave = |k: u32, (rate, height): (f64, f32)| {
            height * gradient(self.seed.wrapping_add(k), (dir * rate).to_array()) as f32
        };
        wave(50, COAST[0]) + wave(51, COAST[1])
    }

    /// Returns the share of its height a range has at sphere point `p`: 1 on a crest line, [`RANGE_BODY`]
    /// in the valleys between, from crest lines of two sizes.
    pub fn ridge(&self, p: DVec3) -> f32 {
        let crest = |k: u32, rate: f64| {
            let line =
                1.0 - gradient(self.seed.wrapping_add(k), (p * rate).to_array()).abs() as f32;
            line * line
        };
        let lines = 0.7 * crest(40, self.crests) + 0.3 * crest(41, 2.3 * self.crests);
        RANGE_BODY + (1.0 - RANGE_BODY) * lines
    }
}
