//! What catches the companion's eye, read from the world: the ship, the home base, a structure,
//! and where the player and the camera are. A few distance checks per frame, nothing more.

use bevy::ecs::system::SystemParam;
use bevy::prelude::*;

use super::motion::Float;
use super::scene::{Scene, course};
use super::sense::{Around, CompanionSense};
use super::tune::{CURIOUS_REACH, SIGHT_REACH};
use crate::home::Base;
use crate::ship::Ship;
use crate::space::Frame;

/// The things around the player the companion may look at.
#[derive(SystemParam)]
pub struct Sights<'w, 's> {
    /// The player's ship.
    ship: Query<'w, 's, &'static Ship>,
    /// The home base, once it is placed.
    base: Option<Res<'w, Base>>,
    /// What the companion senses, with the nearest structure.
    sense: Res<'w, CompanionSense>,
}

/// Returns `pos` when it lies within `reach` meters of `here`.
fn within(pos: Option<Vec3>, here: Vec3, reach: f32) -> Option<Vec3> {
    pos.filter(|pos| pos.distance(here) < reach)
}

impl Sights<'_, '_> {
    /// Returns the render positions of the ship, the door of the home base and the nearest
    /// structure, each only when it is close to `here` and the player is not seated.
    fn sights(&self, around: &Around, here: Vec3) -> [Option<Vec3>; 3] {
        let anchor = &*around.anchor;
        let ship = self
            .ship
            .iter()
            .next()
            .filter(|ship| ship.frame == anchor.frame);
        let ship = ship.map(|ship| anchor.to_render(ship.pos));
        let base = self.base.as_deref();
        let home = base.filter(|base| anchor.frame == Frame::Surface(base.body));
        let door = home.map(|base| anchor.to_render(base.door));
        let free = !around.seated();
        [
            within(ship.filter(|_| free), here, SIGHT_REACH.0),
            within(door.filter(|_| free), here, SIGHT_REACH.1),
            within(self.sense.structure.filter(|_| free), here, CURIOUS_REACH),
        ]
    }

    /// Returns what the companion at the pose `at` sees, moving as `float` tells, beside a
    /// player moving at `drift`. Seated, its pose is in the space of the pilot seat.
    pub fn scene(&self, around: &Around, at: &Transform, float: &Float, drift: Vec3) -> Scene {
        let stance = around.stance();
        let seat = *around.body;
        let (here, body) = match float.seated {
            true => (
                seat.transform_point(at.translation),
                seat.rotation * at.rotation,
            ),
            false => (at.translation, at.rotation),
        };
        let camera = *around.camera;
        Scene {
            here,
            heading: body * float.neck.rotation() * Vec3::NEG_Z,
            eye: stance.eye,
            up: stance.up,
            drift,
            course: match float.seated {
                true => Some(stance.forward),
                false => course(drift, stance.up),
            },
            camera: camera.translation(),
            view: camera.forward().as_vec3(),
            sights: self.sights(around, here),
        }
    }
}
