//! Ambient effects: drifting particles.
//!
//! Everything here lives in render space around the camera of a planet. Heights are read along
//! the local vertical through the anchor, never from render `y`, and the pooled entities are
//! marked `Rebased` so an anchor rebase leaves them where they were on the planet.

mod disc;
mod eye;
mod fleck;
mod ground;
mod kind;
mod pick;
mod planet;
mod pool;
mod rate;
mod respawn;
#[cfg(test)]
mod rig;
mod rng;
mod sense;
mod shed;
mod sim;
mod sink;
mod sky;
#[cfg(test)]
mod tests_flow;
#[cfg(test)]
mod tests_pool;
mod tune;
mod water;

use std::time::Duration;

use bevy::prelude::*;
use bevy::time::common_conditions::on_timer;

pub use disc::soft_disc;
pub use fleck::{Fling, Landed};
pub use water::Churn;

use crate::screens::{Screen, playing};

/// How often the plants within reach of the particles are read again.
const GATHER: Duration = Duration::from_millis(500);

/// The system that reads the camera of the frame into the eye. Effects run after it.
#[derive(SystemSet, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Sight;

/// Registers the ambient particle pool and its systems.
pub(super) fn plugin(app: &mut App) {
    app.add_plugins((water::plugin, fleck::plugin))
        .init_resource::<rng::Rng>()
        .init_resource::<sense::Sensed>()
        .init_resource::<eye::Eye>()
        .add_systems(Startup, pool::build)
        .add_systems(OnEnter(Screen::Gameplay), pool::spawn)
        .add_systems(
            Update,
            (
                eye::watch.in_set(Sight),
                sense::sense,
                shed::gather.run_if(on_timer(GATHER)),
                sim::step,
            )
                .chain()
                .run_if(playing)
                .run_if(eye::grounded),
        );
}
