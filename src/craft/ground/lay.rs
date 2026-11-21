//! Putting down: each thing the pack let go of lies ahead of the robot's feet, on the ground under
//! that spot, and goes into the ground of the world.

use avian3d::prelude::*;
use bevy::ecs::system::SystemParam;
use bevy::math::DVec3;
use bevy::prelude::*;

use super::show::{Lies, extent};
use super::tune::{AHEAD, FALL, RECENT, TURNS};
use super::{Ground, Note, Shed, Thing, codec, restore};
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
    /// its middle lies, ahead of the robot turned by `turn` radians about its up: on the ground
    /// under the spot, or at the height of the feet when the ray finds no ground. Nothing off the
    /// surface of a body.
    fn ahead(&self, half: f32, turn: f32) -> Option<(BodyId, DVec3)> {
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
        let ahead = Quat::from_axis_angle(up, turn) * ahead;
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

    /// Returns where a thing `half` metres across goes: a heap straight ahead, where it joins a heap
    /// of its material; a tool at the first of [`TURNS`] around the feet that keeps it its length
    /// clear of every place in `taken`, else straight ahead.
    fn free(
        &self,
        (half, tool): (f32, bool),
        taken: &[(BodyId, DVec3)],
    ) -> Option<(BodyId, DVec3)> {
        let turns = &TURNS[..[1, TURNS.len()][usize::from(tool)]];
        let spots = turns.iter().filter_map(|turn| self.ahead(half, *turn));
        clear_of(spots, f64::from(half * 2.0), taken)
    }

    /// Returns the place a tool is used on here and the universe time, once a planet is underfoot.
    fn now(&self) -> Option<(Place, f64)> {
        let globe = self.terrain.as_deref()?.globe();
        let place = Place::on(globe.gravity(), globe.record());
        Some((place, self.clock.as_deref()?.seconds))
    }
}

/// Returns the first of `spots` at least `room` metres from every place of `taken` on its body,
/// else the first of `spots`; the spots after the first clear one are never asked for.
pub(super) fn clear_of(
    mut spots: impl Iterator<Item = (BodyId, DVec3)>,
    room: f64,
    taken: &[(BodyId, DVec3)],
) -> Option<(BodyId, DVec3)> {
    let clear = |(body, at): &(BodyId, DVec3)| {
        let near = |(other, there): &(BodyId, DVec3)| other == body && there.distance(*at) < room;
        !taken.iter().any(near)
    };
    let first = spots.next()?;
    if clear(&first) {
        return Some(first);
    }
    Some(spots.find(clear).unwrap_or(first))
}

/// Returns the places things lie at on the ground and were just put down at, which a tool put
/// down keeps clear of.
fn taken(ground: &Ground, recent: &[(BodyId, DVec3, f64)]) -> Vec<(BodyId, DVec3)> {
    let lying = ground
        .pile
        .lying
        .iter()
        .map(|l| (l.body, DVec3::from(l.at)));
    lying
        .chain(recent.iter().map(|&(body, at, _)| (body, at)))
        .collect()
}

/// Puts down every thing the pack let go of, tools spread around the feet. A thing with no ground
/// to lie on, or that the ground refuses, goes back into the pack and the banner says why.
pub(super) fn lay(
    (mut shed, mut ground, mut pack): (ResMut<Shed>, ResMut<Ground>, ResMut<Pack>),
    spot: Spot,
    mut notes: MessageWriter<Note>,
    mut recent: Local<Vec<(BodyId, DVec3, f64)>>,
) {
    let now = spot.now();
    let since = now.map_or(0.0, |(_, since)| since);
    recent.retain(|(_, _, at)| since - at < RECENT);
    for thing in std::mem::take(&mut shed.0) {
        let half = extent(&thing).map_or(0.1, |(size, _)| size.x.max(size.z) * 0.5);
        let tool = matches!(thing, Thing::Tool(_));
        let at = spot.free((half, tool), &taken(&ground, &recent));
        let record = at
            .zip(now)
            .and_then(|(at, (place, since))| codec::lying(&thing, at, &place, since));
        let Some(record) = record else {
            restore(&mut pack, thing);
            notes.write(Note(NOWHERE.to_string()));
            continue;
        };
        recent.push((record.body, DVec3::from(record.at), since));
        if let Some(line) = ground.put(record, thing, &mut pack) {
            notes.write(Note(line.to_string()));
        }
    }
}
