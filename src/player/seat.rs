//! Hand-over with the ship: putting the body somewhere and switching it off while seated.

use avian3d::prelude::*;
use bevy::prelude::*;

use super::controller::Motion;
use super::spawn::Home;
use super::upright::{facing, yaw_toward};
use super::{BODY_LENGTH, BODY_RADIUS, Player};
use crate::space::{Anchor, up};

/// Gap left between the feet and the point the body is placed on.
const PLACE_LIFT: f32 = 0.05;

/// Stands the player body with its feet at render position `feet`, upright along the local
/// vertical and looking along the render direction `toward`, at rest. The spot becomes the
/// place the body returns to if it falls out of the world.
///
/// Does nothing without a world or a body. Queue it from a system with
/// `commands.queue(move |world: &mut World| place(world, feet, toward))`.
pub fn place(world: &mut World, feet: Vec3, toward: Vec3) {
    let Some(anchor) = world.get_resource::<Anchor>().copied() else {
        return;
    };
    let vertical = up(&anchor, feet);
    let center = feet + vertical * (BODY_LENGTH * 0.5 + BODY_RADIUS + PLACE_LIFT);
    let yaw = yaw_toward(toward, vertical);
    let home = Home(anchor.to_frame(center));
    let mut bodies =
        world.query_filtered::<(&mut Transform, &mut Motion, &mut Home), With<Player>>();
    for (mut transform, mut motion, mut back) in bodies.iter_mut(world) {
        *back = home;
        transform.translation = center;
        transform.rotation = facing(vertical, yaw);
        *motion = Motion { yaw, ..default() };
    }
}

/// Switches the player body off while `seated` in the ship and back on when not: a seated body
/// has no collider, so nothing hits it and no query finds it.
///
/// The body keeps its transform. The player systems themselves stop with `Travel::InShip`.
pub fn seat(world: &mut World, seated: bool) {
    let mut bodies = world.query_filtered::<Entity, With<Player>>();
    let found: Vec<Entity> = bodies.iter(world).collect();
    for body in found {
        let mut entity = world.entity_mut(body);
        if seated {
            entity.insert(ColliderDisabled);
        } else {
            entity.remove::<ColliderDisabled>();
        }
    }
}
