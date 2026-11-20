//! The sea datum: how high the liquid of a planet stands for the share of the surface it covers.

use std::f64::consts::PI;

use bevy::math::DVec3;

use super::elevation::tune::SHORE;
use crate::universe::PlanetRecord;

/// Sites the shore of a large planet is measured on: enough to place the share of its low ground within
/// half a percent.
const SITES: u32 = 16_384;

/// Sea heights in raw ground metres by ocean share: each entry is `(share, height)`, the share of the sphere
/// whose raw ground lies at or below that height.
///
/// Measured on 240,000 sites of six planets (seeds 1, 2, 5, 7, 42 and 99, radii from 3 to 40 km), whose
/// curves agree within a metre and a half. The deep floor is flat, so the sea falls fast from a share of
/// 0.5 to 0.25 and then hardly at all, and the land is steep: at 43 m only 0.005 of the sphere stands out,
/// and the highest summit reaches 126 m. The generator itself puts 0.645 of the sphere below its coast: the
/// balanced share is pinned to height 0 all the same, since that coast is where its beaches, cliffs and
/// reefs are.
const LEVELS: [(f64, f32); 15] = [
    (0.01, -27.3),
    (0.02, -20.2),
    (0.05, -16.5),
    (0.1, -15.5),
    (0.2, -14.1),
    (0.3, -11.9),
    (0.4, -7.3),
    (0.5, -4.8),
    (0.55, -3.2),
    (PlanetRecord::BALANCED_OCEAN, 0.0),
    (0.7, 1.4),
    (0.8, 5.6),
    (0.9, 13.2),
    (0.95, 18.6),
    (1.0, 43.5),
];

/// Returns the height of the sea in raw ground metres for an `ocean` share of the surface, or nothing for a
/// dry world.
///
/// [`PlanetRecord::BALANCED_OCEAN`] gives exactly 0, and a larger share always gives a higher sea up to the
/// last level, which still leaves the summits out as small islands.
pub fn datum(ocean: f64) -> Option<f32> {
    if ocean.is_nan() || ocean <= 0.0 {
        return None;
    }
    let share = ocean.clamp(LEVELS[0].0, LEVELS[LEVELS.len() - 1].0);
    let k = LEVELS.windows(2).position(|w| share <= w[1].0).unwrap_or(0);
    let ((s0, h0), (s1, h1)) = (LEVELS[k], LEVELS[k + 1]);
    let t = ((share - s0) / (s1 - s0)) as f32;
    Some(h0 + (h1 - h0) * t)
}

/// Returns the unit directions the shore of a large planet is measured on, spread evenly over the sphere
/// on a golden spiral.
pub fn sites() -> impl Iterator<Item = DVec3> {
    let golden = PI * (3.0 - 5f64.sqrt());
    (0..SITES).map(move |i| {
        let y = 1.0 - 2.0 * (f64::from(i) + 0.5) / f64::from(SITES);
        let (ring, turn) = ((1.0 - y * y).sqrt(), golden * f64::from(i));
        DVec3::new(ring * turn.cos(), y, ring * turn.sin())
    })
}

/// Returns the offset at which the share `low(offset)` of a planet at or under the sea is `lowland`: the
/// datum of a large planet, measured instead of tabled. `low` must fall as the offset grows.
///
/// The offset is halved in on between [`SHORE`]'s two ends, to a millionth of their span. A share no
/// offset reaches gives the end closest to it.
pub fn shore(lowland: f64, low: impl Fn(f32) -> f64) -> f32 {
    let (mut sea, mut land) = SHORE;
    for _ in 0..22 {
        let mid = 0.5 * (sea + land);
        match low(mid) > lowland {
            true => sea = mid,
            false => land = mid,
        }
    }
    0.5 * (sea + land)
}
