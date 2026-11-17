//! Rebasing: keeping the surface anchor under the walking player and carrying free entities along.

use avian3d::prelude::*;
use bevy::ecs::query::QueryData;
use bevy::math::{DQuat, DVec3};
use bevy::prelude::*;

use super::{Anchor, Frame, Rebased};
use crate::player::Player;
use crate::world::Terrain;

#[cfg(test)]
mod tests;

/// Returns how far in metres the player may walk from the anchored spot of a planet of sea
/// level `radius` before the anchor is rebased: 1 percent of the radius, from 30 to 300 m.
pub fn rebase_limit(radius: f64) -> f64 {
    (radius * 0.01).clamp(30.0, 300.0)
}

/// Returns the anchor to replace `anchor` with for a body at render position `pos` on a planet
/// of sea level `radius`, or `None` while the body is within [`rebase_limit`] of the anchored spot.
///
/// The new anchor is the tangent frame at the sea level point under the body: that point renders
/// at the origin and its vertical is render +Y. The render axes turn by the smallest rotation
/// that makes it so, which keeps every heading. A flat anchor or one outside a surface frame is
/// never rebased.
pub fn rebased(anchor: &Anchor, radius: f64, pos: Vec3) -> Option<Anchor> {
    if anchor.flat || !matches!(anchor.frame, Frame::Surface(_)) {
        return None;
    }
    let dir = anchor.to_frame(pos).try_normalize()?;
    let (ground, up) = (dir * radius, anchor.rotation * dir);
    let limit = rebase_limit(radius);
    if (up - DVec3::Y).length() * radius <= limit && ground.distance(anchor.origin) <= limit {
        return None;
    }
    Some(Anchor {
        frame: anchor.frame,
        origin: ground,
        rotation: (DQuat::from_rotation_arc(up, DVec3::Y) * anchor.rotation).normalize(),
        flat: false,
    })
}

/// The rigid motion of render space caused by an anchor change: `new = rotation * old + translation`.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Shift {
    /// Rotation from the old render axes to the new ones.
    pub rotation: Quat,
    /// New render position of the old render origin.
    pub translation: Vec3,
}

impl Shift {
    /// Returns the motion that takes render space under `old` to render space under `new`.
    ///
    /// Both anchors must be expressed in the same frame.
    pub fn between(old: &Anchor, new: &Anchor) -> Self {
        Self {
            rotation: (new.rotation * old.rotation.inverse())
                .normalize()
                .as_quat(),
            translation: (new.rotation * (old.origin - new.origin)).as_vec3(),
        }
    }

    /// Returns the new render position of an old render position.
    pub fn point(&self, pos: Vec3) -> Vec3 {
        self.rotation * pos + self.translation
    }

    /// Returns the new render components of a vector, such as a velocity.
    pub fn direction(&self, vector: Vec3) -> Vec3 {
        self.rotation * vector
    }

    /// Returns the new render orientation of an old render orientation.
    pub fn turn(&self, rot: Quat) -> Quat {
        (self.rotation * rot).normalize()
    }
}

/// Everything of a free entity that is expressed in render space.
#[derive(QueryData)]
#[query_data(mutable)]
pub(super) struct Mover {
    /// Render pose, the truth for a free entity.
    transform: &'static mut Transform,
    /// Physics position, kept equal to the transform.
    position: Option<&'static mut Position>,
    /// Physics orientation, kept equal to the transform.
    rotation: Option<&'static mut Rotation>,
    /// Physics linear velocity.
    linear: Option<&'static mut LinearVelocity>,
    /// Physics angular velocity.
    angular: Option<&'static mut AngularVelocity>,
}

/// Moves one free entity and its physics state by `shift`.
fn carry(shift: &Shift, mover: &mut MoverItem) {
    mover.transform.translation = shift.point(mover.transform.translation);
    mover.transform.rotation = shift.turn(mover.transform.rotation);
    if let Some(position) = mover.position.as_mut() {
        position.0 = mover.transform.translation;
    }
    if let Some(rotation) = mover.rotation.as_mut() {
        rotation.0 = mover.transform.rotation;
    }
    if let Some(linear) = mover.linear.as_mut() {
        linear.0 = shift.direction(linear.0);
    }
    if let Some(angular) = mover.angular.as_mut() {
        angular.0 = shift.direction(angular.0);
    }
}

/// Rebases the surface anchor under the player body once it has walked past the limit, and
/// moves every root [`Rebased`] entity so that it stays where it was on the planet.
pub(super) fn rebase_on_foot(
    terrain: Res<Terrain>,
    mut anchor: ResMut<Anchor>,
    player: Single<Entity, With<Player>>,
    mut movers: Query<Mover, (With<Rebased>, Without<ChildOf>)>,
) {
    let Ok(body) = movers.get(*player) else {
        return;
    };
    let radius = terrain.globe().radius();
    let Some(next) = rebased(&anchor, radius, body.transform.translation) else {
        return;
    };
    let shift = Shift::between(&anchor, &next);
    for mut mover in &mut movers {
        carry(&shift, &mut mover);
    }
    *anchor = next;
}
