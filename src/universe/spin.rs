//! Spins: how fast a body turns, and whether tides have locked its turn to its orbit.

use std::f64::consts::{PI, TAU};

use super::spec::GRAVITATION;

/// Shortest day a body is born with, in break-up periods: at one break-up period the equator
/// is in orbit and the body flies apart, at two it moves at half the orbital speed.
pub const FASTEST: f64 = 2.0;

/// Longest day in seconds a solid planet is born with: four Earth days.
pub const SLOWEST: f64 = 345_600.0;

/// Longest day in seconds a gas giant is born with. Giants gather their spin with their gas, so
/// they turn fast: 30 hours at most.
pub const SLOWEST_GIANT: f64 = 108_000.0;

/// How poorly rock and gas turn tides into heat, the tidal quality factor `Q`: about 100 for a
/// rocky body, 100,000 for a gas giant.
pub const QUALITY: (f64, f64) = (100.0, 1.0e5);

/// Rigidity of rock in pascals, which sets how far a tide deforms a body.
const RIGIDITY: f64 = 3.0e10;

/// Moment of inertia of a planet over its mass times its radius squared: the Earth has 0.33.
const INERTIA: f64 = 0.33;

/// Returns the seconds a body of `radius` metres and surface `gravity` would take per turn for
/// its equator to be in orbit, `2 pi sqrt(R / g)`: 84 minutes for the Earth.
pub fn breakup(radius: f64, gravity: f64) -> f64 {
    TAU * (radius / gravity).sqrt()
}

/// Returns the day in seconds a body is born with for a draw `roll` from 0 to 1: spread evenly
/// on a log scale from [`FASTEST`] break-up periods to `slowest` seconds.
pub(super) fn born(roll: f64, radius: f64, gravity: f64, slowest: f64) -> f64 {
    let fastest = FASTEST * breakup(radius, gravity);
    fastest * (slowest.max(fastest) / fastest).powf(roll)
}

/// Returns the Love number of a body of `radius` metres and surface `gravity`: how far a tide
/// deforms it, `1.5 / (1 + 19 mu / (2 rho g R))` with the rigidity of rock for `mu`.
fn love(radius: f64, gravity: f64) -> f64 {
    let density = 3.0 * gravity / (4.0 * PI * GRAVITATION * radius);
    1.5 / (1.0 + 19.0 * RIGIDITY / (2.0 * density * gravity * radius))
}

/// A body as tides see it.
#[derive(Debug, Clone, Copy, PartialEq)]
pub(super) struct Turning {
    /// Radius in metres.
    pub radius: f64,
    /// Surface gravity in metres per second squared.
    pub gravity: f64,
    /// Seconds per turn it was born with.
    pub day: f64,
    /// Tidal quality factor, see [`QUALITY`].
    pub quality: f64,
}

/// Returns the seconds tides take to lock the spin of `body` to an orbit of `orbit` metres
/// around a mass of gravitational parameter `mu`.
///
/// This is `w a^6 I Q / (3 G M^2 k2 R^5)` (Gladman et al. 1996, "Synchronous Locking of Tidally
/// Evolving Satellites", Icarus 122, <https://doi.org/10.1006/icar.1996.0117>), with the moment
/// of inertia `I = 0.33 m R^2`: 30 billion years for the Earth, 400 million for Mercury.
pub(super) fn lock_seconds(body: &Turning, mu: f64, orbit: f64) -> f64 {
    let turn = TAU / body.day;
    let top = turn * orbit.powi(6) * INERTIA * body.gravity * body.quality;
    top / (3.0 * mu * mu * love(body.radius, body.gravity) * body.radius)
}

/// Returns the day in seconds of `body` on an orbit of `orbit` metres that takes `year`
/// seconds around a mass of parameter `mu`, after `age` seconds of tides: the year once the
/// tides have had time to lock it, else the day it was born with.
pub(super) fn settled(body: &Turning, (mu, orbit, year): (f64, f64, f64), age: f64) -> f64 {
    match lock_seconds(body, mu, orbit) < age {
        true => year,
        false => body.day,
    }
}
