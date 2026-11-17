//! The player's ship: a Newtonian rigid body that takes off, flies, warps and lands.
//!
//! The flight itself is pure 64-bit math on [`Ship`]. The systems read the helm, step the ship
//! at a fixed rate, keep the render anchor on it while the player is seated and publish
//! [`ShipTelemetry`] and [`ShipEvent`] for the rest of the game.

mod assist;
mod atmo;
mod berth;
mod board;
mod body;
mod drag;
mod drive;
mod flight;
mod frames;
mod glow;
mod helm;
mod landing;
mod mesh;
mod orbit;
mod pilot;
mod place;
mod rest;
mod route;
mod save;
mod sim;
mod sky;
mod solid;
mod spawn;
mod steer;
mod telemetry;
mod transit;
mod view;
mod warp;
mod wells;

#[cfg(test)]
mod tests;

pub use body::{SavedFrame, Ship, ShipState};
pub use pilot::{Autopilot, Phase};
pub use telemetry::{ShipTelemetry, WarpState};
pub use view::seated;
pub use warp::Warp;

use bevy::prelude::*;

use crate::screens::playing;
use crate::world::SpawnPoint;

/// Something that happened to the ship. Read it with `MessageReader<ShipEvent>`.
#[derive(Message, Debug, Clone, Copy, PartialEq)]
pub enum ShipEvent {
    /// The ship touched the ground after flying.
    Touchdown {
        /// Speed of the touchdown in metres per second. Above 6 on the gear it damages the ship.
        speed: f64,
        /// Damage of the ship after the touchdown, from 0 to 1.
        damage: f64,
    },
    /// The warp drive did not spool or stopped spooling: the ship is landed, in an atmosphere
    /// or inside a lock sphere.
    WarpRefused,
    /// The warp drive started to spool.
    WarpSpooling,
    /// The ship went into warp.
    WarpEngaged,
    /// The ship dropped out of warp.
    WarpDropped,
    /// The player took the seat.
    Boarded,
    /// The player left the ship.
    Left,
}

/// The stages of the ship systems, in this order in `Update`, while playing with a ship.
#[derive(SystemSet, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ShipSet {
    /// The helm, boarding and the autopilot.
    Input,
    /// Frame changes, the warp drive and the fixed flight steps.
    Flight,
    /// The planet the world streams, the render anchor, the pose of the ship and the camera.
    Place,
    /// Instruments, route status, engine glow and the save record.
    Report,
}

/// Puts every flight resource back to its start, for a new ship.
fn reset(commands: &mut Commands) {
    commands.insert_resource(helm::Helm::default());
    commands.insert_resource(sky::Skies::default());
    commands.insert_resource(sim::Pace::default());
    commands.insert_resource(view::View::default());
    commands.insert_resource(Autopilot::default());
    commands.insert_resource(ShipTelemetry::default());
}

/// Registers the ship resources, messages and systems.
pub(super) fn plugin(app: &mut App) {
    let stages = (
        ShipSet::Input,
        ShipSet::Flight,
        ShipSet::Place,
        ShipSet::Report,
    )
        .chain();
    app.init_resource::<helm::Helm>()
        .init_resource::<sky::Skies>()
        .init_resource::<sim::Pace>()
        .init_resource::<view::View>()
        .init_resource::<Autopilot>()
        .init_resource::<ShipTelemetry>()
        .add_message::<ShipEvent>()
        .add_observer(route::on_engage)
        .configure_sets(
            Update,
            stages.run_if(playing).run_if(any_with_component::<Ship>),
        )
        .add_systems(Update, spawn::spawn.run_if(resource_added::<SpawnPoint>))
        .add_systems(
            Update,
            (
                (helm::read, board::board, route::fly)
                    .chain()
                    .in_set(ShipSet::Input),
                (sim::survey, drive::drive, sim::simulate)
                    .chain()
                    .in_set(ShipSet::Flight),
                (
                    place::track,
                    place::moor,
                    place::pose,
                    view::look.run_if(seated),
                )
                    .chain()
                    .in_set(ShipSet::Place),
                (telemetry::update, route::report, glow::glow, save::record)
                    .in_set(ShipSet::Report),
            ),
        );
}
