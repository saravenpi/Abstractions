//! Waking at the home base: where a new game puts the player once the base is found.

use bevy::prelude::*;

use super::Player;
use super::seat::place;
use crate::home::{Base, Works};
use crate::saves::{ActiveWorld, unplayed};
use crate::screens::Screen;
use crate::space::{Anchor, Frame};
use crate::world::WorldReady;

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

/// Stands the player of a new game at the wake point of the home base, facing the hatch of the
/// container, once the base is found while the world loads.
///
/// The base is found in the background and the loading screen waits for it. Moving the body
/// takes the readiness of the world back: the ground under the base has to load before play.
fn wake_at_base(
    mut commands: Commands,
    base: Res<Base>,
    anchor: Res<Anchor>,
    sleepers: Query<Entity, (With<Player>, Without<Woken>)>,
) {
    let Ok(body) = sleepers.single() else {
        return;
    };
    commands.entity(body).insert(Woken);
    if anchor.frame != Frame::Surface(base.body) {
        return;
    }
    commands.insert_resource(WorldReady(false));
    let feet = anchor.to_render(base.wake);
    let toward = (anchor.rotation * base.look()).as_vec3();
    commands.queue(move |world: &mut World| place(world, feet, toward));
}

/// Registers the wake, which only runs while a new game loads and its base exists.
pub(super) fn plugin(app: &mut App) {
    let founding = resource_exists::<Base>
        .and_then(unplayed)
        .and_then(in_state(Screen::Loading));
    app.add_systems(Update, wake_at_base.run_if(founding));
}
