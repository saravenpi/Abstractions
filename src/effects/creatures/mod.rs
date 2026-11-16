//! Small wandering creatures: night fireflies, day butterflies and flocks of birds.

pub mod anim;
mod birds;
mod butterflies;
mod env;
mod fireflies;
mod flock;
mod gear;
mod habitat;
mod steer;
mod wander;
mod wings;

use bevy::prelude::*;

use crate::screens::{Screen, playing};
use crate::world::Terrain;

/// Spawns every creature pool when gameplay starts, unless they already exist.
fn spawn(
    mut commands: Commands,
    gear: Res<gear::Gear>,
    existing: Query<(), With<fireflies::Firefly>>,
) {
    if !existing.is_empty() {
        return;
    }
    fireflies::spawn(&mut commands, &gear);
    butterflies::spawn(&mut commands, &gear);
    birds::spawn(&mut commands, &gear);
}

/// Registers the creature pools and one movement system per kind.
pub(super) fn plugin(app: &mut App) {
    app.init_resource::<fireflies::Swarms>()
        .init_resource::<flock::Flocks>()
        .add_systems(Startup, gear::build)
        .add_systems(OnEnter(Screen::Gameplay), spawn)
        .add_systems(
            Update,
            (fireflies::step, butterflies::step, birds::step)
                .run_if(playing)
                .run_if(resource_exists::<Terrain>),
        );
}
