//! Waking at the home base: where a new game puts the player once the base is found.

use bevy::prelude::*;

use super::Player;
use super::seat::place;
use crate::home::Base;
use crate::saves::unplayed;
use crate::screens::Screen;
use crate::space::{Anchor, Frame};

/// Marks a body that already woke at the home base.
#[derive(Component, Debug, Clone, Copy)]
struct Woken;

/// Stands the player of a new game at the wake point of the home base, facing the hatch of the
/// container, once the base is found while the world loads.
///
/// The base is found in the background. If it is not there before the game begins, the player
/// starts at the terrain spawn, as without a base.
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
