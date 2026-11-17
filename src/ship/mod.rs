//! The player's ship: a Newtonian rigid body that takes off, flies, warps and lands.
//!
//! The flight itself is pure 64-bit math on [`Ship`]. The systems read the helm, step the ship
//! at a fixed rate, keep the render anchor on it while the player is seated and publish
//! [`ShipTelemetry`], [`ShipEvent`] and [`Hatch`] for the rest of the game. A save asks
//! [`Logbook`] for the record of the ship.

mod assist;
mod atmo;
mod berth;
mod board;
mod body;
mod drag;
mod drive;
mod flight;
mod frames;
mod fx;
mod glow;
mod helm;
mod landing;
mod mesh;
mod mooring;
mod orbit;
mod pilot;
mod place;
mod rest;
mod route;
mod save;
mod shake;
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

pub use board::{Hatch, near_hatch};
pub use body::{SavedFrame, Ship, ShipState};
pub use drag::{SEA_DENSITY, density};
pub use pilot::{Autopilot, Phase};
pub use save::Logbook;
pub use telemetry::{ShipTelemetry, WarpState};
pub use view::{HULL_RADIUS, SEAT, seated};
pub use warp::Warp;

use bevy::prelude::*;

use crate::home::Base;
use crate::saves::unplayed;
use crate::screens::{Screen, playing};
use crate::view::ViewSet;
use crate::world::SpawnPoint;

/// Something that happened to the ship. Read it with `MessageReader<ShipEvent>`.
#[derive(Message, Debug, Clone, Copy, PartialEq)]
pub enum ShipEvent {
    /// The ship left the ground it rested on.
    Liftoff,
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
    /// The planet the world streams, the render anchor, the pose of the ship and the seat the
    /// camera rig follows.
    Place,
    /// Instruments, route status, engine glow and the save record.
    Report,
}

/// Puts every flight resource back to its start, for a new ship.
fn reset(commands: &mut Commands) {
    commands.insert_resource(helm::Helm::default());
    commands.insert_resource(sky::Skies::default());
    commands.insert_resource(sim::Pace::default());
    commands.insert_resource(Autopilot::default());
    commands.insert_resource(ShipTelemetry::default());
    commands.insert_resource(Hatch::default());
    commands.insert_resource(shake::Buffet::default());
}

/// Registers the flight resources and the ship messages.
fn resources(app: &mut App) {
    app.init_resource::<helm::Helm>()
        .init_resource::<sky::Skies>()
        .init_resource::<sim::Pace>()
        .init_resource::<Autopilot>()
        .init_resource::<ShipTelemetry>()
        .init_resource::<Hatch>()
        .init_resource::<shake::Buffet>()
        .add_message::<ShipEvent>();
}

/// Registers the systems that read the pilot and fly the ship.
fn flight(app: &mut App) {
    let input = (
        board::reseat.run_if(seated),
        helm::read,
        board::board,
        route::fly,
    );
    let fly = (sim::survey, drive::drive, sim::simulate);
    app.add_systems(
        Update,
        (
            input.chain().in_set(ShipSet::Input),
            fly.chain().in_set(ShipSet::Flight),
        ),
    );
}

/// Registers the systems that place the ship and report on it.
fn display(app: &mut App) {
    let place = (
        place::track,
        place::moor,
        place::pose,
        view::carry.run_if(seated),
    );
    let report = (
        (telemetry::update, fx::feed).chain(),
        route::report,
        glow::glow,
    );
    app.add_systems(
        Update,
        (
            place.chain().in_set(ShipSet::Place),
            report.in_set(ShipSet::Report),
            view::buffet.after(ViewSet::Rig).run_if(seated),
        ),
    )
    .configure_sets(Update, ViewSet::Rig.after(ShipSet::Place));
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
    resources(app);
    app.add_observer(route::on_engage)
        .configure_sets(
            Update,
            stages.run_if(playing).run_if(any_with_component::<Ship>),
        )
        .add_systems(Update, spawn::spawn.run_if(resource_added::<SpawnPoint>));
    let founding = resource_exists::<Base>
        .and_then(unplayed)
        .and_then(in_state(Screen::Loading));
    app.add_systems(Update, spawn::berth_at_base.run_if(founding));
    flight(app);
    display(app);
}
