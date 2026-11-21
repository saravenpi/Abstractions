//! The force of a blow on the robot: how fast a dash arrives, the mass it lands with and the energy
//! it leaves. The robot is a free body of [`ROBOT_KG`]: a heavy animal throws it more than it
//! crushes it.

use super::kinds::Kind;
use super::tune::{GRIP, REAR};
use crate::fauna::senses::tune::ROBOT_KG;

/// Returns the speed in metres per second at which a dash of `kind` meets its target after a run of
/// `run` metres from rest, by a body `length` metres long whose top speed is `top`, under `gravity`.
/// The feet push with at most [`GRIP`] of the weight, so the speed grows as the square root of the
/// run and a short charge is slow. A trample also drops the reared front of the body on the target.
pub fn contact(kind: Kind, (top, run): (f32, f32), (length, gravity): (f32, f32)) -> f32 {
    let drop = match kind {
        Kind::Trample => 0.5 * length * REAR[0].sin(),
        _ => 0.0,
    };
    let reached = 2.0 * gravity.max(0.0) * (GRIP * run.max(0.0) + drop.max(0.0));
    reached.sqrt().min(top)
}

/// Returns the mass in kilograms that a blow of `kind` from a body of `mass` kilograms lands with on
/// the robot. The robot stands free: a blow throws it, and only the reduced mass of the two bodies
/// deforms it, at most the mass of the robot itself. A trample lands with all its mass, because the
/// ground holds the robot under it.
pub fn meets(kind: Kind, mass: f32) -> f32 {
    let striker = (mass * kind.share()).max(0.0);
    match kind {
        Kind::Trample => striker,
        _ => striker * ROBOT_KG / (striker + ROBOT_KG),
    }
}

/// Returns the energy in joules that a blow of `kind` leaves in the robot, from a body of `mass`
/// kilograms whose striking part arrives at `speed` metres per second: the energy lost when the two
/// bodies meet and move on together. The rest stays motion: the robot is thrown.
pub fn joules(kind: Kind, mass: f32, speed: f32) -> f32 {
    0.5 * meets(kind, mass) * speed * speed
}
