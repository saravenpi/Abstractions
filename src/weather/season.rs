//! The season of a planet: how far round the star it is, from 0 to 1.
//!
//! This stands in for `climate::season` until the climate module has it, and is then removed.

use std::f64::consts::TAU;

use crate::universe::PlanetSpec;

/// Returns how far the planet of `spec` is round its star at `time` seconds, from 0 to 1: the
/// angle of its place on the orbit plane. A moon has the season of its planet.
pub fn season(spec: &PlanetSpec, time: f64) -> f32 {
    let at = spec.host.unwrap_or(spec.orbit()).at(time);
    ((at.z.atan2(at.x) / TAU).rem_euclid(1.0) as f32).fract()
}
