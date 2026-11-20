//! The robot as an animal that fights it sees it.

use bevy::math::DVec3;
use bevy::prelude::*;

use crate::fauna::beast::Beast;
use crate::space::Anchor;

/// The robot an animal fights: its place in the frame of the animal and in render space.
#[derive(Clone, Copy)]
pub struct Foe {
    /// Its place in the frame of the animal.
    pub local: Vec3,
    /// Its place in render space.
    pub render: Vec3,
    /// Its local up in render space.
    pub up: Vec3,
    /// The unit way from the animal to it along the ground, in render space: the way its blows push.
    pub away: Vec3,
}

/// Returns the robot at `robot`, whose place in the frame of the anchor is `here`, as the animal `beast`
/// sees it.
pub fn foe_of(anchor: &Anchor, beast: &Beast, (robot, here): (&Transform, DVec3)) -> Foe {
    let local = beast.spot.local(here);
    let flat = (local - beast.motion.root.at)
        .with_y(0.0)
        .normalize_or_zero();
    Foe {
        local,
        render: robot.translation,
        up: robot.up().as_vec3(),
        away: anchor.rotation_to_render(beast.spot.rot) * flat,
    }
}
