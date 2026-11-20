//! The zoo: every animal species of a seed's home planet in a line on flat ground, each walking its
//! own circle, switching walk, trot and gallop every few seconds, under a free orbit camera.
//!
//! Selected with `PROBE_STAGES=zoo`. The seed is `SMOKE_SEED`, default `DEFAULT_SEED`.
//! - With `SMOKE_DIR` set, it saves `walk.png`, `trot.png` and `gallop.png` there, then exits.
//! - With `ZOO_FIND=<count>`, it prints the life of the home planet of seeds 0 to `count`, then exits.
//!
//! Drag with the left mouse button to orbit, wheel to zoom.

mod act;
mod build;

use std::path::PathBuf;
use std::time::Duration;

use abstractions::fauna::motion::Motion;
use abstractions::life::Species;
use abstractions::life::body::Body;
use bevy::prelude::*;
use bevy::time::TimeUpdateStrategy;

use act::*;
use build::*;

/// A seed whose home planet holds skeletal life with walkers, found with `--find`.
pub const DEFAULT_SEED: u64 = 1;

/// Seconds each gait lasts before the next.
pub const GAIT_SECONDS: f32 = 4.0;

/// The gaits in turn: name, Froude number for walkers, body lengths a second for the others.
pub const GAITS: [(&str, f32, f32); 3] = [
    ("walk", 0.25, 0.4),
    ("trot", 1.4, 1.0),
    ("gallop", 3.6, 2.0),
];

/// Animals in one row.
pub const ROW: usize = 8;

/// Fixed step of the shots mode, seconds.
pub const STEP: f32 = 1.0 / 30.0;

/// One animal of the zoo.
pub struct Beast {
    /// Its body.
    pub body: Body,
    /// Its motion.
    pub motion: Motion,
    /// One entity per part, `None` for a part with no mesh.
    pub parts: Vec<Option<Entity>>,
    /// Centre of its circle.
    pub centre: Vec3,
    /// Radius of its circle.
    pub radius: f32,
}

/// The zoo and its clock.
#[derive(Resource)]
pub struct Zoo {
    /// Every animal.
    pub beasts: Vec<Beast>,
    /// Gravity of the home planet.
    pub gravity: f32,
    /// Seconds since the start.
    pub time: f32,
    /// Folder for the shots, in shots mode.
    pub shots: Option<PathBuf>,
    /// Frames since the last shot was asked, once all are asked.
    pub done: u32,
    /// Seed and life summary shown on screen.
    pub title: String,
}

/// The orbit camera: yaw, pitch, distance and the point it looks at.
#[derive(Component)]
pub struct Orbit {
    /// Yaw in radians.
    pub yaw: f32,
    /// Pitch in radians above the horizon.
    pub pitch: f32,
    /// Distance from the target in metres.
    pub distance: f32,
    /// Point looked at.
    pub target: Vec3,
}

/// Marks the text that names the gait.
#[derive(Component)]
pub struct GaitLabel;

/// The species of the zoo, in the order of its beasts.
#[derive(Resource)]
pub struct Kinds(pub Vec<Species>);

/// Opens the zoo of `SMOKE_SEED`, or scans seeds with `ZOO_FIND`.
pub fn run() -> AppExit {
    let seed = std::env::var("SMOKE_SEED")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(DEFAULT_SEED);
    let find_count = std::env::var("ZOO_FIND").ok().and_then(|n| n.parse().ok());
    let shots = std::env::var("SMOKE_DIR").ok().map(PathBuf::from);
    if let Some(count) = find_count {
        find(count);
        return AppExit::Success;
    }
    if let Some(dir) = &shots
        && let Err(e) = std::fs::create_dir_all(dir)
    {
        eprintln!("cannot make {}: {e}", dir.display());
        return AppExit::error();
    }
    let shooting = shots.is_some();
    let (zoo, species) = zoo(seed, shots);
    println!("{}: {} animals", zoo.title, zoo.beasts.len());
    let mut app = App::new();
    app.add_plugins(DefaultPlugins)
        .insert_resource(zoo)
        .insert_resource(Kinds(species))
        .insert_resource(ClearColor(Color::srgb(0.62, 0.72, 0.82)))
        .add_systems(Startup, setup)
        .add_systems(Update, (walk, orbit, shoot).chain());
    if shooting {
        app.insert_resource(TimeUpdateStrategy::ManualDuration(Duration::from_secs_f32(
            STEP,
        )));
    }
    app.run()
}
