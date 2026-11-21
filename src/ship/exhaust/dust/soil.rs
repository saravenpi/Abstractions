//! What the ground a jet meets is made of, and whether it gives dust.

use super::tune::{GRIT, SETTLE};
use crate::avatar::{Soil, soil};
use crate::universe::{PlanetClass, STANDARD_GRAVITY};
use crate::world::{SEA_LEVEL, Surface};

/// Returns what a jet raises from dry ground of `surface`, `height` metres above sea level, on a
/// planet of `class`, and the share of a full cloud it gives: all of it from sand, the earth under
/// grass, powder snow and the dust of a world without life; a thin grit from rock and gravel, and
/// from the ice of a frozen sea. Water gives none, it throws spray instead, nor does the cloud
/// deck of a gas giant.
pub fn loose(class: PlanetClass, surface: Surface, height: f32) -> Option<(Soil, f32)> {
    match (surface, soil(class, surface)) {
        _ if class == PlanetClass::GasGiant => None,
        (Surface::Rock, kind) => Some((kind, GRIT.0)),
        (Surface::Snow, kind) if height <= SEA_LEVEL => Some((kind, GRIT.0)),
        (_, Soil::Splash | Soil::Air) => None,
        (_, kind) => Some((kind, 1.0)),
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
