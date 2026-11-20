//! The flora of a chunk of a living planet, home included: the species of its producer groups, each with the
//! looks the book holds for it at every age at this time of the year, the chance it gets per try, and how
//! patchy the planet's plant cover is.
//!
//! A planet whose life was never given keeps the hand-made tables, and a dead planet gets a flora without
//! species: only its stones lie there.

use bevy::prelude::*;

use super::shelf::{kept, key};
use super::stand::GROUPS;
use crate::life::flora::{self, Kind, Stage, kinds, productivity};
use crate::life::{Habitat, Life, Species, Tolerance, World};
use crate::universe::hash::mix;
use crate::world::Terrain;
use crate::world::canopy::stage_at;
use crate::world::globe::{Chart, Globe};
use crate::world::lands::thrive::climate_on;
use crate::world::propmesh::BIG;
use crate::world::propmesh::grown::{AGES, Habit, Look, habit, page};

/// Chance per try of a group of each architecture on its best ground, before productivity.
const CHANCE: [f32; 8] = [0.35, 0.12, 0.5, 0.12, 0.12, 0.15, 0.3, 0.12];

/// Least share of the plant tries of home a living planet gets, whatever its class.
const LIVING: f32 = 0.6;

/// Latitudes in radians the mean wind of a planet is read at.
const BREEZES: [f64; 3] = [0.26, 0.79, 1.22];

/// Salt of the planet seed its patchiness is drawn from.
const PATCHY: u64 = 0x5041_5443_4859;

/// One species of a producer group as the scatter plants it.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Grower {
    /// Growth architecture.
    pub arch: u8,
    /// Where it lives.
    pub habitat: Habitat,
    /// Climate it stands.
    pub tolerance: Tolerance,
    /// Chance per try of its whole group on its best ground.
    pub chance: f32,
    /// Index of its group among the producers of the planet, below [`GROUPS`].
    pub group: usize,
    /// Share of the plants of its group that are of this species.
    pub share: f32,
    /// Typical height in metres of a grown plant.
    pub height: f32,
    /// Carrier and reference height of its architecture.
    pub habit: Habit,
    /// Where in the patch noise of its group it grows best, 0 to 1.
    pub niche: f32,
    /// Placement variant of its look at each age.
    pub looks: [u8; AGES],
}

impl Grower {
    /// Returns true when the species is planted by the pass of the big props.
    pub fn big(&self) -> bool {
        (self.habit.carrier as usize) < BIG
    }
}

/// The plant species of one chunk, none on a dead planet, and how patchy its cover is.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct Flora {
    /// Every species of every producer group.
    pub growers: Vec<Grower>,
    /// How patchy the plant cover of the planet is, 0 (lush, nearly unbroken) to 1 (scattered life).
    pub patchiness: f32,
}

/// Returns the share of the plant tries of home chunks of `terrain` get: what the class of the planet
/// lets grow, and at least [`LIVING`] on a living planet, whose life fits its class.
pub fn growth(terrain: &Terrain) -> f32 {
    let globe = terrain.globe();
    let living = globe.lives() == Some(true) && terrain.chart() != Chart::Flat;
    let class = globe.lands().growth();
    if living { class.max(LIVING) } else { class }
}

/// Returns the patchiness of the planet of seed `seed`, 0 to 1, the same on every visit.
pub fn patchiness(seed: u64) -> f32 {
    (mix(seed ^ PATCHY) >> 40) as f32 / (1u64 << 24) as f32
}

/// Returns the world of `globe` as the plant rules read it. The globe does not know its star, whose colour
/// reaches the plants through the pigment of their life, so the star temperature is left at 0.
fn world_of(globe: &Globe) -> World {
    let record = globe.record();
    World {
        gravity: globe.gravity() as f32,
        pressure: record.pressure as f32,
        temperature: record.temperature as f32,
        flux: record.flux as f32,
        radius: globe.radius() as f32,
        ocean: record.ocean as f32,
        star: 0.0,
        tilt: record.tilt as f32,
        liquids: record.liquids().map(|share| share as f32),
    }
}

/// Returns the mean wind of `globe` in metres per second, read at three latitudes of the annual mean.
fn breeze(globe: &Globe) -> f32 {
    let speed = |lat: f64| {
        climate_on(globe, bevy::math::DVec3::new(lat.cos(), lat.sin(), 0.0))
            .wind
            .length()
    };
    BREEZES.iter().map(|&lat| speed(lat)).sum::<f32>() / BREEZES.len() as f32
}

/// Returns species `kind` of producer `species`, group number `group` of `life`, on a `world` with its wind
/// at `stage`, with its looks at every age written into the book.
fn grower(
    life: &Life,
    species: &Species,
    air: (&World, f32),
    stage: Stage,
    (group, kind): (usize, Kind),
) -> Grower {
    let dials = flora::of_kind(life, species, air, stage, kind);
    let arch = dials.architecture;
    let rich = productivity(life, species, air.0);
    Grower {
        arch,
        habitat: species.role.habitat,
        tolerance: species.tolerance,
        chance: CHANCE[usize::from(arch)] * (0.3 + 0.7 * rich),
        group: group.min(GROUPS - 1),
        share: kind.share,
        height: dials.form.height,
        habit: habit(arch),
        niche: kind.niche,
        looks: std::array::from_fn(|age| {
            page(Look {
                dials,
                variant: age as u8,
            })
        }),
    }
}

/// Returns the flora of `life` on `globe` at `stage`: every species of every producer group, the chance of
/// each shared among the groups of its architecture.
fn sow(life: &Life, globe: &Globe, stage: Stage) -> Flora {
    let air = (&world_of(globe), breeze(globe));
    let mut growers: Vec<Grower> = flora::producers(life)
        .enumerate()
        .flat_map(|(group, s)| {
            let rich = productivity(life, s, air.0);
            kinds(s, rich)
                .into_iter()
                .map(move |kind| grower(life, s, air, stage, (group, kind)))
        })
        .collect();
    let groups = |arch: u8| {
        let same = flora::producers(life).filter(|s| flora::architecture(life, s) == arch);
        same.count().max(1) as f32
    };
    growers.iter_mut().for_each(|g| g.chance /= groups(g.arch));
    Flora {
        growers,
        patchiness: patchiness(globe.seed()),
    }
}

impl Flora {
    /// Returns the flora of the chunk centred on chart position `center` of `terrain`, or nothing where the
    /// hand-made tables apply: on the legacy plane and where the life of the planet was never given. The
    /// flora of a planet at one stage of its year is sown once and kept.
    pub fn of(terrain: &Terrain, center: Vec2) -> Option<Self> {
        let globe = terrain.globe();
        let lives = globe.lives().filter(|_| terrain.chart() != Chart::Flat)?;
        let Some(life) = globe.life().filter(|_| lives) else {
            return Some(Self::default());
        };
        let stage = stage_at(terrain, center);
        Some(kept(key(life, globe, stage), || sow(life, globe, stage)))
    }
}
