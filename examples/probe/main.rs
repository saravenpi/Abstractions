//! Headless playtest of the real game: a new game toured stage by stage, one screenshot per stage, with
//! main-thread frame times and the streaming meter logged as `SMOKE` lines.
//!
//! Run: `SMOKE_SEED=1 SMOKE_DIR=<dir> cargo run --features dev --example probe`.
//!
//! Environment:
//! - `SMOKE_DIR`: the screenshot folder.
//! - `SMOKE_SEED`: the world seed, default 7.
//! - `PROBE_STAGES`: comma list of stage groups, default all. Groups: `water` (lens tour and room walls),
//!   `grass` (turf and trees), `ground` (one patch of grass from the horizon and from 2 m to 30 km), `fauna` (animals), `tour` (base, village, flight, orbit, warp) and `zoo`.
//!   The title, loading, boot and wake stages always run.
//! - `SMOKE_ONLY` and `SMOKE_SKIP`: comma lists of stage names to keep or to leave out.
//! - `SMOKE_TARGET` and `SMOKE_FLASH`: see the `warp` and `boot` stages.
//! - `PROBE_JOIN=<address>`: join that server instead of starting a world, with `PROBE_NAME` as the player name
//!   and `PROBE_PASSWORD`. The `mp` stages then meet the other players (see the `mp` module).
//! - `PROBE_SOFT`: when set, a system whose parameters fail validation is logged and skipped instead of
//!   stopping the game, so one run lists every such system.
//!
//! `PROBE_STAGES=models` opens the model studio (see the `models` module): the robot and the companion alone.
//!
//! `PROBE_STAGES=zoo` does not tour the game. It shows every animal of the seed's home planet walking in a
//! line on flat ground (see the `zoo` module); with `SMOKE_DIR` set it saves a shot of each gait and exits,
//! and `ZOO_FIND=<count>` prints the life of the home planet of seeds 0 to `count` instead.

use std::time::{Duration, Instant};

use abstractions::computer::{Engage, Target};
use abstractions::home::Base;
use abstractions::net::JoinServer;
use abstractions::player::{Motion, Player, place};
use abstractions::saves::{SaveDir, Starter, new_entry};
use abstractions::screens::{Screen, WarmUp};
use abstractions::settings::{Persistent, Settings};
use abstractions::ship::{Autopilot, Phase, Ship, ShipTelemetry};
use abstractions::space::{Anchor, Frame, Location, Travel};
use abstractions::universe::{BodyId, Universe};
use abstractions::view::{Cut, Heading, ViewMode, ViewPrefs};
use abstractions::world::globe::Face;
use abstractions::world::structures::{CELL, Kind, entries, structure_in_cell};
use abstractions::world::{METER_STEPS, Meter, Planet, Terrain};
use bevy::ecs::system::RunSystemOnce;
use bevy::input::ButtonState;
use bevy::input::keyboard::{Key, KeyboardInput, NativeKey};
use bevy::math::{DQuat, DVec3};
use bevy::prelude::*;
use bevy::render::view::screenshot::{Screenshot, save_to_disk};
use bevy::time::TimeUpdateStrategy;
use bevy::window::PrimaryWindow;

mod air;
mod base;
mod boot;
mod burn;
mod cabin;
mod camera;
mod foot;
mod grass;
mod ground;
mod herd;
mod hull;
mod launch;
mod look;
mod meter;
mod models;
mod mp;
mod mpact;
mod mpspot;
mod orbital;
mod room;
mod screens;
mod seen;
mod shots;
mod stages;
mod stops;
mod tour;
mod trees;
mod util;
mod walls;
mod watch;
mod water;
mod zoo;

use air::*;
use base::*;
use boot::*;
use burn::*;
use cabin::*;
use camera::*;
use foot::*;
use grass::*;
use ground::*;
use herd::*;
use hull::*;
use launch::*;
use look::*;
use meter::*;
use mp::*;
use mpact::*;
use mpspot::*;
use orbital::*;
use room::*;
use screens::*;
use seen::*;
use shots::*;
use stages::STAGES;
use stops::*;
use tour::*;
use trees::*;
use util::*;
use walls::*;
use watch::*;
use water::*;

/// Runs the zoo when `PROBE_STAGES` names it, else the tour of the game.
fn main() -> AppExit {
    let list = std::env::var("PROBE_STAGES").unwrap_or_default();
    if list.split(',').any(|g| g == "models") {
        return models::run();
    }
    if list.split(',').any(|g| g == "zoo") {
        return zoo::run();
    }
    let mut app = App::new();
    if std::env::var("PROBE_SOFT").is_ok() {
        app.set_error_handler(bevy::ecs::error::error);
    }
    app.add_plugins(abstractions::AppPlugin);
    app.insert_resource(TimeUpdateStrategy::ManualDuration(Duration::from_secs_f64(
        WALK,
    )));
    app.init_resource::<Tour>();
    app.insert_resource(FrameStart(Instant::now(), [0.0; 3]));
    app.add_systems(First, |mut start: ResMut<FrameStart>| {
        start.0 = Instant::now()
    });
    app.add_systems(PreUpdate, mark::<0>);
    app.add_systems(Update, (mark::<1>, run));
    app.add_systems(PostUpdate, mark::<2>);
    app.init_resource::<Lower>();
    app.init_resource::<Eye>();
    app.add_systems(
        PostUpdate,
        hold.after(bevy::transform::TransformSystems::Propagate)
            .after(lower),
    );
    app.add_systems(
        PostUpdate,
        lower.after(bevy::transform::TransformSystems::Propagate),
    );
    app.init_resource::<Tris>();
    app.add_systems(Last, (measure, count_tris, probe));
    app.run()
}
