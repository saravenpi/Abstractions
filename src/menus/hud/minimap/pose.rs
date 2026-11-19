//! Where the player is for the minimap: on which planet, at which point of it, looking which
//! way and facing which way, or where in space.
//!
//! The view and the body can look apart: in third person the camera orbits a body that faces
//! where it walks. The ground view turns with the view, and the arrow in its middle turns by
//! what lies between the two.

use std::sync::Arc;

use bevy::ecs::system::SystemParam;
use bevy::math::DVec3;
use bevy::prelude::*;

use crate::menus::hud::eye::{Eye, HudPose};
use crate::ship::Ship;
use crate::space::{Frame, Location, Travel};
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
    /// Clockwise turn in radians from the way the view looks to the way the body faces, or
    /// the nose of the ship while seated in it.
    pub needle: f32,
}

/// What the minimap reads of the world.
#[derive(SystemParam)]
pub struct Reader<'w, 's> {
    /// The eye of the HUD for this frame.
    eye: Res<'w, HudPose>,
    /// Where the player is.
    location: Option<Res<'w, Location>>,
    /// The planet the world streams.
    planet: Option<Res<'w, Planet>>,
    /// The ship.
    ships: Query<'w, 's, &'static Ship>,
}

/// Returns `along` without its part along the unit vector `up`, or nothing when none is left.
fn level(along: DVec3, up: DVec3) -> Option<DVec3> {
    (along - up * along.dot(up)).try_normalize()
}

/// Returns the clockwise turn in radians, seen from above along `up`, from the direction
/// `ahead` to the direction `facing`, both level.
pub fn needle(ahead: DVec3, facing: DVec3, up: DVec3) -> f32 {
    -up.dot(ahead.cross(facing)).atan2(ahead.dot(facing)) as f32
}

impl Reader<'_, '_> {
    /// Returns the eye of the HUD for this frame.
    pub fn eye(&self) -> Option<&Eye> {
        self.eye.0.as_ref()
    }

    /// Returns the nose and the top of what the player steers, in the frame of the world: the
    /// ship while seated in it, else the body, else the camera.
    pub fn facing(&self) -> Option<(DVec3, DVec3)> {
        let eye = self.eye()?;
        let seated = self.location.as_deref().map(|at| at.travel) == Some(Travel::InShip);
        let ship = self
            .ships
            .iter()
            .find(|ship| ship.frame == eye.anchor.frame);
        if let Some(ship) = ship.filter(|_| seated) {
            return Some((ship.rot * DVec3::NEG_Z, ship.rot * DVec3::Y));
        }
        Some(eye.body.unwrap_or((eye.forward, eye.up)))
    }

    /// Returns the frame the world is anchored in and the camera position in it.
    pub fn place(&self) -> Option<(Frame, DVec3)> {
        self.eye().map(|eye| (eye.anchor.frame, eye.at))
    }

    /// Returns the globe of the planet the world is anchored to, once the world streams it.
    pub fn globe(&self) -> Option<&Arc<Globe>> {
        let (eye, planet) = (self.eye()?, self.planet.as_deref()?);
        (eye.anchor.frame == Frame::Surface(planet.body)).then_some(&planet.globe)
    }

    /// Returns the player on the planet the world is anchored to, or nothing in space. A
    /// camera that looks straight down gives the direction of its top edge.
    pub fn pose(&self) -> Option<Pose> {
        let eye = self.eye()?;
        let Frame::Surface(body) = eye.anchor.frame else {
            return None;
        };
        let ship = self
            .ships
            .iter()
            .find(|ship| ship.frame == eye.anchor.frame);
        let seated = self.location.as_deref().map(|at| at.travel) == Some(Travel::InShip);
        let at = ship.filter(|_| seated).map_or(eye.at, |ship| ship.pos);
        let up = at.try_normalize().filter(|_| !eye.anchor.flat)?;
        let ahead = level(eye.forward, up).or_else(|| level(eye.up, up))?;
        let nose = self.facing().and_then(|(nose, _)| level(nose, up));
        Some(Pose {
            body,
            at,
            ahead,
            ship: ship.filter(|_| !seated).map(|ship| ship.pos),
            needle: nose.map_or(0.0, |nose| needle(ahead, nose, up)),
        })
    }
}
