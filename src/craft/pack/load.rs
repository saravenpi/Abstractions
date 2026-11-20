//! Load: what the robot's frame lifts under the local gravity, and how much a load slows its walk.

use bevy::prelude::*;

use super::tune::{CARRY, HOME_GRAVITY, SLOW_FROM, SLOWEST};

/// Returns the mass the robot carries under `gravity` metres per second squared, in kilograms: the
/// frame lifts the same weight everywhere, so the mass falls as 9.81 / g.
pub fn capacity(gravity: f32) -> f32 {
    CARRY * HOME_GRAVITY / gravity.max(0.1)
}

/// Returns the share of its walking speed a robot keeps carrying `load` of `capacity` kilograms: all
/// of it up to 60 % of the capacity, then less, down to half at a full load.
pub fn pace(load: f32, capacity: f32) -> f32 {
    let share = load / capacity.max(f32::EPSILON);
    let over = ((share - SLOW_FROM) / (1.0 - SLOW_FROM)).clamp(0.0, 1.0);
    1.0 - over * (1.0 - SLOWEST)
}

/// What the pack weighs here. The player controller reads [`Burden::pace`] to slow the walk.
#[derive(Resource, Debug, Clone, Copy, PartialEq)]
pub struct Burden {
    /// Mass carried, in kilograms.
    pub load: f32,
    /// Most mass the robot carries here, in kilograms.
    pub capacity: f32,
    /// Share of the walking speed left, 0 to 1.
    pub pace: f32,
}

impl Default for Burden {
    /// Returns an empty pack under home gravity.
    fn default() -> Self {
        Self::of(0.0, HOME_GRAVITY)
    }
}

impl Burden {
    /// Returns the burden of `load` kilograms under `gravity` metres per second squared.
    pub fn of(load: f32, gravity: f32) -> Self {
        let capacity = capacity(gravity);
        Self {
            load,
            capacity,
            pace: pace(load, capacity),
        }
    }

    /// Returns the mass still free, in kilograms.
    pub fn room(&self) -> f32 {
        (self.capacity - self.load).max(0.0)
    }
}
