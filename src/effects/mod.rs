//! Ambient effects: drifting particles and small wandering creatures.

mod creatures;
mod kind;
mod pick;
mod pool;
mod respawn;
mod rng;
mod sense;
mod sim;

use bevy::prelude::*;

use crate::screens::{Screen, playing};
use crate::world::Terrain;

/// Registers the ambient particle pool and its systems.
pub(super) fn plugin(app: &mut App) {
    creatures::plugin(app);
    app.init_resource::<rng::Rng>()
        .init_resource::<sense::Sensed>()
        .add_systems(Startup, pool::build)
        .add_systems(OnEnter(Screen::Gameplay), pool::spawn)
        .add_systems(
            Update,
            (sense::sense, sim::step)
                .chain()
                .run_if(playing)
                .run_if(resource_exists::<Terrain>),
        );
}
