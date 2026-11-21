//! What the ground a jet meets is made of, and whether it gives dust.

use super::tune::SETTLE;
use crate::avatar::{Soil, soil};
use crate::universe::{PlanetClass, STANDARD_GRAVITY};
use crate::world::{SEA_LEVEL, Surface};

/// Returns what a jet raises from ground of `surface`, `height` metres above sea level, on a
/// planet of `class`: sand, the earth under grass, powder snow and the dust of a world without
/// life. Rock gives none, nor does water, which throws spray instead, nor ice, the snow of a
/// frozen sea, nor the cloud deck of a gas giant.
pub fn loose(class: PlanetClass, surface: Surface, height: f32) -> Option<Soil> {
    match (surface, soil(class, surface)) {
        _ if class == PlanetClass::GasGiant => None,
        (Surface::Rock, _) => None,
        (Surface::Snow, _) if height <= SEA_LEVEL => None,
        (_, Soil::Rock | Soil::Splash | Soil::Air) => None,
        (_, loose) => Some(loose),
    }
}

/// Returns the speed in metres per second at which a puff of `soil` settles under `gravity` in
/// metres per second squared: its speed under standard gravity in [`SETTLE`], in proportion.
pub fn settle(soil: Soil, gravity: f32) -> f32 {
    let home = SETTLE
        .iter()
        .find(|(kind, _)| *kind == soil)
        .map_or(SETTLE[1].1, |(_, speed)| *speed);
    home * gravity / STANDARD_GRAVITY as f32
}
