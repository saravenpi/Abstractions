//! Signs of life as drawn and heard: tracks, droppings, grazed patches, dens and bones on the ground
//! around the local robot, and the calls of the animals near it.
//!
//! Once a second the lay reads the signs of `crate::life::signs` within [`tune::RADIUS`] of the robot
//! from its own `life::herd::Watch`, which can look back along the herds' paths, and writes them into
//! one flat mesh of fixed size. A few times a second the hearing sends the calls of the herds and
//! hunts in earshot to the audio as positioned cues. Neither runs off a living planet.

mod hear;
mod lay;
mod paint;
mod shapes;
#[cfg(test)]
mod stage;
#[cfg(test)]
mod tests_hear;
#[cfg(test)]
mod tests_lay;
pub mod tune;

use std::time::Duration;

use bevy::math::DVec3;
use bevy::prelude::*;
use bevy::time::common_conditions::on_timer;

use super::gate;
use crate::life::herd::Watch;
use crate::life::signs::Sign;
use crate::screens::playing;

/// The signs around the local robot: the watch they are read from, the last signs read, and the
/// entity and mesh that draw them.
#[derive(Resource, Default)]
pub struct Trail {
    /// The watch, once rolled.
    watch: Option<Watch>,
    /// Seed of the globe, unit direction and universe time the watch was rolled for.
    rolled: (u64, DVec3, f64),
    /// The signs read at the last lay, nearest first.
    pub signs: Vec<Sign>,
    /// The entity that draws them and its mesh, once spawned.
    pub drawn: Option<(Entity, Handle<Mesh>)>,
}

/// Registers the signs: their forgetting when the planet under the anchor changes, then, on a living
/// planet in play, the lay once a second and the hearing a few times a second.
pub(super) fn plugin(app: &mut App) {
    let lay = lay::lay.run_if(on_timer(Duration::from_secs_f32(tune::LAY)));
    let hear = hear::hear.run_if(on_timer(Duration::from_secs_f32(tune::HEAR)));
    app.init_resource::<Trail>().add_systems(
        Update,
        (
            lay::forget.run_if(gate::moved),
            (lay, hear).run_if(playing).run_if(gate::living),
        )
            .chain()
            .after(crate::effects::Sight),
    );
}
