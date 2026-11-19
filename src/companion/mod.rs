//! The companion: a small floating robot with a face on a screen, which follows the player.
//!
//! The logic lives in pure functions (`spring`, `spot`, `idle`, `pose`, `mind`, `attention`,
//! `gaze`, `neck`, `mood`, `face`), driven by a few systems. Other modules talk to it through
//! [`CompanionSense`], which they may write, and [`CompanionChirp`], which they may read.

mod aboard;
mod attention;
mod body;
mod brain;
mod face;
mod gaze;
mod idle;
mod limbs;
mod mind;
mod mood;
mod motion;
mod neck;
mod orb;
mod perch;
mod pose;
mod puppet;
mod ribbon;
mod ride;
mod rng;
mod scene;
mod screen;
mod sense;
mod shapes;
mod shine;
mod sights;
mod soar;
mod solid;
mod spawn;
mod spot;
mod spring;
mod tail;
mod think;
pub mod tune;
mod wake;
mod wing;

#[cfg(test)]
mod tests;
#[cfg(test)]
mod tests_face;
#[cfg(test)]
mod tests_flight;
#[cfg(test)]
mod tests_follow;
#[cfg(test)]
mod tests_gaze;
#[cfg(test)]
mod tests_idle;
#[cfg(test)]
mod tests_loop;
#[cfg(test)]
mod tests_motion;
#[cfg(test)]
mod tests_paint;
#[cfg(test)]
mod tests_pair;
#[cfg(test)]
mod tests_parts;
#[cfg(test)]
mod tests_perch;
#[cfg(test)]
mod tests_place;
#[cfg(test)]
mod tests_soar;
#[cfg(test)]
mod tests_solid;
#[cfg(test)]
mod tests_tour;
#[cfg(test)]
mod tests_yard;

pub use face::machine::Face;
pub use face::{Expression, Frame};
pub use mood::{ChirpKind, CompanionChirp};
pub use motion::Float;
pub use pose::Pose;
pub use screen::{Display, Look};
pub use sense::CompanionSense;
pub use spawn::{Stock, look};

use bevy::prelude::*;

use crate::player::PlayerSet;
use crate::render::Warm;
use crate::screens::playing;
use crate::ship::ShipEvent;
use crate::space::{Anchor, Location};
use crate::universe::Universe;
use crate::world::Terrain;

/// Marker for the root entity of a companion: the local one, which has a brain, or a puppet.
#[derive(Component, Debug, Default, Clone, Copy)]
pub struct Companion;

/// Run condition: the world the companion lives in exists.
fn ready(
    anchor: Option<Res<Anchor>>,
    terrain: Option<Res<Terrain>>,
    location: Option<Res<Location>>,
    universe: Option<Res<Universe>>,
) -> bool {
    anchor.is_some() && terrain.is_some() && location.is_some() && universe.is_some()
}

/// Registers the companion: its inputs, its chirps, its spawn and the systems that run it.
pub(super) fn plugin(app: &mut App) {
    let rendered = resource_exists::<Warm>.and_then(resource_exists::<Assets<Image>>);
    app.init_resource::<CompanionSense>()
        .add_systems(Startup, spawn::warm.run_if(rendered))
        .add_message::<CompanionChirp>()
        .add_message::<ShipEvent>()
        .add_systems(
            Update,
            (
                spawn::spawn,
                sense::fill,
                ride::switch,
                think::think,
                motion::float,
                puppet::track,
                puppet::pose,
                limbs::limbs,
                screen::repaint,
                shine::shine,
                tail::draw,
            )
                .chain()
                .after(PlayerSet::Move)
                .run_if(playing)
                .run_if(ready),
        );
}
