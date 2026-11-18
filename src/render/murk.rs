//! Under the liquid of a planet: the colour the fog and the sky take when the camera is below
//! the surface of its sea. A dry planet has no sea to be under, and molten rock glows.

use bevy::math::DVec3;
use bevy::prelude::*;

use super::fog::UNDERWATER;
use super::vantage::{LocalSystem, Vantage};
use crate::universe::PlanetRecord;
use crate::world::Globe;

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

/// Returns the colour under the liquid of the planet below a camera at `vantage` when it is
/// under sea level there, or nothing above it and on a dry planet. Outside a world the sea is
/// the home sea.
pub fn under(local: &LocalSystem, vantage: &Vantage) -> Option<Vec3> {
    if vantage.altitude >= 0.0 {
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
