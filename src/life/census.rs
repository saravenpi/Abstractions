//! The population field: how many of a species live at one place, from its mass, its level and the site.
//!
//! Density follows Damuth's law, `mass^-0.75` (<https://pdodds.w3.uvm.edu/files/papers/others/1981/damuth1981a.pdf>),
//! loses about nine tenths per trophic level, and is shaped by how well the site fits the species:
//! its climate, its habitat and its cover (see `open`).
//! Producers have no density here: plants are not herds.

use super::body::Locomotion;
use super::{Guild, Habitat, Species, Tolerance};
use crate::world::Biome;

mod open;
#[cfg(test)]
mod tests;
#[cfg(test)]
mod tests_scaling;

/// Individuals per square kilometre of a 1 kg plant eater, trophic level 2, before the site.
///
/// Damuth's law is fitted on plant-eating mammals, so the tenfold loss per trophic level counts
/// from level 2, not from the plants. Chosen so that a temperate meadow holds a few herds per
/// square kilometre and a walker meets a herd of large grazers within a few hundred metres.
const DAMUTH: f32 = 1300.0;

/// Trophic level Damuth's law is fitted at: the plant eaters.
const DAMUTH_LEVEL: f32 = 2.0;

/// Most herds of one species per square kilometre, however small it is.
const CROWD: f32 = 3.0;

/// Heights in metres above sea level that count as shore.
const SHORE: f32 = 12.0;

/// The climate at one point of a planet, as census and herds read it.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Site {
    /// Ground height above sea level in metres.
    pub height: f32,
    /// Air temperature in kelvin.
    pub temperature: f32,
    /// Wetness, 0 to 1.
    pub moisture: f32,
    /// Biome of the ground.
    pub biome: Biome,
    /// True on the sea or a pond.
    pub liquid: bool,
    /// Daylight, 0 (night) to 1 (noon).
    pub daylight: f32,
}

/// Returns the individuals of `species` per square kilometre at `site`.
pub fn density(species: &Species, site: &Site) -> f32 {
    peak(species)
        * fit(&species.tolerance, site)
        * habitat(species, site)
        * open::cover(species, site.biome)
}

/// Returns the herds of `species` per square kilometre at `site`: its density over its mean group size,
/// softly capped so that the smallest species do not fill the ground.
pub fn herds(species: &Species, site: &Site) -> f32 {
    crowd(species, density(species, site))
}

/// Returns the herds of `species` per square kilometre at the site that suits it best: no site has more.
pub fn most(species: &Species) -> f32 {
    crowd(species, peak(species))
}

/// Returns the individuals of `species` per square kilometre where the site suits it fully.
fn peak(species: &Species) -> f32 {
    if species.role.guild == Guild::Producer {
        return 0.0;
    }
    let mass = species.history.mass.max(1e-4);
    let level = (species.role.level - DAMUTH_LEVEL).max(0.0);
    let base = DAMUTH * mass.powf(-0.75) * 0.1f32.powf(level);
    if species.boss {
        base.clamp(0.25, 0.55)
    } else if species.cave {
        base * 0.05
    } else {
        base
    }
}

/// Returns the herds per square kilometre of `species` living at `density` individuals per square
/// kilometre, softly capped at [`CROWD`].
fn crowd(species: &Species, density: f32) -> f32 {
    let history = &species.history;
    let group = (f32::from(history.group_min) + f32::from(history.group_max)) * 0.5;
    let herds = density / group.max(1.0);
    herds * CROWD / (herds + CROWD)
}

/// Returns how well `site` suits where `species` lives, 0 to 1. A species of the air that cannot
/// fly or swarm walks the ground, so it never lives over the sea, and a species of the shore wades
/// no deeper than its hips.
fn habitat(species: &Species, site: &Site) -> f32 {
    let aloft = matches!(species.body.locomotion, Locomotion::Fly | Locomotion::Swarm);
    let wade = species.body.height.max(0.1);
    match species.role.habitat {
        Habitat::Water => f32::from(u8::from(site.liquid)),
        Habitat::Air if aloft => 1.0,
        Habitat::Ground | Habitat::Air => f32::from(u8::from(!site.liquid)),
        Habitat::Shore if site.liquid && -site.height > wade => 0.0,
        Habitat::Shore => window(site.height, -wade, SHORE, SHORE),
    }
}

/// Returns how well the temperature and moisture of `site` fit `tolerance`, 0 to 1.
fn fit(tolerance: &Tolerance, site: &Site) -> f32 {
    window(site.temperature, tolerance.cold, tolerance.heat, 6.0)
        * window(site.moisture, tolerance.dry, tolerance.wet, 0.08)
}

/// Returns 1 inside `low..high`, falling smoothly to 0 over `margin` outside it.
fn window(x: f32, low: f32, high: f32, margin: f32) -> f32 {
    let out = (low - x).max(x - high).max(0.0) / margin;
    let t = (1.0 - out).clamp(0.0, 1.0);
    t * t * (3.0 - 2.0 * t)
}
