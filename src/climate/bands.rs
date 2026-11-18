//! Wind bands: the prevailing surface wind by latitude, as the circulation cells drive it.
//!
//! Trade winds blow from the east toward the equator from 0 to 30 degrees, westerlies toward the
//! pole from 30 to 60, polar easterlies toward the equator from 60 to 90, mirrored in the south.
//! Bands after davudk, procedural-world-generation,
//! <https://github.com/davudk/procedural-world-generation/blob/main/docs/research/03-climate-biomes.md>.

use bevy::math::Vec2;

use super::Setting;
use crate::universe::NO_AIR;

/// Wind of the trade band in m/s at home air: east and toward the pole.
const TRADES: Vec2 = Vec2::new(-7.6, -2.4);

/// Wind of the westerly band in m/s at home air: east and toward the pole.
const WESTERLIES: Vec2 = Vec2::new(11.4, 3.7);

/// Wind of the polar band in m/s at home air: east and toward the pole.
const POLAR: Vec2 = Vec2::new(-3.9, -1.2);

/// Degrees on each side of a band edge over which two bands blend.
const BLEND: f32 = 5.0;

/// Day in seconds past which a planet turns too slowly for three cells: one cell spans each
/// hemisphere and the trades blow all the way to the pole.
pub(super) const SLOW_DAY: f32 = 3600.0;

/// Returns 0 below `edge - BLEND`, 1 above `edge + BLEND` and a smooth step between.
fn step(edge: f32, x: f32) -> f32 {
    let t = ((x - edge + BLEND) / (2.0 * BLEND)).clamp(0.0, 1.0);
    t * t * (3.0 - 2.0 * t)
}

/// Returns true when the planet of `setting` turns slower than [`SLOW_DAY`].
pub(super) fn slow(setting: &Setting) -> bool {
    setting.day_seconds.abs() > SLOW_DAY
}

/// Returns the prevailing wind in m/s, toward the east and the north, at latitude `lat` radians.
///
/// Band cores blow 4 to 12 m/s at home air, scaled by the square root of the pressure; the wind
/// falls calm across the band edges, where two bands meet, and is zero without air.
pub(super) fn wind(setting: &Setting, lat: f32) -> Vec2 {
    if setting.pressure < NO_AIR as f32 {
        return Vec2::ZERO;
    }
    let deg = lat.to_degrees().abs();
    let band = match slow(setting) {
        true => TRADES,
        false => TRADES
            .lerp(WESTERLIES, step(30.0, deg))
            .lerp(POLAR, step(60.0, deg)),
    };
    let toward_north = lat.signum() * step(BLEND, deg);
    Vec2::new(band.x, band.y * toward_north) * setting.pressure.sqrt()
}
