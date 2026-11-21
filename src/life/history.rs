//! Life history from mass: small species are many, quick and short-lived, large ones few, slow and caring.
//!
//! The rules are allometric. Life span grows as `mass^0.25`, top speed follows the running fit of
//! Hirt and others (<https://www.nature.com/articles/s41559-017-0241-4>) scaled by the gravity of the
//! planet, and the home range area grows as `mass^1` for every consumer, hunters on a wider range
//! (Jetz and others 2004, "The scaling of animal space use", Science 306, home range area about
//! `mass^1.02` across mammals, <https://www.science.org/doi/10.1126/science.1102138>). Herds of grazers
//! grow with body size, as group size grows with body mass in antelope (Jarman 1974, re-tested by
//! Brashares and others 2000, <https://biology.ucr.edu/people/faculty/Garland/BrasEA00.pdf>).

use super::{Guild, Habitat, History, Role, Rung, World};
use crate::universe::hash::Rng;

#[cfg(test)]
mod tests;

/// Seconds of the game day life spans are counted in: twenty minutes, whatever the spin of the planet.
const DAY: f32 = 1200.0;

/// Gravity in metres per second squared that the speed fit was made for.
const EARTH_GRAVITY: f32 = 9.81;

/// Highest Froude number, `v^2 / (g * leg)`, a walker is driven at: a full gallop, past which its
/// feet could no longer keep up with its body (Alexander and Jayes 1983, "A dynamic similarity
/// hypothesis for the gaits of quadrupedal mammals", Journal of Zoology 201, 135-152).
const FROUDE_TOP: f32 = 3.5;

/// Home range radius in metres of a 100 kg plant eater.
const RANGE_100KG: f32 = 400.0;

/// Mass in kilograms from which grazers live in large herds.
const LARGE_GRAZER: f32 = 20.0;

/// Returns the fastest gallop in metres per second of a walker of leg `height` metres on a world of
/// surface `gravity`.
pub fn gait(height: f32, gravity: f32) -> f32 {
    (FROUDE_TOP * gravity * height.max(0.05)).sqrt()
}

/// Returns the life history of a species of `role` and adult `mass` in kilograms on `world`.
pub fn of(seed: u64, role: &Role, mass: f32, rung: Rung, world: &World) -> History {
    let mass = mass.max(1e-4);
    let mut rng = Rng::new(seed);
    let mut jitter = || rng.range(0.9, 1.1) as f32;
    let (group_min, group_max) = group(role.guild, mass, rung);
    History {
        mass,
        lifespan: DAY * 10.0 * mass.powf(0.25) * jitter(),
        clutch: (8.0 * mass.powf(-0.33) * jitter()).round().clamp(1.0, 60.0) as u8,
        group_min,
        group_max,
        care: care(mass, rung),
        speed: speed(role.habitat, mass, world.gravity) * jitter(),
        range: range(role.guild, mass) * jitter(),
    }
}

/// Returns parental care, 0 to 1: none for the smallest, guarding parents for the large and the evolved.
fn care(mass: f32, rung: Rung) -> f32 {
    let evolved = (rung as u8 as f32 - Rung::Shelled as u8 as f32) * 0.1;
    (0.3 + 0.15 * mass.log10() + evolved).clamp(0.0, 1.0)
}

/// Returns the smallest and largest group of a guild: swarms at the small end, herds, packs and loners.
fn group(guild: Guild, mass: f32, rung: Rung) -> (u8, u8) {
    match guild {
        Guild::Producer | Guild::Ambusher | Guild::Parasite => (1, 1),
        Guild::Hunter if rung == Rung::Old => (4, 8),
        Guild::Hunter => (1, 4),
        _ if mass < 0.05 => (20, 60),
        Guild::Grazer if mass >= LARGE_GRAZER => (5, 30),
        Guild::Grazer => (4, 24),
        Guild::Browser => (2, 8),
        Guild::Granivore | Guild::Filter => (6, 30),
        Guild::Frugivore => (2, 12),
        Guild::Nectar | Guild::Scavenger => (1, 6),
        Guild::Decomposer => (20, 60),
    }
}

/// Returns the top speed in metres per second: the running fit, faster in the air, slower in the liquid
/// and on heavy worlds.
fn speed(habitat: Habitat, mass: f32, gravity: f32) -> f32 {
    let run = 25.5 * mass.powf(0.26) * (1.0 - (-22.0 * mass.powf(-0.6)).exp()) / 3.6;
    let medium = match habitat {
        Habitat::Air => 1.5,
        Habitat::Water => 0.6,
        Habitat::Ground | Habitat::Shore => 1.0,
    };
    let weight = (EARTH_GRAVITY / gravity.max(0.1)).sqrt().clamp(0.4, 1.6);
    run * medium * weight
}

/// Returns the home range radius in metres: area as `mass^1`, so radius as `mass^0.5`, three times as
/// wide for hunters and scavengers.
fn range(guild: Guild, mass: f32) -> f32 {
    let wide = match guild {
        Guild::Hunter | Guild::Scavenger => 3.0,
        _ => 1.0,
    };
    (wide * RANGE_100KG * (mass / 100.0).sqrt()).max(5.0)
}
