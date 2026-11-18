//! Where the player is for the minimap: on which planet, at which point of it and looking
//! which way, or where in space.

use std::sync::Arc;

use bevy::ecs::system::SystemParam;
use bevy::math::DVec3;
use bevy::prelude::*;

use crate::render::GameCamera;
use crate::ship::Ship;
use crate::space::{Anchor, Frame, Location, Travel};
use crate::universe::BodyId;
use crate::world::{Globe, Planet};

/// The player on a planet, in the frame of the planet: metres from its centre.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Pose {
    /// The planet.
    pub body: BodyId,
    /// Where the player is: the ship while seated in it, else the camera.
    pub at: DVec3,
    /// The horizontal direction the camera looks along.
    pub ahead: DVec3,
    /// The ship, while the player is on foot and it stands on this planet.
    pub ship: Option<DVec3>,
}

/// What the minimap reads of the world.
#[derive(SystemParam)]
pub struct Reader<'w, 's> {
    /// The render anchor.
    anchor: Option<Res<'w, Anchor>>,
    /// Where the player is.
    location: Option<Res<'w, Location>>,
    /// The planet the world streams.
    planet: Option<Res<'w, Planet>>,
    /// The ship.
    ships: Query<'w, 's, &'static Ship>,
    /// The game camera.
    camera: Query<'w, 's, &'static GlobalTransform, With<GameCamera>>,
}

/// Returns `along` without its part along the unit vector `up`, or nothing when none is left.
fn level(along: DVec3, up: DVec3) -> Option<DVec3> {
    (along - up * along.dot(up)).try_normalize()
}

impl Reader<'_, '_> {
    /// Returns the frame the world is anchored in and the camera position in it.
    pub fn place(&self) -> Option<(Frame, DVec3)> {
        let (anchor, camera) = (self.anchor.as_deref()?, self.camera.iter().next()?);
        Some((anchor.frame, anchor.to_frame(camera.translation())))
    }

    /// Returns the globe of the planet the world is anchored to, once the world streams it.
    pub fn globe(&self) -> Option<&Arc<Globe>> {
        let (anchor, planet) = (self.anchor.as_deref()?, self.planet.as_deref()?);
        (anchor.frame == Frame::Surface(planet.body)).then_some(&planet.globe)
    }

    /// Returns the player on the planet the world is anchored to, or nothing in space. A
    /// camera that looks straight down gives the direction of its top edge.
    pub fn pose(&self) -> Option<Pose> {
        let (anchor, camera) = (self.anchor.as_deref()?, self.camera.iter().next()?);
        let Frame::Surface(body) = anchor.frame else {
            return None;
        };
        let eye = anchor.to_frame(camera.translation());
        let ship = self.ships.iter().find(|ship| ship.frame == anchor.frame);
        let seated = self.location.as_deref().map(|at| at.travel) == Some(Travel::InShip);
        let at = ship.filter(|_| seated).map_or(eye, |ship| ship.pos);
        let up = at.try_normalize().filter(|_| !anchor.flat)?;
        let turned = |axis: Dir3| anchor.rotation.inverse() * axis.as_dvec3();
        let ahead =
            level(turned(camera.forward()), up).or_else(|| level(turned(camera.up()), up))?;
        Some(Pose {
            body,
            at,
            ahead,
            ship: ship.filter(|_| !seated).map(|ship| ship.pos),
        })
    }
}
