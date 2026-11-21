//! Air: an exponential atmosphere that slows the ship and turns its nose into the airflow.

use bevy::math::{DQuat, DVec3};

use super::deck::{DEEPEST, giant};
use super::tune::MAIN_ACCEL;
use crate::universe::PlanetSpec;
use crate::wind::thickness;

/// Air density at sea level in kilograms per cubic metre.
pub const SEA_DENSITY: f64 = 1.2;

/// Drag area over twice the mass along the body axes: sideways, vertical and nose first, in
/// square metres per kilogram. Nose first it is a drag area of 9.75 m² on the 15 t ship, a
/// quarter of the 39 m² it had before 0.11.0: the dart is clean along its nose, so its three
/// gravities of thrust hold 275 m/s in air at sea level, twice the 137 m/s it held. Across the
/// nose the wings and the flat belly keep the drag high, which holds the path on the nose.
pub const DRAG: DVec3 = DVec3::new(6.0e-3, 9.0e-3, 3.25e-4);

/// Angular acceleration per pascal of dynamic pressure that turns a ship flying sideways into the airflow.
const STABILITY: f64 = 2.5e-4;

/// Largest angular acceleration the airflow gives, in radians per second squared.
const STABILITY_LIMIT: f64 = 4.0;

/// Scale heights under the top of the air over which its density eases on to zero.
pub const TOP_FADE: f64 = 2.0;

/// Spin damping per unit of density times speed.
const SPIN_DRAG: f64 = 0.0125;

/// Returns the air density in kilograms per cubic metre `altitude` metres above the sea of a planet.
///
/// `density = rho0 * thickness * exp(-altitude / scale_height)` below the top of the atmosphere,
/// zero above it. A real air has no top, so over the last [`TOP_FADE`] scale heights under it
/// the density eases on to zero, and nothing that reads it, the drag, the heat of an entry or
/// the wind, jumps as a ship crosses the top. The scale height is the physical one of the planet, see
/// [`PlanetSpec::scale_height`], and the thickness is the density of its air at sea level
/// against the home planet, so thin air drags less, thick air more, and a planet without air
/// has no density at any height.
/// Under the sea level of a solid planet the density stays the one at sea level. Under the
/// cloud deck of a gas giant it keeps growing, down to [`DEEPEST`] scale heights.
pub fn density(planet: &PlanetSpec, altitude: f64) -> f64 {
    if altitude >= planet.atmosphere_height {
        return 0.0;
    }
    let scale = planet.scale_height();
    let low = if giant(planet) { -DEEPEST * scale } else { 0.0 };
    let under = ((planet.atmosphere_height - altitude) / (TOP_FADE * scale)).clamp(0.0, 1.0);
    let fade = under * under * (3.0 - 2.0 * under);
    SEA_DENSITY * thickness(&planet.record) * (-altitude.max(low) / scale).exp() * fade
}

/// Returns the dynamic pressure in pascals at a density and an airspeed.
pub fn pressure(density: f64, speed: f64) -> f64 {
    0.5 * density * speed * speed
}

/// Returns the fastest speed in metres per second the main engine holds nose first against the
/// drag of air of `density` kilograms per cubic metre, at `boost` times its thrust: where the
/// drag `DRAG.z * density * v^2` equals the thrust. Without air there is no such limit.
pub fn terminal(density: f64, boost: f64) -> f64 {
    (MAIN_ACCEL * boost / (DRAG.z * density.max(0.0))).sqrt()
}

/// Returns the velocity after `h` seconds of drag in air of `density`, for a ship with orientation `rot`.
///
/// Each body axis has its own drag, so a ship flying sideways is pushed back toward its nose.
/// The update is implicit and cannot overshoot at any speed.
pub fn slow(rot: DQuat, vel: DVec3, density: f64, h: f64) -> DVec3 {
    let rate = density * vel.length() * h;
    rot * ((rot.inverse() * vel) / (DVec3::ONE + DRAG * rate))
}

/// Returns the angular acceleration around the body axes that air of `density` gives a ship.
///
/// The nose turns toward the direction of flight and spin is damped. There is no lift.
pub fn stability(rot: DQuat, vel: DVec3, spin: DVec3, density: f64) -> DVec3 {
    let speed = vel.length();
    if density <= 0.0 || speed < 1e-3 {
        return DVec3::ZERO;
    }
    let flow = rot.inverse() * vel / speed;
    let turn = DVec3::NEG_Z.cross(flow) * (STABILITY * pressure(density, speed));
    turn.clamp_length_max(STABILITY_LIMIT) - spin * (SPIN_DRAG * density * speed)
}
