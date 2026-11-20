//! Throw: a spear in flight, from the arm to the target.

use super::inertia::{balance, mass};
use super::tune::{ARM_MASS, DRAG, THROW_ENERGY};
use super::{Build, Place};

/// How a build flies when thrown.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Throw {
    /// Speed leaving the hand, in metres per second.
    pub launch: f32,
    /// Balance point as a share of the length from the butt, 0 to 1.
    pub balance: f32,
    /// True when the balance lies ahead of the middle, so the spear flies point first.
    pub stable: bool,
    /// Drag per metre of flight, ρ·C_d·A / 2m, in one per metre: speed falls as e^(−drag·d).
    pub drag: f32,
    /// Farthest throw in vacuum at 45°, v² / g, in metres.
    pub range: f32,
    /// Mass in kilograms.
    pub mass: f32,
}

impl Throw {
    /// Returns the speed after `distance` metres of flight, in metres per second.
    pub fn speed_at(&self, distance: f32) -> f32 {
        self.launch * (-self.drag * distance).exp()
    }

    /// Returns the kinetic energy on arrival after `distance` metres, in joules.
    pub fn energy_at(&self, distance: f32) -> f32 {
        let v = self.speed_at(distance);
        0.5 * self.mass * v * v
    }
}

/// Returns the throw of `build` at `place`.
///
/// The arm puts a fixed work into the spear and itself. A spear flies point first when its centre of
/// mass lies ahead of its centre of pressure, which sits near the middle of a plain shaft; drag is on
/// the widest cross section, the shaft or the head.
pub fn throw(build: &Build, place: &Place) -> Throw {
    let pieces = build.pieces();
    let mass = mass(&pieces);
    let launch = (2.0 * THROW_ENERGY / (mass + ARM_MASS)).sqrt();
    let length = build.grip + build.reach();
    let share = (balance(&pieces) + build.grip) / length.max(f32::EPSILON);
    let wide = build.shaft.diameter.max(build.head.width());
    let area = std::f32::consts::PI * wide * wide / 4.0;
    Throw {
        launch,
        balance: share,
        stable: share > 0.5,
        drag: place.air * DRAG * area / (2.0 * mass.max(f32::EPSILON)),
        range: launch * launch / place.gravity.max(f32::EPSILON),
        mass,
    }
}
