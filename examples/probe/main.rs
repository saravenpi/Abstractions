//! Headless playtest of the real game: a new game toured stage by stage, one screenshot per stage, with
//! main-thread frame times and the streaming meter logged as `SMOKE` lines.
//!
//! Run: `SMOKE_SEED=1 SMOKE_DIR=<dir> cargo run --features dev --example probe`.
//!
//! Environment:
//! - `SMOKE_DIR`: the screenshot folder.
//! - `SMOKE_SEED`: the world seed, default 7.
//! - `PROBE_STAGES`: comma list of stage groups, default all. Groups: `water` (the lens tour),
//!   `grass` (turf and trees), `ground` (one patch of grass from the horizon and from 2 m to 30 km), `fauna` (animals), `tour` (base, flight, orbit, warp) and `zoo`.
//!   `ui` (the menus, the map at each level, the guide, the pack and the pause menu) and `play` (the game
//!   being played: the map of explored ground, stones picked and a tool made, a herd stalked, a fight, a
//!   kill carved, the robot hurt and mended, the pack and the guide after the hunt; see the `play` module).
//!   `craft` shoots the pack and the workshop on a staged pack: empty, stones and a hide, each intent,
//!   a card focused and a tool made (see the `workshop` module).
//!   `stars` shoots the star field in space and in orbit (see the `stars` module). `feel` shoots swings at
//!   a small and a large animal in third and first person, a rock struck with H and the wingsuit opened
//!   with a tool drawn (see the `playfeel` module; run it on `SMOKE_SEED=57`).
//!   The title, loading, boot and wake stages always run.
//! - `glide` opens the wingsuit from the top of a jump on flat ground, off a cliff at a run, out of a long fall
//!   and from a jump off the parked ship, three shots each, with `PROBE glide` lines (see the `opening` module).
//! - `entry` (a fall from a low orbit into the air, from a chase camera and from the seat) and `exit`
//!   (the climb from the pad to orbit under the autopilot) shoot the heat of an entry and leaving the air.
//! - `cruise` drops the ship at the arrival shell over the far side of the home planet, presses the home key
//!   and shoots the orbital cruise, the glide into the air and its end near the base, with the HUD shown
//!   (see the `cruise` module). Run it with `SMOKE_SEED=57`.
//! - `SMOKE_ONLY` and `SMOKE_SKIP`: comma lists of stage names to keep or to leave out.
//! - `SMOKE_TARGET` and `SMOKE_FLASH`: see the `warp` and `boot` stages.
//! - `PROBE_JOIN=<address>`: join that server instead of starting a world, with `PROBE_NAME` as the player name
//!   and `PROBE_PASSWORD`. The `mp` stages then meet the other players (see the `mp` module).
//! - `PROBE_UI_SIZE`: `small` or `large` sets the interface size of the run (the Interface size setting),
//!   to shoot `wake` and `menus` at each step.
//! - `PROBE_SOFT`: when set, a system whose parameters fail validation is logged and skipped instead of
//!   stopping the game, so one run lists every such system.
//!
//! `PROBE_STAGES=postcards` takes the clean pictures for the website and the title screen, stages `p01-home`
//! to `p22-river` (see the `postcards` module). It is left out of a run of all groups. Without `SMOKE_SEED` it
//! plays seed 57. `PROBE_SIZE=1920x1080` sets the window size and `POSTCARD_SETTLE` the frames a card waits.
//!
//! `PROBE_STAGES=long` runs the `long-play` stage (see the `long` module): the universe clock days to ten years
//! ahead and a fast flight from 50 m to 30 km, with `PROBE long` lines on the clock, the anchor, the camera
//! planes, the fog, the exposure and the visibility ranges. `PROBE_SAVE=<dir>` starts the run from the most
//! recently played world file in that folder (or in its `worlds` folder), copied to `SMOKE_DIR/probe-save`,
//! with a `settings.toml` beside it applied to the run (see the `saved` module).
//!
//! `PROBE_STAGES=perf` runs the `perf` stages (see the `perf` module): still at the base, walking, boarding,
//! a low fast flight and a hover in orbit, with a `PROBE perf` line every 30 frames and the render pass costs of
//! Bevy's render diagnostics. It is left out of a run of all groups.
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
mod cards_air;
mod cards_foot;
mod cards_land;
mod cards_life;
mod coarse;
mod cruise;
mod entry;
mod feel;
mod foot;
mod grass;
mod ground;
mod herd;
mod hull;
mod isolate;
mod launch;
mod long;
mod longlog;
mod look;
mod meter;
mod models;
mod mp;
mod mpact;
mod mpspot;
mod opening;
mod orbital;
mod perf;
mod perfcount;
mod play;
mod playbook;
mod playcarve;
mod playfeel;
mod playfight;
mod playframe;
mod playherd;
mod playloot;
mod playmap;
mod postcards;
mod saved;
mod screens;
mod seen;
mod shots;
mod stages;
mod stars;
mod stops;
mod tour;
mod trees;
mod util;
mod watch;
mod water;
mod wing;
mod workshop;
mod zoo;

use air::*;
use base::*;
use boot::*;
use burn::*;
use cabin::*;
use camera::*;
use cards_air::*;
use cards_foot::*;
use cards_land::*;
use cards_life::*;
use coarse::*;
use cruise::*;
use entry::*;
use feel::*;
use foot::*;
use grass::*;
use ground::*;
use herd::*;
use hull::*;
use isolate::*;
use launch::*;
use long::*;
use longlog::*;
use look::*;
use meter::*;
use mp::*;
use mpact::*;
use mpspot::*;
use opening::*;
use orbital::*;
use perf::*;
use play::*;
use playbook::*;
use playcarve::*;
use playfeel::*;
use playfight::*;
use playframe::*;
use playherd::*;
use playloot::*;
use playmap::*;
use postcards::*;
use saved::*;
use screens::*;
use seen::*;
use shots::*;
use stages::STAGES;
use stars::*;
use stops::*;
use tour::*;
use trees::*;
use util::*;
use watch::*;
use water::*;
use wing::*;
use workshop::*;

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
    if list.split(',').any(|g| g == "perf") {
        app.add_plugins(bevy::render::diagnostic::RenderDiagnosticsPlugin);
    }
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
    app.init_resource::<Tripod>();
    app.add_systems(Startup, size_window);
    app.add_systems(
        PostUpdate,
        tripod
            .after(bevy::transform::TransformSystems::Propagate)
            .after(hold),
    );
    app.init_resource::<Tris>();
    app.init_resource::<RangeTally>();
    app.add_systems(Last, (measure, count_tris, probe, tally));
    app.run()
}
