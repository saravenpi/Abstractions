//! Places on a planet: the latitude and longitude of a direction from its centre, the point of
//! the flat map they are drawn at, and the words that say them. The flat map wraps east and
//! west: a point and its copies whole turns away are the same place.
//!
//! The flat map is the whole planet spread from longitude 0 on the left edge, going east, to
//! the full turn on the right edge, with the north pole up. It is twice as wide as it is high
//! and fills the middle half of the square chart, from -1 to 1 across and -0.5 to 0.5 up.

use std::f64::consts::{FRAC_PI_2, PI, TAU};

use bevy::math::{DVec3, Vec2};

use crate::computer::direction;
use crate::ui::units::fixed;

/// Returns the latitude and the longitude of the direction `dir` in radians: the latitude
/// from -pi/2 at the south pole to pi/2 at the north pole, the longitude from 0 to a full turn
/// east. It undoes [`direction`].
pub fn lat_lon(dir: DVec3) -> (f64, f64) {
    let dir = dir.normalize_or(DVec3::X);
    let lat = dir.y.clamp(-1.0, 1.0).asin();
    let lon = (-dir.z).atan2(dir.x).rem_euclid(TAU);
    (lat, lon)
}

/// Returns the chart point the direction `dir` is drawn at on the flat map.
pub fn to_chart(dir: DVec3) -> Vec2 {
    let (lat, lon) = lat_lon(dir);
    Vec2::new((lon / PI - 1.0) as f32, (lat / PI) as f32)
}

/// Returns the chart point `at` moved by whole turns of longitude into the flat map, from -1
/// to 1 across, for a point of a map that wraps east and west.
pub fn wrapped(at: Vec2) -> Vec2 {
    Vec2::new((at.x + 1.0).rem_euclid(2.0) - 1.0, at.y)
}

/// Returns the copy of the chart point `at`, whole turns of longitude east or west, nearest to
/// the longitude `x` the lens looks at.
pub fn copy_near(at: Vec2, x: f32) -> Vec2 {
    Vec2::new(at.x + 2.0 * ((x - at.x) * 0.5).round(), at.y)
}

/// Returns the direction drawn at the chart point `at`, or nothing off the flat map.
pub fn from_chart(at: Vec2) -> Option<DVec3> {
    let inside = at.x.abs() <= 1.0 && at.y.abs() <= 0.5;
    inside.then(|| direction((f64::from(at.x) + 1.0) * PI, f64::from(at.y) * PI))
}

/// Returns the latitude and the longitude of `dir` in words, as `12.4° N  103.2° E`: the
/// longitude counted east or west of longitude 0, at most half a turn.
pub fn coords(dir: DVec3) -> String {
    let (lat, lon) = lat_lon(dir);
    let lon = if lon > PI { lon - TAU } else { lon };
    let north = ["S", "N"][usize::from(lat >= 0.0)];
    let east = ["W", "E"][usize::from(lon >= 0.0)];
    let degrees = |angle: f64| (angle.abs() / FRAC_PI_2 * 90.0 * 10.0).round() / 10.0;
    format!(
        "{}° {north}  {}° {east}",
        fixed(degrees(lat), 1),
        fixed(degrees(lon), 1)
    )
}
