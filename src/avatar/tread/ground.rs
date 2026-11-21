//! What a foot stands on, read from what the world has there: the deck and the pad of the
//! base, the wing and the hull of a ship, and the natural ground, its cover, its biome, its
//! water and the rain on it.

use bevy::ecs::system::SystemParam;
use bevy::prelude::*;

use super::Hulls;
use super::tune::{GRAVEL_SLOPE, MUDDY, RAISED, RIVER, WADE};
use crate::audio::StepSurface;
use crate::home::Base;
use crate::space::{Anchor, Location};
use crate::weather::Weather;
use crate::world::{Biome, Surface, Terrain};

/// The world around the feet.
#[derive(SystemParam)]
pub struct Site<'w> {
    /// Frame time.
    pub time: Res<'w, Time>,
    /// The ground.
    pub terrain: Res<'w, Terrain>,
    /// The frame of the render anchor.
    pub anchor: Res<'w, Anchor>,
    /// The planet the player is on.
    pub location: Res<'w, Location>,
    /// The ships one can stand on.
    pub hulls: Res<'w, Hulls>,
    /// The base of the player.
    pub base: Option<Res<'w, Base>>,
    /// The weather where the player is.
    pub weather: Option<Res<'w, Weather>>,
}

/// Returns the step of earth under `rain` from 0 to 1: soil, turned to mud once wet.
pub fn soil(rain: f32) -> StepSurface {
    match rain > MUDDY {
        true => StepSurface::Mud,
        false => StepSurface::Soil,
    }
}

/// Returns loose stones where the rock at chart position `at` lies nearly flat, bare rock
/// where it is steep.
fn rock(terrain: &Terrain, at: Vec3) -> StepSurface {
    let across = terrain.height(at.x + 1.0, at.z) - terrain.height(at.x - 1.0, at.z);
    let along = terrain.height(at.x, at.z + 1.0) - terrain.height(at.x, at.z - 1.0);
    match 0.5 * across.hypot(along) < GRAVEL_SLOPE {
        true => StepSurface::Gravel,
        false => StepSurface::Rock,
    }
}

/// Returns what the natural ground gives a foot at chart position `feet`, under `rain` from 0
/// to 1, with the depth of the water it stands in. A foot in water deeper than a wade swims,
/// and a gas giant has no ground: neither gives a step.
pub fn natural(terrain: &Terrain, feet: Vec3, rain: f32) -> Option<(StepSurface, f32)> {
    let globe = terrain.globe();
    let depth = globe.sea_level() - feet.y;
    if !globe.has_ground() || depth > WADE.1 {
        return None;
    }
    if depth > WADE.0 {
        return Some((StepSurface::Water, depth));
    }
    let earth = soil(rain);
    let surface = match terrain.surface(feet.x, feet.z) {
        Surface::River => return Some((StepSurface::Water, RIVER)),
        Surface::Water | Surface::Shallows | Surface::Sand => StepSurface::Sand,
        Surface::Pond => StepSurface::Mud,
        Surface::Reef => StepSurface::Rock,
        Surface::Rock => rock(terrain, feet),
        Surface::Snow => StepSurface::Snow,
        Surface::Grass => match terrain.biome(feet.x, feet.z) {
            Biome::Forest | Biome::Taiga | Biome::Tropical => earth,
            _ => StepSurface::Grass,
        },
    };
    Some((surface, 0.0))
}

/// Returns what a foot at render position `feet` stands on, with the depth of the water it
/// stands in. Raised over the ground it stands on a ship, a deck of the base or a boulder; on
/// the ground, on the landing pad or the natural ground.
pub fn footing(site: &Site, feet: Vec3) -> Option<(StepSurface, f32)> {
    let (terrain, at) = (&*site.terrain, site.anchor.to_frame(feet));
    let chart = terrain.locate(at);
    let raised = chart.y - terrain.height(chart.x, chart.z) > RAISED;
    let rain = site.weather.as_ref().map_or(0.0, |weather| weather.rain);
    let base = site
        .base
        .as_ref()
        .filter(|base| base.body == site.location.body);
    let deck = base.is_some_and(|base| (raised && base.covers(at, 0.0)) || base.on_pad(at));
    let mut ships = site.hulls.0.iter();
    let hull = raised && ships.any(|(centre, reach)| centre.distance(at) <= f64::from(*reach));
    let plain = |surface| Some((surface, 0.0));
    match (hull, deck) {
        (true, _) => plain(StepSurface::Hull),
        (_, true) => plain(StepSurface::Metal),
        _ if raised => plain(StepSurface::Rock),
        _ => natural(terrain, chart, rain),
    }
}
