//! Rain: how wet a place is, from the ocean share, the circulation belts, the distance to the
//! sea and the slopes facing the wind.
//!
//! The belt table is the S(phi) table of davudk, procedural-world-generation,
//! <https://github.com/davudk/procedural-world-generation/blob/main/docs/research/03-climate-biomes.md>.

use bevy::math::Vec2;

use super::bands::slow;
use super::{Setting, Spot};

/// Rain factor by latitude in degrees on a fast spinner: wet equator, dry subtropics near 30
/// degrees, wet mid latitudes near 50 and dry poles.
const BELTS: [(f32, f32); 9] = [
    (0.0, 2.0),
    (10.0, 1.4),
    (20.0, 0.35),
    (30.0, 0.25),
    (40.0, 0.9),
    (50.0, 1.3),
    (60.0, 1.1),
    (70.0, 0.6),
    (90.0, 0.25),
];

/// Rain factor by latitude in degrees on a slow spinner, one cell per hemisphere: a wet equator
/// and dry land from 25 degrees to the pole.
const ONE_CELL: [(f32, f32); 4] = [(0.0, 2.0), (10.0, 1.4), (25.0, 0.25), (90.0, 0.25)];

/// How fast rain falls off inland: the factor is `exp(-1.5 * inland)`.
const INLAND: f32 = 1.5;

/// Rain gained per unit of slope climbed along the wind.
const LIFT: f32 = 2.0;

/// Smallest and largest factor slopes give to the rain.
const LIFT_RANGE: (f32, f32) = (0.3, 2.5);

/// Returns the value of the piecewise linear `table` at `deg`, held at its ends.
fn lookup(table: &[(f32, f32)], deg: f32) -> f32 {
    let upper = table
        .iter()
        .position(|&(d, _)| d >= deg)
        .unwrap_or(table.len() - 1);
    let (d1, v1) = table[upper];
    let (d0, v0) = table[upper.saturating_sub(1)];
    match d1 > d0 {
        true => v0 + (v1 - v0) * ((deg - d0) / (d1 - d0)).clamp(0.0, 1.0),
        false => v1,
    }
}

/// Returns the wetness at `spot`, at latitude `lat` radians, under the wind `wind`.
///
/// The ocean share times the belt factor, times `exp(-1.5 * inland)` past the coast, times
/// `1 + 2 * slope . wind direction` held to 0.3 to 2.5: windward slopes catch the rain, lee
/// slopes lie in its shadow.
pub(super) fn moisture(setting: &Setting, spot: &Spot, lat: f32, wind: Vec2) -> f32 {
    let table: &[(f32, f32)] = match slow(setting) {
        true => &ONE_CELL,
        false => &BELTS,
    };
    let belt = lookup(table, lat.to_degrees().abs());
    let coast = (-INLAND * spot.inland.max(0.0)).exp();
    let lift =
        (1.0 + LIFT * spot.slope.dot(wind.normalize_or_zero())).clamp(LIFT_RANGE.0, LIFT_RANGE.1);
    setting.ocean.max(0.0) * belt * coast * lift
}
