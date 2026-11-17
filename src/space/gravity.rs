//! Gravity: the pull of the anchored planet on free bodies.

use bevy::prelude::*;

use super::{Anchor, altitude, up};
use crate::world::Globe;

/// Returns the acceleration in metres per second squared that `globe` gives a body at render
/// position `pos`, in render axes.
///
/// It points down the local vertical. Its size is the surface gravity at and below sea level,
/// and falls with the inverse square of the distance from the planet centre above it. Only
/// meaningful while the anchor is a surface frame of that globe.
pub fn local_gravity(anchor: &Anchor, globe: &Globe, pos: Vec3) -> Vec3 {
    let radius = globe.radius();
    let distance = radius + f64::from(altitude(anchor, globe, pos)).max(0.0);
    let pull = globe.gravity() * (radius / distance).powi(2);
    -up(anchor, pos) * pull as f32
}

/// Registers the gravity systems.
pub(super) fn plugin(_app: &mut App) {}
