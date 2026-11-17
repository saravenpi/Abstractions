//! Local vertical: which way is up and how high a render position is, on the anchored planet.

use bevy::prelude::*;

use super::Anchor;
use crate::world::{Globe, SEA_LEVEL};

/// Returns the height in metres of render position `pos` above the sea level of `globe`.
///
/// Only meaningful while the anchor is a surface frame of that globe.
pub fn altitude(anchor: &Anchor, globe: &Globe, pos: Vec3) -> f32 {
    if anchor.flat {
        return pos.y - SEA_LEVEL;
    }
    (anchor.to_frame(pos).length() - globe.radius()) as f32
}

/// Returns the unit direction away from the planet centre at render position `pos`, in render axes.
///
/// Only meaningful while the anchor is a surface frame.
pub fn up(anchor: &Anchor, pos: Vec3) -> Vec3 {
    if anchor.flat {
        return Vec3::Y;
    }
    (anchor.rotation * anchor.to_frame(pos).normalize()).as_vec3()
}
