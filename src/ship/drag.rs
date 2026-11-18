//! Air: an exponential atmosphere that slows the ship and turns its nose into the airflow.

use bevy::math::{DQuat, DVec3};

use crate::universe::PlanetSpec;
use crate::wind::thickness;

/// Air density at sea level in kilograms per cubic metre.
pub const SEA_DENSITY: f64 = 1.2;

/// Share of the atmosphere height that makes one scale height.
const SCALE_SHARE: f64 = 0.125;

/// Drag area over twice the mass along the body axes: sideways, vertical and nose first.
pub const DRAG: DVec3 = DVec3::new(6.0e-3, 9.0e-3, 1.3e-3);

/// Angular acceleration per pascal of dynamic pressure that turns a ship flying sideways into the airflow.
const STABILITY: f64 = 2.5e-4;

/// Largest angular acceleration the airflow gives, in radians per second squared.
const STABILITY_LIMIT: f64 = 4.0;

/// Spin damping per unit of density times speed.
const SPIN_DRAG: f64 = 0.0125;

/// Returns the air density in kilograms per cubic metre `altitude` metres above the sea of a planet.
///
/// `density = rho0 * thickness * exp(-altitude / scale_height)` below the top of the atmosphere,
/// zero above it. The thickness is the pressure of the planet against the home planet, so thin
/// air drags less, thick air more, and a planet without air has no density at any height.
pub fn density(planet: &PlanetSpec, altitude: f64) -> f64 {
    if altitude >= planet.atmosphere_height {
        return 0.0;
    }
    let scale = planet.atmosphere_height * SCALE_SHARE;
    SEA_DENSITY * thickness(&planet.record) * (-altitude.max(0.0) / scale).exp()
}

/// Returns the dynamic pressure in pascals at a density and an airspeed.
pub fn pressure(density: f64, speed: f64) -> f64 {
    0.5 * density * speed * speed
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
