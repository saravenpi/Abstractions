//! Marks: what sits on the plates without following their edges. Volcanoes over hot spots where the
//! interior is still hot, and impact basins where nothing has renewed the surface.

use std::f64::consts::TAU;

use bevy::math::DVec3;

use super::interior::Interior;
use super::tune::{BASIN, BASIN_DEPTH, BASINS, CONE, CONE_DRIFT, CONE_HEIGHT, CONES};
use crate::world::globe::craters::{DEPTH, FOOT, profile};
use crate::world::globe::roll::rolls;

/// The grid cell the rolls of the marks are drawn at.
const KEY: [i64; 3] = [0x3A2C, 0, 0];

/// One round mark on the sphere.
#[derive(Debug, Clone, Copy, PartialEq)]
struct Mark {
    /// Unit direction of the centre.
    centre: DVec3,
    /// Radians from the centre to the foot of a volcano, or to the rim of a basin.
    reach: f64,
    /// Cosine of the angle past which the mark adds nothing.
    edge: f64,
    /// Height of a volcano as a share of the highest mountains, or the width in metres of the bowl that
    /// has the depth of the basin.
    size: f32,
}

/// The volcanoes and the impact basins of one planet.
#[derive(Debug, Clone, PartialEq)]
pub struct Marks {
    /// Volcanoes over hot spots.
    cones: Vec<Mark>,
    /// Impact basins.
    basins: Vec<Mark>,
}

impl Marks {
    /// Returns the marks of planet `seed` of `radius` metres with `interior`: as many volcanoes as its heat
    /// feeds and as many basins as its surface has kept.
    pub fn new(seed: u64, radius: f64, interior: &Interior) -> Self {
        let cones = (CONES * interior.activity).round() as u32;
        let basins = (BASINS * interior.scars).round() as u32;
        Self {
            cones: (0..cones).map(|k| cone(seed, k, interior)).collect(),
            basins: (0..basins).map(|k| basin(seed, k, radius)).collect(),
        }
    }

    /// Returns what the marks add in unit direction `dir`: the height of the volcanoes as a share of the
    /// highest mountains, and the height of the basins in metres, negative in a bowl.
    pub fn at(&self, dir: DVec3) -> (f32, f32) {
        let within = |m: &Mark| {
            let dot = m.centre.dot(dir);
            (dot > m.edge).then(|| (dot.min(1.0).acos() / m.reach) as f32)
        };
        let slope = |m: &Mark| within(m).map(|r| m.size * (1.0 - r) * (1.0 - r));
        let bowl = |m: &Mark| within(m).map(|r| profile(r, m.size));
        (
            self.cones.iter().filter_map(slope).sum(),
            self.basins.iter().filter_map(bowl).sum(),
        )
    }
}

/// Returns the seeded unit direction of mark `slot` of planet `seed`, and two more rolls for its shape.
fn place(seed: u64, slot: u32) -> (DVec3, f64, f64) {
    let [u, v, a, b] = rolls(seed, KEY, slot).map(f64::from);
    let (y, angle) = (2.0 * u - 1.0, TAU * v);
    let ring = (1.0 - y * y).sqrt();
    (DVec3::new(ring * angle.cos(), y, ring * angle.sin()), a, b)
}

/// Returns volcano `k` of planet `seed`: a shield with straight flanks steepening to the summit, lower on
/// a planet whose plates carry it off its hot spot.
fn cone(seed: u64, k: u32, interior: &Interior) -> Mark {
    let (centre, wide, tall) = place(seed, k);
    let reach = CONE.0 + (CONE.1 - CONE.0) * wide;
    let height = CONE_HEIGHT.0 + (CONE_HEIGHT.1 - CONE_HEIGHT.0) * tall as f32;
    Mark {
        centre,
        reach,
        edge: reach.cos(),
        size: height * (1.0 - CONE_DRIFT * interior.mobility),
    }
}

/// Returns impact basin `k` of planet `seed` of `radius` metres: small ones far more often than wide ones,
/// a fifth as deep as wide until the depth of wide craters, which grows much slower, takes over.
fn basin(seed: u64, k: u32, radius: f64) -> Mark {
    let (centre, span, _) = place(seed, 1000 + k);
    let reach = BASIN.0 * (BASIN.1 / BASIN.0).powf(span * span);
    let width = 2.0 * reach * radius;
    let depth =
        (BASIN_DEPTH.0 * (width / 1000.0).powf(BASIN_DEPTH.1)).min(f64::from(DEPTH) * width);
    Mark {
        centre,
        reach,
        edge: (reach * f64::from(FOOT)).cos(),
        size: (depth / f64::from(DEPTH)) as f32,
    }
}
