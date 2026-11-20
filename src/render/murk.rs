//! Under the liquid of a planet: the colour the fog and the sky take when the camera is below
//! the surface of its sea. A dry planet has no sea to be under, and molten rock glows.
//!
//! The fog under the surface is a pass of its own, masked pixel by pixel by whether the view
//! of the pixel starts under the water, see [`gauge`] and [`pass`].

mod gauge;
mod pass;

pub(super) use self::gauge::plugin as gauge;
pub use gauge::{Murk, Swell, Waterline};
pub(super) use pass::plugin as pass;

use bevy::math::DVec3;
use bevy::prelude::*;

use super::fog::UNDERWATER;
use super::vantage::{LocalSystem, Vantage};
use crate::universe::PlanetRecord;
use crate::world::Globe;

/// Depth in metres below sea level past which the whole lens is under the water, below the
/// lowest trough of the waves: from there down the sky is the colour of the liquid.
const DEEP: f32 = 0.5;

/// Brightness of the fog inside a glowing liquid against the glow of its class: the eye is in
/// the light, not looking at its surface, so it blooms without blanking the view.
const GLOW_SHARE: f32 = 0.5;

/// Returns the sRGB colour of the fog and the sky below the surface of the liquid of a planet,
/// or nothing on a dry planet, where nothing is ever submerged.
///
/// Water is the blue-green of the home sea. A liquid that shines by itself, molten rock, is its
/// own colour at a share of its glow.
pub fn murk(record: &PlanetRecord) -> Option<Vec3> {
    if record.ocean <= 0.0 {
        return None;
    }
    let look = record.class.look();
    if look.glow <= 0.0 {
        return Some(UNDERWATER);
    }
    let lit = look.sea * (look.glow * GLOW_SHARE);
    let seen = Color::linear_rgb(lit.x, lit.y, lit.z).to_srgba();
    Some(Vec3::new(seen.red, seen.green, seen.blue))
}

/// Returns the colour under the liquid of the planet below a camera at `vantage` when the whole
/// lens is under its surface there, or nothing above it and on a dry planet. Outside a world
/// the sea is the home sea. Nearer the surface the underwater pass splits the view instead.
pub fn under(local: &LocalSystem, vantage: &Vantage) -> Option<Vec3> {
    if vantage.altitude >= -DEEP {
        return None;
    }
    let rank = u8::try_from(vantage.body).ok();
    let planet = rank.and_then(|rank| local.0.as_ref()?.body(rank, vantage.moon));
    murk(planet.map_or(&PlanetRecord::TEMPERATE, |planet| &planet.record))
}

/// Returns the height of a point `altitude` metres above the sea level of `globe`, along the
/// unit vector `dir` from its centre, above what lies under it: the ground, or the sea where
/// the ground is below it. A dry planet has no sea to stand over, so its low ground counts
/// down to where it is.
pub fn clearance(globe: &Globe, dir: DVec3, altitude: f32) -> f32 {
    altitude - globe.height(dir).max(globe.sea_level())
}
