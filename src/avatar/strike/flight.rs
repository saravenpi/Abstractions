//! Flight: a thrown spear under the gravity of the planet and the drag of its air, and what happens
//! when it arrives.

use bevy::prelude::*;

use super::held::Held;
use super::tune::{RESTITUTION, SOIL, STICK};
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
}

/// Returns the point and the velocity `dt` seconds after `at` and `velocity` under `gravity`, with
/// a drag of `drag` per metre: the speed falls as e^(−drag·distance) (semi-implicit Euler).
pub fn fly(at: Vec3, velocity: Vec3, gravity: Vec3, drag: f32, dt: f32) -> (Vec3, Vec3) {
    let velocity = velocity + gravity * dt;
    let velocity = velocity * (-drag * velocity.length() * dt).exp();
    (at + velocity * dt, velocity)
}

/// Returns how deep a point arriving with `energy` joules over `area` square metres drives into the
/// ground, in metres: the work over the pressure the soil holds against it.
pub fn depth(energy: f32, area: f32) -> f32 {
    energy / (SOIL * area.max(1e-6))
}

/// Returns true when a spear that flies point first (`stable`) arriving with `energy` joules over
/// `area` stands in the ground.
pub fn stands(stable: bool, energy: f32, area: f32) -> bool {
    stable && depth(energy, area) >= STICK
}

/// Returns true when `blow` gets through the first layer of the reference target: a spear that
/// does stays in the animal.
pub fn pierces(blow: &Blow) -> bool {
    strike(blow, &reference()).passed > 0
}

/// Returns `velocity` bounced off a surface of unit `normal`, keeping [`RESTITUTION`] of it.
pub fn bounce(velocity: Vec3, normal: Vec3) -> Vec3 {
    velocity.reflect(normal) * RESTITUTION
}
