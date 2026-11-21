//! Putting down: each thing the pack let go of lies ahead of the robot's feet, on the ground under
//! that spot, and goes into the ground of the world.

use avian3d::prelude::*;
use bevy::ecs::system::SystemParam;
use bevy::math::DVec3;
use bevy::prelude::*;

use super::show::{Lies, extent};
use super::tune::{AHEAD, FALL};
use super::{Ground, Note, Shed, codec, restore};
use crate::craft::Pack;
use crate::craft::build::Place;
use crate::player::{BODY_LENGTH, BODY_RADIUS, Player};
use crate::render::GameCamera;
use crate::space::{Anchor, Clock, Frame, up};
use crate::universe::BodyId;
use crate::world::Terrain;

/// The line when there is no ground to put a thing down on.
const NOWHERE: &str = "Nothing to put it down on here";

/// Where a thing is put down: the robot, the eyes, the ground under them and the time.
#[derive(SystemParam)]
pub struct Spot<'w, 's> {
    anchor: Option<Res<'w, Anchor>>,
    terrain: Option<Res<'w, Terrain>>,
    clock: Option<Res<'w, Clock>>,
    camera: Query<'w, 's, &'static GlobalTransform, With<GameCamera>>,
    robot: Query<'w, 's, (Entity, &'static Transform), With<Player>>,
    space: SpatialQuery<'w, 's>,
    lies: Query<'w, 's, (), With<Lies>>,
}

impl Spot<'_, '_> {
    /// Returns the body and the place in its turning frame where a thing `half` metres across from
    /// its middle lies, ahead of the robot: on the ground under the spot, or at the height of the
    /// feet when the ray finds no ground. Nothing off the surface of a body.
    fn ahead(&self, half: f32) -> Option<(BodyId, DVec3)> {
        let anchor = self.anchor.as_deref()?;
        let Frame::Surface(body) = anchor.frame else {
            return None;
        };
        let (me, at) = self.robot.single().ok()?;
        let up = up(anchor, at.translation);
        let look = self
            .camera
            .single()
            .map_or(*at.forward(), |eye| *eye.forward());
        let ahead = look
            .reject_from_normalized(up)
            .normalize_or(up.any_orthonormal_vector());
        let over = at.translation + ahead * (AHEAD + half);
        let down = Dir3::new(-up).ok()?;
        let open = |entity: Entity| entity != me && !self.lies.contains(entity);
        let filter = SpatialQueryFilter::default();
        let hit = self
            .space
            .cast_ray_predicate(over, down, FALL, true, &filter, &open);
        let drop = hit.map_or(BODY_RADIUS + BODY_LENGTH * 0.5, |hit| hit.distance);
        Some((body, anchor.to_frame(over - up * drop)))
    }

    /// Returns the place a tool is used on here and the universe time, once a planet is underfoot.
    fn now(&self) -> Option<(Place, f64)> {
        let globe = self.terrain.as_deref()?.globe();
        let place = Place::on(globe.gravity(), globe.record());
        Some((place, self.clock.as_deref()?.seconds))
    }
}

/// Puts down every thing the pack let go of. A thing with no ground to lie on, or that the ground
/// refuses, goes back into the pack and the banner says why.
pub(super) fn lay(
    (mut shed, mut ground, mut pack): (ResMut<Shed>, ResMut<Ground>, ResMut<Pack>),
    spot: Spot,
    mut notes: MessageWriter<Note>,
) {
    let now = spot.now();
    for thing in std::mem::take(&mut shed.0) {
        let half = extent(&thing).map_or(0.1, |(size, _)| size.x.max(size.z) * 0.5);
        let at = spot.ahead(half).zip(now);
        let record = at.and_then(|(at, (place, since))| codec::lying(&thing, at, &place, since));
        let Some(record) = record else {
            restore(&mut pack, thing);
            notes.write(Note(NOWHERE.to_string()));
            continue;
        };
        if let Some(line) = ground.put(record, thing, &mut pack) {
            notes.write(Note(line.to_string()));
        }
    }
}
