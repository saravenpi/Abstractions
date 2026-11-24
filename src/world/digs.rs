//! Player-carved terrain craters: spherical stamps, deterministic height deltas, and saves.

mod map;
mod save;
#[cfg(test)]
mod tests;
mod tint;

use bevy::prelude::*;
use serde::{Deserialize, Serialize};

pub use self::map::{Digs, MAX_DIG_DEPTH, MAX_DIG_STAMPS};
pub use self::save::{load, store};
use crate::universe::BodyId;

/// Default crater radius carved per dig stroke, in metres.
pub const DIG_RADIUS: f32 = 1.85;

/// Default bare-hand crater depth carved per dig stroke, in metres.
pub const DIG_DEPTH: f32 = 0.48;

/// Spherical terrain crater stamp anchored to a planet body.
#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq)]
pub struct DigStamp {
    /// Planet where the crater was dug.
    pub planet: BodyId,
    /// Unit surface direction from the planet centre in its rotating frame.
    pub dir: [f64; 3],
    /// Crater radius along the surface, in metres.
    pub radius: f32,
    /// Peak vertical depression at the crater centre, in metres.
    pub depth: f32,
}

/// Message emitted when a dig stroke carves a crater on a planet.
#[derive(Message, Debug, Clone, Copy, PartialEq)]
pub struct TerrainDug(pub DigStamp);
