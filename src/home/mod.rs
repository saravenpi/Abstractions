//! The home base on the home planet: the control container, the station and the landing pad.
//!
//! [`Base`] says where it is. The rest draws it once per world, spawns it while the player is
//! near it on the home planet, and gives it a little life.

mod base;
mod block;
mod cables;
mod choose;
mod container;
mod draft;
mod found;
mod glow;
mod ground;
mod lights;
mod lot;
mod pad;
mod paint;
mod parts;
mod path;
mod plan;
mod site;
mod soup;
mod spawn;
mod station;
mod tune;
mod works;

#[cfg(test)]
mod tests;
#[cfg(test)]
mod tests_app;
#[cfg(test)]
mod tests_cover;
#[cfg(test)]
mod tests_found;
#[cfg(test)]
mod tests_ground;
#[cfg(test)]
mod tests_home;
#[cfg(test)]
mod tests_points;
#[cfg(test)]
mod tests_solid;
#[cfg(test)]
mod tests_sun;

pub use base::Base;
pub use tune::{FOOTPRINT, PAD_HALF, PAD_LIFT};
pub use works::Works;

use bevy::prelude::*;

use crate::render::Warm;
use crate::screens::Screen;
use crate::universe::Universe;

/// Forgets the base of the world that ended, with any work still running for it.
fn forget(mut commands: Commands, mut works: ResMut<works::Works>) {
    *works = works::Works::default();
    commands.remove_resource::<Base>();
}

/// Run condition: a world exists and its base is not known yet.
fn unfound(universe: Option<Res<Universe>>, base: Option<Res<Base>>) -> bool {
    universe.is_some() && base.is_none()
}

/// Run condition: the base is known and spawned.
fn raised(base: Option<Res<Base>>, homes: Query<(), With<spawn::Home>>) -> bool {
    base.is_some() && !homes.is_empty()
}

/// Registers the systems that find, build, spawn and light the base.
fn systems(app: &mut App) {
    app.init_resource::<works::Works>().add_systems(
        Update,
        (
            works::locate.run_if(unfound),
            (works::prepare, spawn::tend)
                .chain()
                .run_if(resource_exists::<Base>),
            (lights::notice, lights::glow, lights::beacon)
                .chain()
                .run_if(raised),
        ),
    );
}

/// Registers the home base systems.
pub(super) fn plugin(app: &mut App) {
    systems(app);
    app.add_systems(OnEnter(Screen::Title), forget)
        .add_systems(Startup, works::warm.run_if(resource_exists::<Warm>));
}
