//! Ambient effects: drifting particles and small wandering creatures.
//!
//! Everything here lives in render space around the camera of a planet. Heights are read along
//! the local vertical through the anchor, never from render `y`, and the pooled entities are
//! marked `Rebased` so an anchor rebase leaves them where they were on the planet.

mod creatures;
mod eye;
mod ground;
mod kind;
mod pick;
mod pool;
mod respawn;
mod rng;
mod sense;
mod sim;

use bevy::prelude::*;

use crate::screens::{Screen, playing};

/// The system that reads the camera of the frame into the eye. Effects run after it.
#[derive(SystemSet, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Sight;

/// Registers the ambient particle pool and its systems.
pub(super) fn plugin(app: &mut App) {
    creatures::plugin(app);
    app.init_resource::<rng::Rng>()
        .init_resource::<sense::Sensed>()
        .init_resource::<eye::Eye>()
        .add_systems(Startup, pool::build)
        .add_systems(OnEnter(Screen::Gameplay), pool::spawn)
        .add_systems(
            Update,
            (eye::watch.in_set(Sight), sense::sense, sim::step)
                .chain()
                .run_if(playing)
                .run_if(eye::grounded),
        );
}
