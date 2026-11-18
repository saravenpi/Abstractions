//! The lens: what the sky draws the warp with this frame, and where a body is seen through it.

use bevy::math::DVec3;
use bevy::prelude::*;

use super::optics::{bend, bubble, doppler};

/// What the sky draws the warp with. At rest it changes nothing.
#[derive(Resource, Debug, Clone, Copy, PartialEq)]
pub struct Lens {
    /// Unit vector of the direction of travel, in render axes.
    pub heading: Vec3,
    /// How far the sequence is from rest, from 0 to 1.
    pub level: f32,
    /// Speed the sky is seen through, as a share of the speed of light.
    pub beta: f32,
    /// Length in radians of the streak of a star at right angles to the heading.
    pub streak: f32,
    /// Brightness of the light ahead.
    pub core: f32,
    /// Brightness of the flash of the drop-out.
    pub flash: f32,
    /// Share of their light single stars keep.
    pub stars: f32,
}

impl Default for Lens {
    /// Looks along -Z at a sky at rest.
    fn default() -> Self {
        Self {
            heading: Vec3::NEG_Z,
            level: 0.0,
            beta: 0.0,
            streak: 0.0,
            core: 0.0,
            flash: 0.0,
            stars: 1.0,
        }
    }
}

/// Where a body is seen through the lens.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Seen {
    /// Offset from the camera in metres, in render axes.
    pub offset: DVec3,
    /// Seen size over true size at that distance.
    pub scale: f32,
}

#[cfg(test)]
impl Seen {
    /// Returns the angular radius in radians a body of `radius` metres is seen with.
    pub fn angle(&self, radius: f64) -> f32 {
        (radius / self.offset.length().max(1.0e-9)) as f32 * self.scale
    }
}

impl Lens {
    /// Returns where a body of `radius` metres that truly sits `offset` metres from the camera,
    /// in render axes, is seen.
    ///
    /// A body close by rides in the warp bubble and is seen as it is. Farther out it is seen
    /// through the speed of the ship like the stars: pulled toward the heading and smaller
    /// ahead, by the same rule, so a body keeps its place among the stars. The distance is
    /// kept, so the order of the bodies in depth holds.
    ///
    /// The size follows the distance to the body, and the direction the distance at which the
    /// line of flight misses it. That one does not change while the ship flies straight, so a
    /// body the ship passes slides aft the whole way and never swings back.
    pub fn body(&self, offset: DVec3, radius: f64) -> Seen {
        let (distance, radius) = (offset.length(), radius.max(1.0e-9));
        let sized = self.beta * bubble((distance / radius) as f32);
        let Some(dir) = (offset / distance).try_normalize().filter(|_| sized > 0.0) else {
            return Seen { offset, scale: 1.0 };
        };
        let heading = self.heading.as_dvec3();
        let miss = (offset - heading * offset.dot(heading)).length();
        let bent = self.beta * bubble((miss / radius) as f32);
        Seen {
            offset: bend(dir, heading, bent) * distance,
            scale: 1.0 / doppler(dir.dot(heading) as f32, sized),
        }
    }
}
