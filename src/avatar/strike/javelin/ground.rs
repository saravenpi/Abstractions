//! Ground: what a thrown javelin reads of the ground under it, and what the ground is made of.
//! Soft ground takes a point; rock turns it away.

use bevy::prelude::*;

use super::tune::SOILS;
use crate::world::Surface;

/// What a point meets in the ground.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Soil {
    /// Ground that gives way: the pressure it holds against a point driven in, in pascals.
    Soft(f32),
    /// Rock: a point glances off it.
    Hard,
}

impl Soil {
    /// Returns what the ground of `surface` is: earth, sand, mud and snow give way, rock and reef
    /// do not.
    pub fn of(surface: Surface) -> Self {
        let [earth, sand, mud, snow] = SOILS;
        match surface {
            Surface::Rock | Surface::Reef => Soil::Hard,
            Surface::Grass => Soil::Soft(earth),
            Surface::Sand | Surface::Shallows => Soil::Soft(sand),
            Surface::Pond | Surface::River | Surface::Water => Soil::Soft(mud),
            Surface::Snow => Soil::Soft(snow),
        }
    }
}

/// The ground under a flying javelin, as it reads it.
pub trait Ground {
    /// Returns how far the render position `at` is above the ground, straight up: negative under it.
    fn clearance(&self, at: Vec3) -> f32;
    /// Returns the point of the ground straight over or under `at`, in render space.
    fn lift(&self, at: Vec3) -> Vec3;
    /// Returns the unit normal of the ground under `at`, in render axes.
    fn normal(&self, at: Vec3) -> Vec3;
    /// Returns what the ground under `at` is.
    fn soil(&self, at: Vec3) -> Soil;
}
