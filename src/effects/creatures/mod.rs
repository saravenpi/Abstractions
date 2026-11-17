//! Small wandering creatures: night fireflies, day butterflies and flocks of birds.

pub mod anim;
mod birds;
mod butterflies;
mod carry;
mod env;
mod fireflies;
mod flock;
mod gear;
mod habitat;
mod steer;
mod wander;
mod wings;

#[cfg(test)]
mod tests_fireflies;
#[cfg(test)]
mod tests_globe;
#[cfg(test)]
mod tests_rebase;

use bevy::prelude::*;

use super::Sight;
use super::eye::grounded;
use crate::screens::{Screen, playing};

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

/// Registers the creature pools and one movement system per kind. They run once the eye of the
/// frame is known, after the remembered points were carried across an anchor rebase.
pub(super) fn plugin(app: &mut App) {
    let steps = (fireflies::step, butterflies::step, birds::step);
    app.init_resource::<fireflies::Swarms>()
        .init_resource::<flock::Flocks>()
        .add_systems(Startup, gear::build)
        .add_systems(OnEnter(Screen::Gameplay), spawn)
        .add_systems(
            Update,
            (carry::carry, steps)
                .chain()
                .after(Sight)
                .run_if(playing)
                .run_if(grounded),
        );
}
