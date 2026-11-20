//! Galaxies: one per universe grid cell, with about 60 percent of the cells holding one.

use bevy::math::IVec3;

use super::hash::{self, Rng};
use super::home::HOME_GALAXY;
use super::ids::GalaxyId;
use super::spec::{GalaxyKind, GalaxySpec};
use super::{GALAXY_CELL, LY, SECTOR_M, names};

/// Share of the grid cells that hold a galaxy.
const FILL: f64 = 0.6;

/// Smallest and largest galaxy radius in light-years: the Milky Way disc is 87,400 light-years
/// across, so 25,000 to 75,000 covers dwarf to giant spirals
/// (<https://en.wikipedia.org/wiki/Milky_Way>).
pub const RADIUS_LY: (f64, f64) = (25_000.0, 75_000.0);

/// Nearest and farthest a galaxy centre lies from the minimum corner of its cell, as a share of the cell.
pub const CENTER_SPAN: (f64, f64) = (0.2, 0.8);

/// Fewest and most arms of a spiral galaxy.
pub const ARMS: (u8, u8) = (2, 5);

/// Child seed number of the universe seed that every galaxy seed derives from.
const SALT: u64 = 0x6A1A;

/// Child seed numbers of a galaxy seed for its presence, shape, centre and name.
const PARTS: [u64; 4] = [0, 1, 2, 3];

/// The numbers that shape the star density of one galaxy, all derived from the galaxy seed.
#[derive(Debug, Clone, Copy, PartialEq)]
pub(super) struct Shape {
    /// Seed of the galaxy, the parent of every sector seed.
    pub seed: u64,
    /// Overall shape.
    pub kind: GalaxyKind,
    /// Number of arms, 0 unless the galaxy is a spiral.
    pub arms: u8,
    /// Radius of the star disc in light-years.
    pub radius: f64,
    /// Deviation of the dense core as a share of the radius, from 0.05 to 0.1.
    pub core: f64,
    /// Deviation of the disc thickness as a share of the radius, from 0.008 to 0.015: 400 to 750
    /// light-years at 50,000, for a thin disc of 700 to 1,500 light-years.
    pub thick: f64,
    /// Angle in radians the arms have turned by at the rim, negative for the other winding.
    pub swirl: f64,
}

impl Shape {
    /// Derives the shape of a galaxy from its seed. The home galaxy is always a spiral.
    pub fn new(seed: u64, is_home: bool) -> Self {
        let mut rng = Rng::new(hash::child(seed, PARTS[1]));
        let kind = if is_home {
            GalaxyKind::Spiral
        } else {
            kind(rng.below(100))
        };
        let spiral = kind == GalaxyKind::Spiral;
        let arms = ARMS.0 + rng.below(u32::from(ARMS.1 - ARMS.0) + 1) as u8;
        let radius = rng.range(RADIUS_LY.0, RADIUS_LY.1);
        let winding = if rng.below(2) == 0 { 1.0 } else { -1.0 };
        Self {
            seed,
            kind,
            arms: if spiral { arms } else { 0 },
            radius,
            core: rng.range(0.05, 0.1),
            thick: rng.range(0.008, 0.015),
            swirl: winding * rng.range(3.0, 7.0),
        }
    }
}

/// Returns the kind of a galaxy for a roll below 100: most are spirals, few are irregular.
fn kind(roll: u32) -> GalaxyKind {
    match roll {
        0..60 => GalaxyKind::Spiral,
        60..85 => GalaxyKind::Elliptical,
        _ => GalaxyKind::Irregular,
    }
}

/// Returns the seed of the galaxy of a grid cell, or nothing when the cell is empty.
fn seed(universe: u64, id: GalaxyId) -> Option<u64> {
    let seed = hash::cell(hash::child(universe, SALT), id.0);
    let present = id == HOME_GALAXY || Rng::new(hash::child(seed, PARTS[0])).unit() < FILL;
    present.then_some(seed)
}

/// Returns the shape of the galaxy of a grid cell, or nothing when the cell is empty.
pub(super) fn shape(universe: u64, id: GalaxyId) -> Option<Shape> {
    seed(universe, id).map(|seed| Shape::new(seed, id == HOME_GALAXY))
}

/// Returns the galaxy of a grid cell, or nothing when the cell is empty.
pub(super) fn spec(universe: u64, id: GalaxyId) -> Option<GalaxySpec> {
    let shape = shape(universe, id)?;
    let mut rng = Rng::new(hash::child(shape.seed, PARTS[2]));
    let sectors = (GALAXY_CELL / SECTOR_M).round();
    let mut axis = || (rng.range(CENTER_SPAN.0, CENTER_SPAN.1) * sectors).round() as i32;
    let center_sector = IVec3::new(axis(), axis(), axis());
    Some(GalaxySpec {
        id,
        seed: shape.seed,
        name: names::galaxy(hash::child(shape.seed, PARTS[3]), id == HOME_GALAXY),
        kind: shape.kind,
        arms: shape.arms,
        radius: shape.radius * LY,
        center_sector,
        center: center_sector.as_dvec3() * SECTOR_M,
    })
}
