//! Herds around a point: whether one of a walking species lives within a walk of it.

use bevy::math::DVec3;

use crate::fauna::herds::{Land, read};
use crate::life::body::Locomotion;
use crate::life::herd::{Search, near};
use crate::life::{Guild, Life, Species};
use crate::universe::PlanetSpec;
use crate::world::Globe;

/// Metres within which a herd is looked for around the spawn.
const WALK: f64 = 1500.0;

/// Returns true for a walking animal species.
pub fn walker(s: &Species) -> bool {
    s.role.guild != Guild::Producer && s.body.locomotion == Locomotion::Walk
}

/// Returns true when a herd of a walking species of `life` lives within [`WALK`] of `dir`.
pub fn herd(globe: &Globe, planet: &PlanetSpec, life: &Life, dir: DVec3) -> bool {
    let land = Land {
        globe,
        life,
        planet,
    };
    let search = Search {
        at: dir,
        radius: WALK,
        planet: globe.radius(),
        time: 600.0,
    };
    let herds = read(land, 600.0, |g| near(life, g, &search));
    herds
        .iter()
        .any(|h| walker(&life.species[usize::from(h.species)]))
}
