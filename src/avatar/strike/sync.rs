//! The held tool on the body: a view of the tool the hotbar holds. [`crate::craft::Held`] owns which
//! item is held; this puts it in the hands, or takes it out when the slot is empty.

use bevy::prelude::*;

use super::held::Held;
use super::input::Robot;
use crate::craft::build::Place;
use crate::craft::{Held as Chosen, Pack};
use crate::world::Terrain;

/// Returns true when the hotbar holds another tool than last frame, or the body of the local robot
/// is a new one.
pub fn stale(chosen: Res<Chosen>, robot: Robot, mut seen: Local<Option<Entity>>) -> bool {
    let body = robot.body();
    let new = body != *seen;
    *seen = body;
    chosen.is_changed() || new
}

/// Puts the tool the hotbar holds in the hands of the local robot, or empties them. The tool is
/// built from its item in the pack, for the gravity and air of the planet underfoot.
pub fn sync(
    mut commands: Commands,
    (chosen, pack, terrain): (Res<Chosen>, Res<Pack>, Res<Terrain>),
    robot: Robot,
    held: Query<&Held>,
) {
    let Some(body) = robot.body() else { return };
    let Some(hold) = chosen.0 else {
        commands.entity(body).remove::<Held>();
        return;
    };
    if held.get(body).is_ok_and(|now| now.id == hold.id) {
        return;
    }
    let Some(item) = pack.item(hold.id) else {
        return;
    };
    let place = Place::on(terrain.globe().gravity(), terrain.globe().record());
    let tool = Held::new(item.recipe.clone(), hold.build, place);
    commands.entity(body).insert(Held {
        id: hold.id,
        ..tool
    });
}
