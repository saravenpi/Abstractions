//! The point chunks stream around: the anchor entity, or the spawn point before it exists, seen
//! from the planet.

use std::f64::consts::TAU;

use bevy::ecs::system::SystemParam;
use bevy::math::{DQuat, DVec3};
use bevy::prelude::*;

use super::{Planet, SpawnPoint, StreamAnchor};
use crate::space::{Anchor, Clock, Frame};
use crate::universe::Universe;

/// Returns the rotation from the axes of a planet to the axes of its system at `time` seconds.
///
/// A planet turns around its own +Y, from west to east, once every `day_seconds`.
pub fn spin(day_seconds: f64, time: f64) -> DQuat {
    DQuat::from_rotation_y(TAU * time / day_seconds)
}

/// How the planet frame sits in the frame of the anchor: `anchor = offset + turn * planet`.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Link {
    /// Position of the planet centre in the anchor frame.
    pub offset: DVec3,
    /// Rotation from planet axes to the axes of the anchor frame.
    pub turn: DQuat,
}

impl Link {
    /// Returns the link from the frame of `planet` to `frame` at `time` seconds: the identity on
    /// the surface frame of the planet, its orbit and spin in the frame of its star, else nothing.
    pub fn between(frame: Frame, planet: &Planet, universe: &Universe, time: f64) -> Option<Self> {
        match frame {
            Frame::Surface(body) if body == planet.body => Some(Self {
                offset: DVec3::ZERO,
                turn: DQuat::IDENTITY,
            }),
            Frame::System(star) if star == planet.body.star => Some(Self {
                offset: universe.planet_position(planet.body, time),
                turn: spin(planet.day_seconds, time),
            }),
            _ => None,
        }
    }

    /// Returns the planet frame position of a position given in the anchor frame.
    pub fn to_planet(&self, pos: DVec3) -> DVec3 {
        self.turn.inverse() * (pos - self.offset)
    }

    /// Returns the anchor frame position of a position given in the planet frame.
    pub fn to_anchor(&self, pos: DVec3) -> DVec3 {
        self.offset + self.turn * pos
    }
}

/// Reads how the streamed planet sits against the render anchor.
#[derive(SystemParam)]
pub struct Frames<'w> {
    anchor: Option<Res<'w, Anchor>>,
    planet: Option<Res<'w, Planet>>,
    universe: Option<Res<'w, Universe>>,
    clock: Option<Res<'w, Clock>>,
}

impl Frames<'_> {
    /// Returns the anchor and the link from the planet frame to its frame, when the anchor is
    /// on the planet or in its system.
    pub fn link(&self) -> Option<(Anchor, Link)> {
        let (anchor, planet) = (self.anchor.as_ref()?, self.planet.as_ref()?);
        let time = self.clock.as_ref().map_or(0.0, |clock| clock.seconds);
        let link = Link::between(anchor.frame, planet, self.universe.as_ref()?, time)?;
        Some((**anchor, link))
    }
}

/// Reads the streaming focus.
#[derive(SystemParam)]
pub struct Focus<'w, 's> {
    anchor: Query<'w, 's, &'static Transform, With<StreamAnchor>>,
    spawn: Option<Res<'w, SpawnPoint>>,
    frames: Frames<'w>,
}

impl Focus<'_, '_> {
    /// Returns the focus position in render space.
    pub fn position(&self) -> Option<Vec3> {
        self.anchor
            .iter()
            .next()
            .map(|t| t.translation)
            .or(self.spawn.as_ref().map(|s| s.0))
    }

    /// Returns the focus position in metres from the planet centre, in the planet frame.
    pub fn planet(&self) -> Option<DVec3> {
        let (anchor, link) = self.frames.link()?;
        Some(link.to_planet(anchor.to_frame(self.position()?)))
    }
}
