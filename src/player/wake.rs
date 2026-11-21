//! Waking at the home base: where a new game puts the player once the base is found, and standing
//! a body on the ground under it when the place it was saved at does not load.

use bevy::prelude::*;

use super::Player;
use super::seat::place;
use crate::home::{Base, Works};
use crate::saves::{ActiveWorld, unplayed};
use crate::screens::Screen;
use crate::sim::PlayerId;
use crate::space::{Anchor, Frame, altitude, up};
use crate::world::{Terrain, WorldReady};

/// Marks a body that already woke at the home base.
#[derive(Component, Debug, Clone, Copy)]
pub struct Woken;

/// Run condition: the body is where the game starts. False while a new game still looks for
/// its home base or has not stood the body there, so the world never opens on a body that is
/// about to be moved to ground that is not loaded yet.
pub fn awake(
    active: Option<Res<ActiveWorld>>,
    works: Option<Res<Works>>,
    base: Option<Res<Base>>,
    sleepers: Query<(), (With<Player>, Without<Woken>)>,
) -> bool {
    let fresh = active.is_none_or(|active| active.0.save.place.is_none());
    !fresh || works.is_none() || (base.is_some() && sleepers.is_empty())
}

/// Wakes the bodies of a new game and stands the local player at the wake point of the home
/// base, facing the hatch of the container, once the base is found while the world loads.
///
/// The base is found in the background and the loading screen waits for it. Moving the body
/// takes the readiness of the world back: the ground under the base has to load before play.
fn wake_at_base(
    mut commands: Commands,
    base: Res<Base>,
    anchor: Res<Anchor>,
    sleepers: Query<Entity, (With<Player>, Without<Woken>)>,
) {
    if sleepers.is_empty() {
        return;
    }
    for body in &sleepers {
        commands.entity(body).insert(Woken);
    }
    if anchor.frame != Frame::Surface(base.body) {
        return;
    }
    commands.insert_resource(WorldReady(false));
    let feet = anchor.to_render(base.wake);
    let toward = (anchor.rotation * base.look()).as_vec3();
    commands.queue(move |world: &mut World| place(world, feet, toward));
}

/// Stands the body of the local player on the ground under it, or on the sea where the ground is
/// under the sea, looking the way it looked. Does nothing without a world or a body.
///
/// The loading screen queues it for a body whose place did not get ready in time, with
/// `commands.queue(land)`. The body is then where the ground is asked for at its finest.
pub fn land(world: &mut World) {
    let anchor = world.get_resource::<Anchor>().copied();
    let terrain = world.get_resource::<Terrain>().cloned();
    let (Some(anchor), Some(terrain)) = (anchor, terrain) else {
        return;
    };
    let mut bodies = world.query_filtered::<(&PlayerId, &Transform), With<Player>>();
    let mine = bodies.iter(world).find(|(id, _)| **id == PlayerId::LOCAL);
    let Some(at) = mine.map(|(_, at)| *at) else {
        return;
    };
    let chart = terrain.locate(anchor.to_frame(at.translation));
    let floor = terrain
        .height(chart.x, chart.z)
        .max(terrain.globe().sea_level());
    let height = altitude(&anchor, terrain.globe(), at.translation);
    let feet = at.translation + up(&anchor, at.translation) * (floor - height);
    place(world, feet, at.forward().as_vec3());
}

/// Registers the wake, which only runs while a new game loads and its base exists.
pub(super) fn plugin(app: &mut App) {
    let founding = resource_exists::<Base>
        .and_then(unplayed)
        .and_then(in_state(Screen::Loading));
    app.add_systems(Update, wake_at_base.run_if(founding));
}
