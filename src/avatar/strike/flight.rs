//! Flight: a thrown spear under the gravity of the planet and the drag of its air, and what happens
//! when it arrives.

use bevy::prelude::*;

use super::held::Held;
use super::javelin::tune::{SKID, TUMBLE, VANE};
use super::tune::RESTITUTION;
use crate::craft::Lent;
use crate::craft::build::{Blow, reference, strike};

/// Where a thrown spear is.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Rest {
    /// In the air.
    Flying,
    /// Stuck in the ground or in an animal.
    Stuck,
    /// Lying where it bounced to a stop.
    Lying,
}

/// A thrown spear, its origin at the point. It carries the item, to give it back when picked up.
#[derive(Component, Debug, Clone, PartialEq)]
pub struct Spear {
    /// The item thrown.
    pub item: Held,
    /// Velocity in render space, in metres per second.
    pub velocity: Vec3,
    /// Where it is.
    pub rest: Rest,
    /// The player body that threw it, which it does not hit.
    pub thrower: Entity,
    /// The body root that threw it, named in the blows it lands.
    pub body: Entity,
    /// Where its point was before the last step, in render space.
    pub was: Vec3,
    /// The item in the pack's terms, taken out of the pack while the spear flies and put back when
    /// it is picked up.
    pub lent: Option<Lent>,
}

/// Returns the point and the velocity `dt` seconds after `at` and `velocity` under `gravity`, with
/// a drag of `drag` per metre: the speed falls as e^(−drag·distance). The point moves by the mean
/// of the two velocities, which follows the parabola of a throw in vacuum exactly.
pub fn fly(at: Vec3, velocity: Vec3, gravity: Vec3, drag: f32, dt: f32) -> (Vec3, Vec3) {
    let next = velocity + gravity * dt;
    let next = next * (-drag * next.length() * dt).exp();
    (at + (velocity + next) * 0.5 * dt, next)
}

/// Returns the shaft `axis` of a spear turned over `dt` seconds by the air it flies through at
/// `velocity`: one balanced ahead of its middle (`stable`) noses over along its path, its axis
/// closing on the path over [`VANE`] metres of flight; one balanced behind tumbles end over end.
pub fn vane(axis: Vec3, velocity: Vec3, stable: bool, dt: f32) -> Vec3 {
    let speed = velocity.length();
    if speed < f32::EPSILON {
        return axis;
    }
    let way = velocity / speed;
    if !stable {
        let spin = way.cross(axis).normalize_or(way.any_orthonormal_vector());
        return (Quat::from_axis_angle(spin, TUMBLE * dt) * axis).normalize();
    }
    let share = 1.0 - (-speed * dt / VANE).exp();
    let turn = Quat::IDENTITY.slerp(Quat::from_rotation_arc(axis, way), share);
    (turn * axis).normalize()
}

/// Returns how deep a point arriving with `energy` joules over `area` square metres drives into
/// ground that holds `pressure` pascals against it, in metres: the work over the force.
pub fn depth(energy: f32, area: f32, pressure: f32) -> f32 {
    energy / (pressure * area).max(1e-6)
}

/// Returns true when `blow` gets through the first layer of the reference target: a spear that
/// does stays in the animal.
pub fn pierces(blow: &Blow) -> bool {
    strike(blow, &reference()).passed > 0
}

/// Returns `velocity` glanced off a surface of unit `normal`: [`RESTITUTION`] of the speed into it
/// comes back out, [`SKID`] of the speed along it is kept.
pub fn glance(velocity: Vec3, normal: Vec3) -> Vec3 {
    let into = normal * velocity.dot(normal);
    (velocity - into) * SKID - into * RESTITUTION
}
