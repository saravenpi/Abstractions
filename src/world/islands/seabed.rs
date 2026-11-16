//! The sea floor: sandy shelf with bars, reef bumps, rolling deep floor, seamounts and canyons.

use noise::{Fbm, MultiFractal, NoiseFn, Perlin};

use super::{smooth, sub_seed};

/// Mean height of the deep sea floor.
const DEEP: f32 = -15.0;

/// Depth of the shelf edge where the floor starts to drop.
const SHELF_DEPTH: f32 = -5.0;

/// The noise fields of the sea floor.
#[derive(Clone)]
pub struct Seabed {
    rolling: Fbm<Perlin>,
    dune: Perlin,
    mount: Perlin,
    canyon: Perlin,
    patch: Perlin,
    bump: Perlin,
}

impl Seabed {
    /// Builds the sea floor fields from the world seed.
    pub fn new(seed: u64) -> Self {
        Self {
            rolling: Fbm::<Perlin>::new(sub_seed(seed, 12))
                .set_octaves(2)
                .set_frequency(1.0 / 260.0),
            dune: Perlin::new(sub_seed(seed, 13)),
            mount: Perlin::new(sub_seed(seed, 14)),
            canyon: Perlin::new(sub_seed(seed, 15)),
            patch: Perlin::new(sub_seed(seed, 16)),
            bump: Perlin::new(sub_seed(seed, 17)),
        }
    }

    /// Returns the floor height `d` away from the coast measure, `cliff` steepening the shelf and
    /// `warmth` (temperature noise) allowing reefs in warm water.
    pub fn floor(&self, p: [f64; 2], d: f32, cliff: f32, warmth: f32) -> f32 {
        let width = 0.15 - 0.1 * cliff;
        let top = smooth(0.0, width, d);
        let ripple = self.dune.get([p[0] / 26.0, p[1] / 26.0]) as f32 * 0.5 * smooth(0.01, 0.05, d);
        let shelf = SHELF_DEPTH * top + ripple * (1.0 - top * top);
        let deep = DEEP
            + 3.5 * self.rolling.get(p) as f32
            + 0.9 * self.dune.get([p[0] / 17.0 + 40.0, p[1] / 17.0]) as f32;
        let mut h = shelf + (deep - shelf) * smooth(width, width + 0.22, d);
        let open = smooth(0.3, 0.6, d);
        if open > 0.0 {
            let q = self.canyon.get([p[0] / 420.0, p[1] / 420.0]).abs() as f32;
            let peak = smooth(
                0.3,
                0.6,
                self.mount.get([p[0] / 320.0, p[1] / 320.0]) as f32,
            );
            h += open * (-26.0 * smooth(0.1, 0.0, q) + (-1.5 - deep) * peak);
        }
        if top > 0.2 && warmth > 0.0 {
            h = (h + self.reef(p, warmth)).min(-0.9);
        }
        h
    }

    /// Returns the coral bump height added to the shelf in warm water.
    fn reef(&self, p: [f64; 2], warmth: f32) -> f32 {
        let patch = smooth(0.0, 0.25, self.patch.get([p[0] / 70.0, p[1] / 70.0]) as f32)
            * smooth(0.0, 0.2, warmth);
        if patch <= 0.0 {
            return 0.0;
        }
        let k = 1.0 - self.bump.get([p[0] / 11.0, p[1] / 11.0]).abs() as f32;
        patch * 2.2 * k * k
    }
}
