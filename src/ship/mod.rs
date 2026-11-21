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
mod churn;
mod clear;
pub mod cockpit;
mod cruise;
mod deck;
mod drag;
mod drive;
mod exhaust;
mod flight;
mod frames;
mod fx;
mod gear;
mod glow;
mod headlight;
mod heat;
mod helm;
mod hold;
mod hull;
mod hull_cast;
mod landing;
mod legs;
mod mesh;
mod moons;
mod mooring;
mod orbit;
mod pilot;
mod place;
mod rest;
mod restore;
mod route;
mod save;
mod shake;
mod sim;
mod sky;
mod solid;
mod sound;
mod spawn;
mod steer;
mod strike;
mod telemetry;
mod tones;
mod transit;
mod tune;
mod view;
mod wake;
mod warp;
mod wells;

#[cfg(test)]
mod tests;

pub use board::{Hatch, near_hatch};
pub use body::{Beside, Ship, ShipState};
pub use cruise::top_speed;
pub use drag::{SEA_DENSITY, density};
pub use gear::reach;
pub use glow::{Engine, level_of};
pub use helm::Helm;
pub use legs::Leg;
pub use mesh::{Looks, dress};
pub use pilot::{Autopilot, Phase};
pub use save::Logbook;
pub use spawn::airframe;
pub use telemetry::{ShipTelemetry, WarpState};
pub use view::{HULL_RADIUS, SEAT, seated};
pub use warp::{CRUISE_CAP, GALAXY_CAP, JUMP_CAP, JUMP_DISTANCE, RAMP, SPOOL_TIME, Warp};

use bevy::prelude::*;

use crate::home::Base;
use crate::render::Warm;
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
    /// The pilot switched the headlight: on when true.
    Headlight(bool),
}

/// A crash: a touchdown at or above the crash speed of the point that took it, 25 m/s on the
/// locked gear and 10 m/s on the hull or a folded gear. Nothing damages the ship for it yet;
/// read it with `MessageReader<Impact>`.
#[derive(Message, Debug, Clone, Copy, PartialEq)]
pub struct Impact {
    /// Approach speed of the point that hit, in metres per second along the ground normal.
    pub speed: f64,
    /// Where it hit, in the axes of the surface frame of the planet the ship is on.
    pub at: bevy::math::DVec3,
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
    crate::player::interacting(app);
    app.init_resource::<helm::Helm>()
        .init_resource::<sky::Skies>()
        .init_resource::<sim::Pace>()
        .init_resource::<Autopilot>()
        .init_resource::<ShipTelemetry>()
        .init_resource::<Hatch>()
        .init_resource::<shake::Buffet>()
        .init_resource::<crate::view::ViewPrefs>()
        .add_message::<ShipEvent>()
        .add_message::<Impact>();
}

/// Registers the systems that read the pilot and fly the ship.
fn flight(app: &mut App) {
    let input = (
        board::reseat.run_if(seated),
        helm::read,
        headlight::switch,
        board::board,
        route::halt.run_if(route::halting),
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
        (
            telemetry::update,
            fx::feed,
            churn::feed,
            heat::offer.run_if(heat::hot),
        )
            .chain(),
        legs::fold,
        (route::report, route::finish).chain(),
        (glow::throttle, glow::glow).chain(),
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
        .add_observer(route::on_cancel)
        .configure_sets(
            Update,
            stages.run_if(playing).run_if(any_with_component::<Ship>),
        )
        .add_systems(Update, spawn::spawn.run_if(resource_added::<SpawnPoint>))
        .add_systems(Startup, spawn::warm.run_if(resource_exists::<Warm>));
    let founding = resource_exists::<Base>
        .and_then(unplayed)
        .and_then(in_state(Screen::Loading));
    app.add_systems(Update, spawn::berth_at_base.run_if(founding));
    flight(app);
    display(app);
    hull_cast::plugin(app);
    cockpit::plugin(app);
    exhaust::plugin(app);
    wake::plugin(app);
    sound::plugin(app);
}
