//! Shared test setups of the javelin: a plane of ground at any slope and a javelin in flight.

use bevy::prelude::*;

use super::super::flight::{Rest, Spear};
use super::super::stage::spear;
use super::ground::{Ground, Soil};

/// A plane of ground through the origin.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Slope {
    /// Its unit normal, pointing up.
    pub normal: Vec3,
    /// What it is made of.
    pub soil: Soil,
}

impl Slope {
    /// Returns ground of `soil` rising `angle` radians toward `toward`, a unit way along the level.
    pub fn rising(angle: f32, toward: Vec3, soil: Soil) -> Self {
        let tilt = Quat::from_axis_angle(Vec3::Y.cross(toward).normalize_or(Vec3::X), -angle);
        Self {
            normal: tilt * Vec3::Y,
            soil,
        }
    }

    /// Returns the height of the plane under the level position of `at`.
    fn height(&self, at: Vec3) -> f32 {
        -(self.normal.x * at.x + self.normal.z * at.z) / self.normal.y
    }
}

impl Ground for Slope {
    /// Returns the height of `at` over the plane, straight up.
    fn clearance(&self, at: Vec3) -> f32 {
        at.y - self.height(at)
    }

    /// Returns the plane straight over or under `at`.
    fn lift(&self, at: Vec3) -> Vec3 {
        at.with_y(self.height(at))
    }

    /// Returns the normal of the plane.
    fn normal(&self, _: Vec3) -> Vec3 {
        self.normal
    }

    /// Returns what the plane is made of.
    fn soil(&self, _: Vec3) -> Soil {
        self.soil
    }
}

/// Returns the test javelin flying from `at` at `velocity`, balanced to fly point first.
pub fn flying(at: Vec3, velocity: Vec3) -> Spear {
    let mut item = spear();
    item.throw.stable = true;
    Spear {
        item,
        velocity,
        rest: Rest::Flying,
        thrower: Entity::PLACEHOLDER,
        body: Entity::PLACEHOLDER,
        was: at,
        lent: None,
    }
}
