//! What the game camera shows on the rig: the whole hull in the frame, and the ground below.

use bevy::prelude::*;

use super::Game;
use crate::ship::{HULL_RADIUS, Ship};
use crate::space::{Anchor, Placed};
use crate::view::CameraPose;
use crate::world::Planet;

/// Width over height of the screen the frame is checked on.
const ASPECT: f32 = 16.0 / 9.0;

/// Returns true when the sphere of `radius` around the render point `centre` is wholly inside
/// the view of `pose` on a screen of [`ASPECT`].
fn in_view(pose: &CameraPose, centre: Vec3, radius: f32) -> bool {
    let eye = pose.rotation.inverse() * (centre - pose.position);
    let tall = 0.5 * pose.fov;
    let wide = (tall.tan() * ASPECT).atan();
    let room = |half: f32, across: f32| -eye.z * half.sin() - across.abs() * half.cos();
    room(tall, eye.y) >= radius && room(wide, eye.x) >= radius
}

impl Game {
    /// Returns the render position of the centre of the hull.
    pub fn hull(&mut self) -> Vec3 {
        let anchor = self.get::<Anchor>();
        let world = self.app.world_mut();
        let mut ships = world.query_filtered::<&Placed, With<Ship>>();
        anchor.to_render(ships.single(world).expect("one ship").pos)
    }

    /// Returns true when the whole hull is in the view of the game camera.
    pub fn hull_in_view(&mut self) -> bool {
        let centre = self.hull();
        in_view(&self.get::<CameraPose>(), centre, HULL_RADIUS)
    }

    /// Returns the height of the game camera above the ground the planet generates, in metres.
    pub fn camera_height(&mut self) -> f64 {
        let (anchor, planet) = (self.get::<Anchor>(), self.get::<Planet>());
        let at = anchor.to_frame(self.get::<CameraPose>().position);
        at.length() - planet.globe.radius() - f64::from(planet.globe.height(at.normalize()))
    }
}
