//! Land shape: warped continent mask, mountains, hills and plateaus.

use noise::{Fbm, MultiFractal, NoiseFn, Perlin, RidgedMulti};

/// Height of the open sea floor.
pub const SEA_FLOOR: f32 = -12.0;

/// Continent noise value above which the ground rises out of the sea.
const LAND_LEVEL: f32 = 0.08;

/// Returns 0 below `a`, 1 above `b` and a smooth blend between (also works when `a > b`).
pub fn smooth(a: f32, b: f32, x: f32) -> f32 {
    let t = ((x - a) / (b - a)).clamp(0.0, 1.0);
    t * t * (3.0 - 2.0 * t)
}

/// Derives the noise seed number `k` from the world seed.
pub fn sub_seed(seed: u64, k: u32) -> u32 {
    let h = (seed ^ (seed >> 32)) as u32;
    h.wrapping_add(k.wrapping_mul(0x9E37_79B1)) ^ k.wrapping_mul(0x85EB_CA6B)
}

/// Snaps `v` to steps of `step` with steep risers, giving terraced cliffs.
fn terrace(v: f32, step: f32) -> f32 {
    let t = v / step;
    let k = t.floor();
    (k + smooth(0.8, 1.0, t - k)) * step
}

/// The noise fields that shape the land.
#[derive(Clone)]
pub struct Shape {
    warp: Perlin,
    land: Fbm<Perlin>,
    mount: Fbm<Perlin>,
    ridge: RidgedMulti<Perlin>,
    hills: Fbm<Perlin>,
}

impl Shape {
    /// Builds the noise fields from the world seed.
    pub fn new(seed: u64) -> Self {
        Self {
            warp: Perlin::new(sub_seed(seed, 1)),
            land: Fbm::<Perlin>::new(sub_seed(seed, 2))
                .set_octaves(5)
                .set_frequency(1.0 / 520.0),
            mount: Fbm::<Perlin>::new(sub_seed(seed, 3))
                .set_octaves(2)
                .set_frequency(1.0 / 650.0),
            ridge: RidgedMulti::<Perlin>::new(sub_seed(seed, 4))
                .set_octaves(3)
                .set_frequency(1.0 / 170.0),
            hills: Fbm::<Perlin>::new(sub_seed(seed, 5))
                .set_octaves(3)
                .set_frequency(1.0 / 90.0),
        }
    }

    /// Returns the ground height at `(x, z)`: sea floor, beach, then relief inland.
    pub fn elevation(&self, x: f32, z: f32) -> f32 {
        let (px, pz) = (x as f64, z as f64);
        let wx = px + 140.0 * self.warp.get([px * 0.0035, pz * 0.0035]);
        let wz = pz + 140.0 * self.warp.get([px * 0.0035 + 31.7, pz * 0.0035 - 17.3]);
        let s = self.land.get([wx, wz]) as f32 - LAND_LEVEL;
        if s <= -0.12 {
            return SEA_FLOOR + (s + 0.12).max(-0.5) * 4.0;
        }
        if s < 0.0 {
            return SEA_FLOOR * (1.0 - smooth(-0.12, 0.0, s));
        }
        let beach = 3.0 * smooth(0.0, 0.1, s);
        if s < 0.04 {
            return beach;
        }
        beach + self.relief(wx, wz, s) * smooth(0.04, 0.3, s)
    }

    /// Returns the relief above the beach for the warped position, `s` being the inland distance measure.
    fn relief(&self, wx: f64, wz: f64, s: f32) -> f32 {
        let m = self.mount.get([wx, wz]) as f32;
        let h = (self.hills.get([wx, wz]) as f32 * 0.5 + 0.5).clamp(0.0, 1.0);
        let rolling = h * (5.0 + 16.0 * smooth(-0.1, 0.25, m));
        let plateau = terrace(h * 26.0, 7.0);
        let base = rolling + (plateau - rolling) * smooth(-0.2, -0.38, m);
        let mt = smooth(0.12, 0.38, m) * smooth(0.14, 0.4, s);
        if mt <= 0.0 {
            return base;
        }
        let r = ((self.ridge.get([wx, wz]) as f32 + 1.0) * 0.5).clamp(0.0, 1.0);
        base + mt * (26.0 + 68.0 * r * r)
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashSet;

    use crate::world::{Biome, SEA_LEVEL, Terrain};

    /// Over a 4 km square the world stays mostly sea, shows most biomes and has tall peaks.
    #[test]
    fn world_has_sea_biomes_and_peaks() {
        for seed in 1..=3 {
            let t = Terrain::new(seed);
            let pts: Vec<(f32, f32)> = (0..100)
                .flat_map(|i| {
                    (0..100).map(move |j| (i as f32 * 40.0 - 2000.0, j as f32 * 40.0 - 2000.0))
                })
                .collect();
            let land: Vec<&(f32, f32)> = pts
                .iter()
                .filter(|p| t.height(p.0, p.1) > SEA_LEVEL)
                .collect();
            let sea = 1.0 - land.len() as f32 / pts.len() as f32;
            let biomes: HashSet<Biome> = land.iter().map(|p| t.biome(p.0, p.1)).collect();
            let top = land
                .iter()
                .map(|p| t.height(p.0, p.1))
                .fold(f32::MIN, f32::max);
            assert!((0.4..0.8).contains(&sea), "seed {seed} sea {sea}");
            assert!(biomes.len() >= 4, "seed {seed} biomes {biomes:?}");
            assert!(top > 50.0, "seed {seed} top {top}");
        }
    }
}
