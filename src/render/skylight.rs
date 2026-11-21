//! How bright the sky is, and what that hides: the stars and the band of the galaxy show only
//! where the sky behind them is darker than they are.
//!
//! The old rule keyed both on the height of the star over the horizon, and on the thinness of the
//! air, so a world with thin air showed its stars and its galaxy through a bright day sky. Here
//! the luminance of the sky colour at the direction decides, the same quantity the dome draws.
//! A sky at the luminance [`DAY`] is the clear home noon sky, which hides every star fainter than
//! [`DAY_FLUX`]. Each decade of luminance below it lets stars [`SPAN`] steps of flux fainter
//! show, so the brightest stars come out first at dusk, as they do in the real twilight, and the
//! faintest ones need the dark of night.

use bevy::prelude::*;

#[cfg(test)]
use super::starfield::DAY_FLUX;

/// Luminance in linear light of the clear home noon sky: what hides every star but the star of
/// the system.
#[cfg(test)]
pub const DAY: f32 = 0.3;

/// Steps of star flux gained for each decade the luminance of the sky falls under [`DAY`]. The
/// palette keeps the night sky about two decades under the day one, and the stars of the field
/// span about six steps of flux.
#[cfg(test)]
pub const SPAN: f32 = 3.0;

/// Smallest luminance read, under which the sky is black.
#[cfg(test)]
pub const LEAST: f32 = 0.0001;

/// Luminance of the sky under which the band of the galaxy shows in full, and over which it is
/// gone. Unit: linear light.
pub const BAND: (f32, f32) = (0.004, 0.03);

/// Returns the luminance in linear light of the sRGB colour `srgb`.
pub fn luminance(srgb: Vec3) -> f32 {
    let linear = Color::srgb(srgb.x, srgb.y, srgb.z).to_linear();
    0.2126 * linear.red + 0.7152 * linear.green + 0.0722 * linear.blue
}

/// Returns the flux under which a star is hidden by a sky of luminance `sky`, and over which
/// plus one it is shown in full. It is never over [`DAY_FLUX`]: a brighter sky hides no more.
#[cfg(test)]
pub fn hide_flux(sky: f32) -> f32 {
    (DAY_FLUX + SPAN * (sky.max(LEAST) / DAY).log10()).min(DAY_FLUX)
}

/// Returns how far a star with the light `flux` shows through a sky of luminance `sky`, from 0
/// to 1. `starfield.wesl` has the same rule.
#[cfg(test)]
pub fn shown(flux: f32, sky: f32) -> f32 {
    let t = (flux - hide_flux(sky)).clamp(0.0, 1.0);
    t * t * (3.0 - 2.0 * t)
}

/// Returns how far the band of the galaxy shows through a sky of luminance `sky`, from 0 to 1.
pub fn band(sky: f32) -> f32 {
    let t = ((sky - BAND.0) / (BAND.1 - BAND.0)).clamp(0.0, 1.0);
    1.0 - t * t * (3.0 - 2.0 * t)
}

/// Returns how far the band of the galaxy shows under a sky with the sRGB colours `zenith` and
/// `horizon`: its mean luminance decides.
pub fn band_of(zenith: Vec3, horizon: Vec3) -> f32 {
    band(0.5 * (luminance(zenith) + luminance(horizon)))
}
