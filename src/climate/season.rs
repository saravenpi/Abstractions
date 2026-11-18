//! Season: how far round its orbit a planet is, the input the climate's yearly swing reads.

use std::f64::consts::TAU;

use crate::universe::PlanetSpec;

/// Returns the season of `spec` at `time` seconds, from 0 to 1: the angle along its orbit around
/// the star as a share of a turn, the host's orbit for a moon. At 0 the north has its summer.
pub fn season(spec: &PlanetSpec, time: f64) -> f32 {
    let orbit = spec.host.unwrap_or_else(|| spec.orbit());
    let angle = orbit.phase + TAU * time / orbit.seconds;
    ((angle / TAU).rem_euclid(1.0) as f32) % 1.0
}
