//! Craters: seeded bowls with raised rims over the whole of a world without air, on the sphere only.
//!
//! Each atlas cell holds a count of craters by its area, placed and sized by seeded rolls; the pit maps
//! of the atlas sort them into bins, so a ground sample reads only the craters that can reach it.

use bevy::math::DVec3;

use super::Globe;
use super::atlas::{Cell, STEPS};
use super::roll::{roll, rolls};
use crate::world::islands::smooth;

/// Craters per square metre of ground: ten per square kilometre.
const DENSITY: f64 = 1e-5;

/// Smallest and widest crater, rim to rim, in metres.
const SIZES: (f32, f32) = (16.0, 160.0);

/// Depth of the floor from the rim crest, as a share of the width.
pub const DEPTH: f32 = 0.2;

/// Height of the rim above the ground around, as a share of the width.
const RIM: f32 = 0.04;

/// Distance from the centre, in crater radii, where the rim has fallen back to the ground.
pub const FOOT: f32 = 1.6;

/// One crater: its centre on the sea level sphere, in metres from the planet centre, and its width.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Crater {
    /// Centre of the bowl.
    pub centre: DVec3,
    /// Width rim to rim in metres.
    pub size: f32,
}

impl Crater {
    /// Returns the height this crater adds at planet point `p`.
    pub fn depth(&self, p: DVec3) -> f32 {
        profile(
            (p - self.centre).length() as f32 / (0.5 * self.size),
            self.size,
        )
    }
}

/// Returns the craters of `cell` of `globe`: as many as its area holds on average, smaller ones far more
/// often than wide ones.
pub fn craters(globe: &Globe, cell: Cell) -> Vec<Crater> {
    let n = globe.cells();
    let key = [cell.face as i64, i64::from(cell.i), i64::from(cell.j)];
    let [wide, tall] = cell.size(n, globe.radius());
    let count = (DENSITY * wide * tall + f64::from(roll(globe.seed(), key, 0))) as u32;
    (1..=count)
        .map(|k| {
            let [x, y, span, _] = rolls(globe.seed(), key, k);
            let local = [x, y].map(|c| f64::from(c) * STEPS as f64);
            Crater {
                centre: cell.dir(local, n) * globe.radius(),
                size: SIZES.0 * (SIZES.1 / SIZES.0).powf(span * span),
            }
        })
        .collect()
}

/// Returns the height a crater `size` metres wide adds `r` of its radii from its centre: a bowl from the
/// floor, `DEPTH` times the width under the rim crest, up to the crest, then the rim falling to the ground.
pub fn profile(r: f32, size: f32) -> f32 {
    if r >= FOOT {
        0.0
    } else if r < 1.0 {
        (RIM - DEPTH) * size + DEPTH * size * r * r
    } else {
        RIM * size * (1.0 - smooth(1.0, FOOT, r))
    }
}
