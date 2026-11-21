//! The model studio: the robot and the companion alone on flat ground in white light, from fixed
//! views and in their main poses, one shot each.
//!
//! Selected with `PROBE_STAGES=models`. With `SMOKE_DIR` set it saves the shots there and exits.
//! `MODELS_ONLY` keeps the shots whose name contains one of its comma-separated words.
//! The robot and the companion are drawn by the real game systems from their pose components, with
//! nothing else of the game around them, so a run takes seconds instead of minutes.

mod plan;
mod set;

use std::path::PathBuf;
use std::time::Duration;

use bevy::prelude::*;
use bevy::render::view::screenshot::{Screenshot, save_to_disk};
use bevy::time::TimeUpdateStrategy;

use plan::{Shot, shots};

/// Fixed step of the studio, seconds.
pub const STEP: f32 = 1.0 / 30.0;

/// Frames a pose is held before its shot, so the blend between poses has settled.
pub const SETTLE: u32 = 36;

/// The shots still to take, the one in hand and the frames it has waited.
#[derive(Resource)]
pub struct Plan {
    /// Shots not yet taken, the next last.
    pub todo: Vec<Shot>,
    /// Frames waited since the current shot was set up.
    pub waited: u32,
    /// Folder the shots go to.
    pub dir: PathBuf,
}

/// Opens the studio and runs the plan.
pub fn run() -> AppExit {
    let dir = PathBuf::from(std::env::var("SMOKE_DIR").unwrap_or_else(|_| "models".into()));
    if let Err(e) = std::fs::create_dir_all(&dir) {
        eprintln!("cannot make {}: {e}", dir.display());
        return AppExit::error();
    }
    let mut todo = shots();
    if let Ok(only) = std::env::var("MODELS_ONLY") {
        todo.retain(|s| only.split(',').any(|w| s.name.contains(w)));
    }
    todo.reverse();
    let mut app = App::new();
    app.add_plugins(DefaultPlugins)
        .insert_resource(Plan {
            todo,
            waited: 0,
            dir,
        })
        .insert_resource(TimeUpdateStrategy::ManualDuration(Duration::from_secs_f32(
            STEP,
        )))
        .insert_resource(ClearColor(Color::srgb(0.62, 0.72, 0.82)))
        .add_systems(Startup, (set::stage, set::robot, set::buddy).chain());
    abstractions::avatar::studio(&mut app);
    abstractions::companion::studio(&mut app);
    app.add_systems(
        Update,
        (set::advance, drive)
            .chain()
            .before(abstractions::avatar::AvatarSet::Animate),
    );
    app.run()
}

/// Takes the shot in hand once its pose has settled, then sets up the next, and exits when all are written.
fn drive(mut commands: Commands, mut plan: ResMut<Plan>, mut exit: MessageWriter<AppExit>) {
    plan.waited += 1;
    if plan.waited < SETTLE {
        return;
    }
    if plan.waited == SETTLE
        && let Some(shot) = plan.todo.last()
    {
        let path = plan.dir.join(format!("{}.png", shot.name));
        commands
            .spawn(Screenshot::primary_window())
            .observe(save_to_disk(path));
    }
    if plan.waited > SETTLE + 8 {
        plan.todo.pop();
        plan.waited = 0;
        if plan.todo.is_empty() {
            exit.write(AppExit::Success);
        }
    }
}
