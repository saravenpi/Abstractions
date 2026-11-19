//! Drawing the other players: each record track sampled a little in the past, moved from its
//! wire frame into the frame of the anchor at universe time, and written into its puppet.
//!
//! Only the things on the body this player is at, or flying in its system, get a puppet; the
//! rest keep their buffer only. A puppet beyond [`HIDE`] metres is hidden.

mod companion;
#[cfg(test)]
mod companion_tests;
mod robot;
mod ship;

pub use companion::companions;
pub use robot::robots;
pub use ship::ships;

use abstractions_net::jitter::View;
use abstractions_net::wire;
use bevy::ecs::system::SystemParam;
use bevy::prelude::*;

use super::bodies::Bodies;
use super::frame::{Place, Pose};
use super::track::Track;
use crate::render::GameCamera;
use crate::space::{Anchor, Clock, Location, Placed};
use crate::universe::{BodyId, Universe};

/// Metres from the camera beyond which a puppet is hidden.
pub const HIDE: f32 = 600.0;

/// What drawing reads of the world.
#[derive(SystemParam)]
pub struct Sight<'w, 's> {
    /// The render anchor.
    anchor: Option<Res<'w, Anchor>>,
    /// Where the player is.
    location: Option<Res<'w, Location>>,
    /// The universe.
    universe: Option<Res<'w, Universe>>,
    /// Universe time.
    clock: Res<'w, Clock>,
    /// The bodies frames are converted against.
    bodies: ResMut<'w, Bodies>,
    /// The game camera.
    camera: Query<'w, 's, &'static GlobalTransform, With<GameCamera>>,
}

/// Where the puppets are drawn from this frame.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Eye {
    /// The render anchor.
    pub anchor: Anchor,
    /// The body the player is at.
    pub body: BodyId,
    /// Universe time of the frame.
    pub time: f64,
    /// Render position of the camera, the origin without one.
    pub camera: Vec3,
}

impl Sight<'_, '_> {
    /// Returns where the puppets are drawn from, once the world exists.
    pub fn eye(&mut self) -> Option<Eye> {
        let (anchor, location) = (*self.anchor.as_deref()?, self.location.as_deref()?);
        self.bodies.sync(self.universe.as_deref()?);
        let camera = self
            .camera
            .single()
            .map_or(Vec3::ZERO, |at| at.translation());
        Some(Eye {
            anchor,
            body: location.body,
            time: self.clock.seconds,
            camera,
        })
    }

    /// Returns the state of `track` now and its pose in the anchor frame, or nothing while it
    /// is empty, is not drawn from here or cannot be placed.
    /// The sample is a little in the past; see [`moment`] for the time it is converted at.
    pub fn locate<E: Copy + Default>(
        &mut self,
        track: &mut Track<E>,
        eye: &Eye,
    ) -> Option<(View<E>, Placed)> {
        if !shown(track.frame, eye.body) {
            return None;
        }
        let view = track.buffer.sample(eye.time)?;
        let pose = Pose {
            pos: view.position,
            vel: bevy::math::DVec3::ZERO,
            rot: view.rotation.as_dquat(),
        };
        let (from, to) = (Place::from(track.frame), Place::from(eye.anchor.frame));
        let time = moment((from, to), view.time, eye.time);
        let pose = self.bodies.convert(pose, from, to, time)?;
        Some((
            view,
            Placed {
                pos: pose.pos,
                rot: pose.rot,
            },
        ))
    }
}

/// Returns the universe time a pose sampled at `sampled` is moved from the frame `from` into
/// the frame `to` at, `now` being the time of the frame drawn. Into a frame nearer the ground
/// than its own, at its own time, so a ship hovering over a turning planet stays over the same
/// ground. Into a frame farther out, now, so a thing on a planet or around it stays with the
/// planet, which is drawn where it is now.
pub fn moment((from, to): (Place, Place), sampled: f64, now: f64) -> f64 {
    match to.depth() < from.depth() {
        true => sampled,
        false => now,
    }
}

/// Returns whether a thing in `frame` is drawn by a player at `body`: on that body or around
/// it, or in the frame of its star.
pub fn shown(frame: wire::Frame, body: BodyId) -> bool {
    match frame {
        wire::Frame::Surface(at) | wire::Frame::Around(at) => at == body,
        wire::Frame::System(star) => star == body.star,
    }
}

/// Returns the render transform of `placed` and how far it is from the camera.
pub fn seen(eye: &Eye, placed: &Placed) -> (Transform, f32) {
    let at = Transform {
        translation: eye.anchor.to_render(placed.pos),
        rotation: eye.anchor.rotation_to_render(placed.rot),
        scale: Vec3::ONE,
    };
    (at, at.translation.distance(eye.camera))
}

/// Despawns the puppet and the name of `track`, which keeps its records.
pub fn drop_puppet<E>(commands: &mut Commands, track: &mut Track<E>) {
    for entity in [track.puppet.take(), track.tag.take()]
        .into_iter()
        .flatten()
    {
        commands.entity(entity).try_despawn();
    }
}

/// Returns the visibility of a puppet `far` metres away, or hidden when `hidden`.
pub fn visible(far: f32, hidden: bool) -> Visibility {
    match hidden || far > HIDE {
        true => Visibility::Hidden,
        false => Visibility::Inherited,
    }
}
