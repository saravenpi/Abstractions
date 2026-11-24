//! Fauna: the creatures of the planet underfoot, spawned from its life, moved by their needs and
//! animated by procedural gaits on rigid parts.
//!
//! The herds of the planet's life are pure functions of universe time, read through one
//! `life::herd::Watch` around the local robot. A few times a second the scan spawns the members near
//! the robot and fades out those that went; every frame the senses fill each one's awareness of the
//! robots and hunters near it and set its stance, the walk steps it towards its place in its herd
//! through that stance, and the voice moves its body to the calls it makes.

mod attack;
mod beast;
pub mod bleed;
mod bulk;
mod collide;
mod fade;
mod gate;
mod glow;
mod ground;
pub mod guide;
mod herds;
mod lead;
mod looks;
pub mod mesh;
pub mod motion;
mod near;
mod place;
mod react;
#[cfg(test)]
mod rig;
mod roll;
mod scan;
pub mod senses;
mod shore;
mod shove;
mod signs;
mod site;
mod slant;
#[cfg(test)]
mod stage;
mod steer;
mod sunk;
pub mod sync;
#[cfg(test)]
mod tests_collide;
#[cfg(test)]
mod tests_ground;
#[cfg(test)]
mod tests_handover;
#[cfg(test)]
mod tests_hunt;
#[cfg(test)]
mod tests_live;
#[cfg(test)]
mod tests_meet;
#[cfg(test)]
mod tests_near;
#[cfg(test)]
mod tests_react;
#[cfg(test)]
mod tests_spawn;
#[cfg(test)]
mod tests_survey;
#[cfg(test)]
mod tests_sync;
#[cfg(test)]
mod tests_temper;
#[cfg(test)]
mod tests_walk;
pub mod tune;
mod walk;
mod want;
mod wing;

use std::time::Duration;

use bevy::prelude::*;
use bevy::time::common_conditions::on_timer;

pub use attack::{Attacking, Broken, Carved, Dead, Kind, slay};
pub use beast::{Beast, Part};
pub use bulk::{bulk, capsule};
pub use herds::Herds;
pub use near::{Nearby, Sighting};
pub use sync::{RemoteFauna, SyncTarget};
pub use want::{Lead, carcass_key};
#[cfg(test)]
pub(crate) use {
    fade::Fade,
    place::Spot,
    react::{Mood, Temper},
};

use crate::life::SpeciesId;
use crate::screens::playing;

/// One drawn individual. Its parts are its children, one entity per body part.
#[derive(Component, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Creature {
    /// Its species.
    pub species: SpeciesId,
    /// The herd it belongs to.
    pub herd: u64,
    /// Its place in the herd.
    pub slot: u8,
}

/// What swings and spears may strike: a drawn animal that is not [`Dead`].
pub type Live = (With<Creature>, Without<Dead>);

/// Holds an animal still: its walk skips it while a hitstop freezes it with the robot that struck it.
#[derive(Component, Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct Frozen;

/// The set of the walk, which poses every drawn animal and its parts for the frame: what reads
/// where an animal is drawn this frame runs after it.
#[derive(SystemSet, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Posed;

/// Registers the fauna systems: the forgetting of animals when the planet under the anchor changes,
/// then, on a living planet in play, the scan, the walk and the fades. Calm water mirrors the animals.
pub(super) fn plugin(app: &mut App) {
    app.register_required_components::<Creature, crate::world::Reflected>();
    let scan = scan::scan.run_if(on_timer(Duration::from_secs_f32(tune::SCAN)));
    let shine = glow::shine.run_if(on_timer(Duration::from_secs_f32(tune::SCAN)));
    app.init_resource::<herds::Herds>()
        .init_resource::<gate::Spec>()
        .init_resource::<Nearby>()
        .init_resource::<RemoteFauna>()
        .init_resource::<looks::Looks>()
        .add_systems(
            Update,
            (
                gate::forget.run_if(gate::moved),
                (
                    scan,
                    senses::sense,
                    signs::sing.run_if(signs::sounding),
                    walk::walk.in_set(Posed),
                    fade::fade,
                    shine,
                )
                    .chain()
                    .run_if(playing)
                    .run_if(gate::living),
            )
                .chain()
                .after(crate::effects::Sight),
        )
        .add_plugins((senses::plugin, signs::plugin, attack::plugin, bleed::plugin));
    app.add_plugins((guide::plugin, after_walk));
}

/// Registers what follows the walk each frame: flyers steered clear of colliders, the live herd marks,
/// the bodies the player meets, and once a second the walkers set back on the ground.
fn after_walk(app: &mut App) {
    app.add_plugins((
        wing::plugin,
        collide::plugin,
        near::plugin,
        bulk::plugin,
        sunk::plugin,
    ));
}
