//! The hard end of the loading screen: a world that does not get ready is not waited for
//! forever. First the body of a resumed world is stood on the ground under it, then the game
//! starts with what is loaded. Either way the player is told why once play begins.

use bevy::prelude::*;

use super::{Screen, in_control};
use crate::net::SessionNotice;
use crate::saves::ActiveWorld;
use crate::world::{Resettled, WorldReady};

/// Seconds of loading after which the body of a resumed world is stood on the ground under it.
pub const LAND_AFTER: f32 = 60.0;

/// Seconds of loading after which the game starts with what is loaded.
pub const START_AFTER: f32 = 120.0;

/// Most seconds one frame counts for, so a frame that blocked on generation is not taken for
/// a world that does not load.
const FRAME_MOST: f32 = 0.25;

/// What the player is told when the body was stood on the ground under its saved place.
pub const LANDED: &str =
    "The place you were saved at did not load. You are on the ground below it.";

/// What the player is told when the game started before the world was ready.
pub const STARTED: &str = "The world took too long to load. Some ground may still be missing.";

/// What the loading screen does about a world that is not ready yet.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Step {
    /// Keep waiting.
    Wait,
    /// Stand the body on the ground under it.
    Land,
    /// Start the game with what is loaded.
    Start,
}

/// How long the world has been loading without getting ready.
#[derive(Resource, Debug, Default, Clone, Copy, PartialEq)]
pub struct Stall {
    /// Seconds of loading, each frame counted for at most [`FRAME_MOST`].
    pub(super) seconds: f32,
    /// True once the body was stood on the ground.
    pub(super) landed: bool,
}

impl Stall {
    /// Counts a frame of `delta` seconds and returns what to do now: land once after
    /// [`LAND_AFTER`], start from [`START_AFTER`] on.
    pub fn after(&mut self, delta: f32) -> Step {
        self.seconds += delta.clamp(0.0, FRAME_MOST);
        if self.seconds >= START_AFTER {
            return Step::Start;
        }
        if self.seconds >= LAND_AFTER && !self.landed {
            self.landed = true;
            return Step::Land;
        }
        Step::Wait
    }
}

/// Run condition: the world loaded for [`START_AFTER`], so the loading screen ends whatever
/// the body still waits for.
pub fn expired(stall: Res<Stall>) -> bool {
    stall.seconds >= START_AFTER
}

/// Starts counting again for a new loading screen.
fn reset(mut stall: ResMut<Stall>) {
    *stall = Stall::default();
}

/// Counts the loading time of a world that is not ready and ends the wait: the body of a
/// resumed world is stood on the ground under it, and later the world is flagged ready as it is.
pub(super) fn watch(
    time: Res<Time<Real>>,
    mut stall: ResMut<Stall>,
    ready: Option<ResMut<WorldReady>>,
    active: Option<Res<ActiveWorld>>,
    mut commands: Commands,
) {
    let Some(mut ready) = ready.filter(|ready| !ready.0) else {
        return;
    };
    let resumed = active.is_some_and(|active| active.0.save.place.is_some());
    match stall.after(time.delta_secs()) {
        Step::Land if resumed => {
            commands.queue(|world: &mut World| crate::player::land(world));
            commands.insert_resource(Resettled(LANDED));
        }
        Step::Start => {
            ready.0 = true;
            commands.insert_resource(Resettled(STARTED));
        }
        _ => {}
    }
}

/// Tells the player, in the banner, why the body is not where the save left it, once.
fn tell(line: Res<Resettled>, mut notices: MessageWriter<SessionNotice>, mut commands: Commands) {
    notices.write(SessionNotice {
        line: line.0.to_string(),
    });
    commands.remove_resource::<Resettled>();
}

/// Registers the count of the loading time and the line the player is told.
pub(super) fn plugin(app: &mut App) {
    app.init_resource::<Stall>()
        .add_systems(OnEnter(Screen::Loading), reset)
        .add_systems(
            Update,
            (
                watch.run_if(in_state(Screen::Loading)),
                tell.run_if(resource_exists::<Resettled>).run_if(in_control),
            ),
        );
}
