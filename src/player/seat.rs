//! Hand-over with the ship: putting the body somewhere and switching it off while seated.

use avian3d::prelude::*;
use bevy::prelude::*;

use super::controller::Motion;
use super::spawn::Home;
use super::upright::{facing, yaw_toward};
use super::{BODY_LENGTH, BODY_RADIUS, Player};
use crate::gear::Gear;
use crate::space::{Anchor, up};
use crate::view::{Cut, Heading};
use crate::world::Planet;

/// Gap left between the feet and the point the body is placed on.
const PLACE_LIFT: f32 = 0.05;

/// The parts of the player body that are put somewhere.
type Placed<'a> = (
    &'a mut Transform,
    &'a mut Motion,
    &'a mut Home,
    Option<&'a mut Gear>,
);

/// Stands the player body with its feet at render position `feet`, upright along the local
/// vertical and looking along the render direction `toward`, at rest, with the flight gear
/// folded. The spot becomes the place the body returns to if it falls out of the world, and
/// the camera cuts to the new view instead of gliding there.
///
/// Does nothing without a world or a body, and on a planet without ground, where no body can stand.
/// Queue it from a system with `commands.queue(move |world: &mut World| place(world, feet, toward))`.
pub fn place(world: &mut World, feet: Vec3, toward: Vec3) {
    let deck = world
        .get_resource::<Planet>()
        .is_some_and(|p| !p.globe.has_ground());
    let Some(anchor) = world.get_resource::<Anchor>().copied().filter(|_| !deck) else {
        return;
    };
    let vertical = up(&anchor, feet);
    let center = feet + vertical * (BODY_LENGTH * 0.5 + BODY_RADIUS + PLACE_LIFT);
    let yaw = yaw_toward(toward, vertical);
    let home = Home(anchor.to_frame(center));
    let mut bodies = world.query_filtered::<Placed, With<Player>>();
    for (mut transform, mut motion, mut back, gear) in bodies.iter_mut(world) {
        *back = home;
        transform.translation = center;
        transform.rotation = facing(vertical, yaw);
        *motion = Motion { yaw, ..default() };
        gear.into_iter()
            .for_each(|mut gear| *gear = Gear::default());
    }
    let heading = Some(Heading { yaw, pitch: 0.0 });
    if let Some(mut cuts) = world.get_resource_mut::<Messages<Cut>>() {
        cuts.write(Cut { heading });
    }
}

/// Switches the player body off while `seated` in the ship and back on when not: a seated body
/// has no collider, so nothing hits it and no query finds it. Taking the seat also folds the
/// flight gear and stops the body, whatever it was doing.
///
/// The body keeps its transform. The player systems themselves stop with `Travel::InShip`.
pub fn seat(world: &mut World, seated: bool) {
    let mut bodies = world.query_filtered::<Entity, With<Player>>();
    let found: Vec<Entity> = bodies.iter(world).collect();
    for body in found {
        let mut entity = world.entity_mut(body);
        if !seated {
            entity.remove::<ColliderDisabled>();
            continue;
        }
        entity.insert(ColliderDisabled);
        if let Some(mut gear) = entity.get_mut::<Gear>() {
            *gear = Gear::default();
        }
        if let Some(mut motion) = entity.get_mut::<Motion>() {
            *motion = Motion {
                yaw: motion.yaw,
                ..default()
            };
        }
    }
}
