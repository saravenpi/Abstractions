//! Newtonian flight: one fixed step of gravity, thrust and rotation, in whatever frame the ship is in.

use bevy::math::{DQuat, DVec3};

use super::assist::{LIFT_ACCEL, steer};
use super::atmo;
use super::body::Ship;
use super::wells::{Well, gravity};
use crate::space::Frame;
use crate::universe::PlanetSpec;

/// Length of one simulation step in seconds.
pub const STEP: f64 = 1.0 / 120.0;

/// Acceleration of the main engine at full throttle in metres per second squared.
pub const MAIN_ACCEL: f64 = 66.0;

/// How many times stronger the main engine is with the boost.
pub const BOOST: f64 = 3.0;

/// What the pilot asks of the ship during one step.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct Controls {
    /// Stick from -1 to 1 around the body axes: pitch up, yaw left, roll left.
    pub turn: DVec3,
    /// Vertical thrusters from -1 (down) to 1 (up).
    pub lift: f64,
    /// Whether the boost is held.
    pub boost: bool,
}

/// What surrounds the ship during one step.
pub struct Scene<'a> {
    /// Planet whose surface frame the ship is in, when it is in one.
    pub planet: Option<&'a PlanetSpec>,
    /// Terrain height in metres above sea level at a position in the frame of that planet.
    pub ground: &'a dyn Fn(DVec3) -> f64,
    /// Bodies of the current system in the frame of its star.
    pub wells: &'a [Well],
    /// Seconds since the positions of the wells were sampled.
    pub elapsed: f64,
}

/// What one step did to the ship, for the instruments.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Report {
    /// Acceleration the crew feels, in metres per second squared: everything except gravity.
    pub proper: DVec3,
    /// Speed of a new touchdown in metres per second, zero without one.
    pub impact: f64,
    /// Dynamic pressure of the air in pascals.
    pub pressure: f64,
    /// Height of the gear above the ground or the sea in metres, infinite away from a surface.
    pub radar: f64,
}

impl Default for Report {
    fn default() -> Self {
        Self {
            proper: DVec3::ZERO,
            impact: 0.0,
            pressure: 0.0,
            radar: f64::INFINITY,
        }
    }
}

/// Returns the acceleration of the main engine in body axes.
pub fn main_engine(ship: &Ship, controls: &Controls) -> DVec3 {
    let boost = if controls.boost { BOOST } else { 1.0 };
    DVec3::NEG_Z * (ship.throttle * MAIN_ACCEL * boost)
}

/// Turns the ship for `h` seconds: attitude thrusters, then `torque` around the body axes.
pub fn rotate(ship: &mut Ship, stick: DVec3, torque: DVec3, h: f64) {
    ship.spin = steer(ship.spin, stick, ship.assist, h) + torque * h;
    ship.rot = (ship.rot * DQuat::from_scaled_axis(ship.spin * h)).normalize();
}

/// Advances a ship in a star frame by `h` seconds with semi-implicit Euler: velocity first, then position.
fn orbit(ship: &mut Ship, controls: &Controls, scene: &Scene, h: f64) -> Report {
    let thrust = ship.rot * (main_engine(ship, controls) + DVec3::Y * (controls.lift * LIFT_ACCEL));
    ship.vel += (gravity(scene.wells, ship.pos, scene.elapsed) + thrust) * h;
    ship.pos += ship.vel * h;
    rotate(ship, controls.turn, DVec3::ZERO, h);
    Report {
        proper: thrust,
        ..Report::default()
    }
}

/// Advances the ship by one step of `h` seconds.
///
/// In a surface frame the planet pulls, the air drags and the ground holds. In a star frame every
/// body of the system pulls. Between the stars the ship coasts. Frames are switched by the caller.
pub fn step(ship: &mut Ship, controls: &Controls, scene: &Scene, h: f64) -> Report {
    match (ship.frame, scene.planet) {
        (Frame::Surface(_), Some(planet)) => atmo::step(ship, controls, (planet, scene.ground), h),
        (Frame::System(_), _) => orbit(ship, controls, scene, h),
        _ => orbit(
            ship,
            controls,
            &Scene {
                wells: &[],
                ..*scene
            },
            h,
        ),
    }
}
