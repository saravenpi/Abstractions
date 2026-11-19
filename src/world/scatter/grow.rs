//! The flora of a chunk of a living planet, home included: its producer groups, each with the looks the
//! book holds for it at this time of the year, and the chance it gets per try.
//!
//! A planet whose life was never given keeps the hand-made tables, and a dead planet gets a flora without
//! groups: only its stones lie there.

use bevy::prelude::*;

use crate::life::flora::{self, Stage, productivity};
use crate::life::{Habitat, Life, Species, Tolerance, World};
use crate::world::Terrain;
use crate::world::globe::{Chart, Globe};
use crate::world::lands::thrive::climate_on;
use crate::world::propmesh::grown::{Habit, Look, habit, page};
use crate::world::propmesh::{BIG, VARIANTS};

/// Chance per try of a group of each architecture on its best ground, before productivity.
const CHANCE: [f32; 8] = [0.35, 0.12, 0.5, 0.12, 0.12, 0.15, 0.3, 0.12];

/// Least share of the plant tries of home a living planet gets, whatever its class.
const LIVING: f32 = 0.6;

/// Latitudes in radians the mean wind of a planet is read at.
const BREEZES: [f64; 3] = [0.26, 0.79, 1.22];

/// One producer group as the scatter plants it.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Grower {
    /// Growth architecture.
    pub arch: u8,
    /// Where it lives.
    pub habitat: Habitat,
    /// Climate it stands.
    pub tolerance: Tolerance,
    /// Chance per try on its best ground.
    pub chance: f32,
    /// Typical height in metres.
    pub height: f32,
    /// Canopy layers, which spread the sizes it grows at.
    pub layers: u8,
    /// Carrier and reference height of its architecture.
    pub habit: Habit,
    /// Placement variant of each of its shape variants.
    pub looks: [u8; VARIANTS],
}

impl Grower {
    /// Returns true when the group is planted by the pass of the big props.
    pub fn big(&self) -> bool {
        (self.habit.carrier as usize) < BIG
    }
}

/// The plant groups of one chunk; none on a dead planet.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct Flora {
    /// Every producer group.
    pub growers: Vec<Grower>,
}

/// Returns the share of the plant tries of home chunks of `terrain` get: what the class of the planet
/// lets grow, and at least [`LIVING`] on a living planet, whose life fits its class.
pub fn growth(terrain: &Terrain) -> f32 {
    let globe = terrain.globe();
    let living = globe.lives() == Some(true) && terrain.chart() != Chart::Flat;
    let class = globe.lands().growth();
    if living { class.max(LIVING) } else { class }
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

/// Returns producer `species` of `life` on `world` in a wind of `wind` metres per second at `stage`, with
/// its looks written into the book.
fn grower(life: &Life, species: &Species, world: &World, wind: f32, stage: Stage) -> Grower {
    let dials = flora::dials(life, species, world, wind, stage);
    let arch = dials.architecture;
    let rich = productivity(life, species, world);
    Grower {
        arch,
        habitat: species.role.habitat,
        tolerance: species.tolerance,
        chance: CHANCE[usize::from(arch)] * (0.3 + 0.7 * rich),
        height: dials.form.height,
        layers: dials.form.layers,
        habit: habit(arch),
        looks: std::array::from_fn(|v| {
            page(Look {
                dials,
                variant: v as u8,
            })
        }),
    }
}

impl Flora {
    /// Returns the flora of the chunk centred on chart position `center` of `terrain`, or nothing where the
    /// hand-made tables apply: on the legacy plane and where the life of the planet was never given.
    pub fn of(terrain: &Terrain, center: Vec2) -> Option<Self> {
        let globe = terrain.globe();
        let lives = globe.lives().filter(|_| terrain.chart() != Chart::Flat)?;
        let Some(life) = globe.life().filter(|_| lives) else {
            return Some(Self::default());
        };
        let latitude = terrain
            .chart()
            .dir(center.x, center.y)
            .y
            .clamp(-1.0, 1.0)
            .asin() as f32;
        let stage = flora::stage(globe.season(), latitude, globe.record().tilt as f32);
        let (world, wind) = (world_of(globe), breeze(globe));
        let groups = flora::producers(life).map(|s| grower(life, s, &world, wind, stage));
        let mut growers: Vec<Grower> = groups.collect();
        let crowd = |arch: u8, all: &[Grower]| all.iter().filter(|g| g.arch == arch).count() as f32;
        let shares: Vec<f32> = growers.iter().map(|g| crowd(g.arch, &growers)).collect();
        growers
            .iter_mut()
            .zip(shares)
            .for_each(|(g, n)| g.chance /= n);
        Some(Self { growers })
    }
}
