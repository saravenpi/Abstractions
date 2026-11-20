//! Seasons in the colours of the ground: leaves turn gold and rust in autumn and grass and
//! leaves fade under snow where the climate freezes, by latitude and the place on the orbit.
//!
//! The ground shader turns every fragment by the season of its own latitude and height, from a small
//! block of the state of the planet, so the season is the same from the ground and from orbit and no
//! mesh is ever built again. This file holds the rules, and the CPU twin of what `season.wesl` does.

#[cfg(test)]
use bevy::prelude::*;

use super::celestial::smoothstep;
#[cfg(test)]
use super::weather::alike;
#[cfg(test)]
use crate::climate::{self, Setting, Spot};
#[cfg(test)]
use crate::universe::NO_AIR;

/// Temperatures in kelvin from which snow lies on all the flat ground down to none at all.
#[cfg(test)]
pub(super) const FREEZE: (f32, f32) = (269.0, 275.0);

/// Lean of the sun path under which autumn shows only in part: the sine of the tilt times the
/// sine of the latitude, about a tilt of 23 degrees at a latitude of 23 degrees.
const AUTUMN_REF: f32 = 0.15;

/// Share of a leaf colour the full autumn turns.
#[cfg(test)]
pub(super) const AUTUMN_SHARE: f32 = 0.75;

/// Shares of the local year where autumn colours start, are full, start to go and are gone:
/// from the equinox after summer to just past the winter solstice.
const AUTUMN_SPAN: [f32; 4] = [0.22, 0.34, 0.45, 0.56];

/// Early autumn hue of a leaf, in sRGB.
#[cfg(test)]
pub(super) const GOLD: Vec3 = Vec3::new(0.80, 0.56, 0.18);

/// Late autumn hue of a leaf, in sRGB.
#[cfg(test)]
pub(super) const RUST: Vec3 = Vec3::new(0.70, 0.30, 0.12);

/// Colour of fresh snow, in sRGB.
#[cfg(test)]
pub(super) const SNOW: Vec3 = Vec3::new(0.93, 0.95, 0.98);

/// How sharply a colour counts as a leaf as green rises over red and blue.
#[cfg(test)]
pub(super) const LEAF_GAIN: f32 = 6.0;

/// What the season does to the ground at one place, each value from 0 to 1.
#[cfg(test)]
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SeasonLook {
    /// How far the leaves have turned.
    pub autumn: f32,
    /// How much of the flat ground snow covers.
    pub snow: f32,
}

#[cfg(test)]
impl SeasonLook {
    /// The ground as it is drawn without seasons.
    pub const NONE: Self = Self {
        autumn: 0.0,
        snow: 0.0,
    };

    /// Returns the look of the ground at `spot` on the planet of `setting` at `season`.
    pub fn at(setting: &Setting, spot: &Spot, season: f32) -> Self {
        let latitude = spot.dir.y.clamp(-1.0, 1.0).asin() as f32;
        let air = setting.pressure >= NO_AIR as f32;
        let temperature = climate::at(setting, spot, season).temperature;
        Self {
            autumn: autumn(latitude, season, setting.tilt),
            snow: if air { snow(temperature) } else { 0.0 },
        }
    }
}

#[cfg(test)]
/// Returns how far the leaves have turned at `latitude` radians at `season` on a planet whose
/// spin axis leans by `tilt` radians: nothing at the equator or without a tilt, most in the
/// autumn of each hemisphere, half a year apart.
pub fn autumn(latitude: f32, season: f32, tilt: f32) -> f32 {
    let strength = (tilt.sin().abs() * latitude.sin().abs() / AUTUMN_REF).min(1.0);
    let south = if latitude < 0.0 { 0.5 } else { 0.0 };
    strength * window((season + south).rem_euclid(1.0))
}

/// Returns how far autumn has come at `local` of the local year, from 0 to 1, before the lean of the sun
/// path is counted.
pub fn window(local: f32) -> f32 {
    let [start, full, fade, gone] = AUTUMN_SPAN;
    smoothstep(start, full, local) * (1.0 - smoothstep(fade, gone, local))
}

/// Returns the lean of the sun path per unit of the sine of the latitude that makes autumn full, for a spin
/// axis that leans by `tilt` radians.
pub fn lean(tilt: f32) -> f32 {
    tilt.sin().abs() / AUTUMN_REF
}

#[cfg(test)]
/// Returns how much of the flat ground snow covers at `temperature` kelvin.
pub fn snow(temperature: f32) -> f32 {
    1.0 - smoothstep(FREEZE.0, FREEZE.1, temperature)
}

#[cfg(test)]
/// Returns how much the sRGB `color` reads as grass or leaves, from 0 to 1.
fn leafiness(color: Vec3) -> f32 {
    let over = color.y - color.x.max(color.z);
    (over / color.y.max(1.0e-3) * LEAF_GAIN).clamp(0.0, 1.0)
}

#[cfg(test)]
/// Returns the sRGB ground colour `color` under the season `look`: grass and leaves turn
/// toward gold, then rust, and fade to snow. Rock, sand, wood and water stay as they are.
pub fn tint(color: Vec3, look: SeasonLook) -> Vec3 {
    let leaf = leafiness(color);
    if leaf <= 0.0 {
        return color;
    }
    let hue = GOLD.lerp(RUST, look.autumn);
    let turned = color.lerp(alike(color, hue), look.autumn * AUTUMN_SHARE * leaf);
    turned.lerp(SNOW, look.snow * leaf).min(Vec3::ONE)
}
