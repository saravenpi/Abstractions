//! The tour: its stage contract, its notes and the loop that runs one stage per frame.

use super::*;

/// Simulated seconds per frame on foot.
pub const WALK: f64 = 1.0 / 30.0;

/// Simulated seconds per frame in flight, as in the voyage test.
pub const FLIGHT: f64 = 0.1;

/// What a stage asks for this frame.
pub enum Act {
    /// Keep running this stage.
    Wait,
    /// Take a screenshot with this name, then go to the next stage.
    Shoot(String),
    /// Go to the next stage without a screenshot.
    Next,
    /// Log why the stage could not run and go to the next one.
    Fail(String),
}

/// Facts found on the way and reused by later stages.
#[derive(Default)]
pub struct Notes {
    /// Planet position of the nearest village origin.
    pub village: Option<DVec3>,
    /// Planet position of the middle of that room on its floor.
    pub room: Option<DVec3>,
    /// Planet position of the threshold of its door, and the unit direction out of it.
    pub door: Option<(DVec3, DVec3)>,
    /// Planet position of a shore point near the base.
    pub shore: Option<DVec3>,
    /// The planet the ship warps to.
    pub far: Option<BodyId>,
    /// The animal looked at, and where the robot stands to look at it.
    pub animal: Option<(Entity, DVec3)>,
    /// The frame at which a stage took its last shot from a held camera: it ends on the next.
    pub shot_at: Option<u32>,
    /// The universe time a night stage left, which it goes back to when it ends.
    pub day: Option<f64>,
    /// What the play stage under way keeps between frames.
    pub play: Play,
    /// The frame of a postcard before the one at which the robot was on foot or seated as it wants.
    pub ready: Option<u32>,
    /// Position and orientation of the ship parked on the landing pad.
    pub berth: Option<(DVec3, DQuat)>,
    /// Planet positions a postcard found on its first frame and uses on the next ones.
    pub spots: Vec<DVec3>,
}

/// One stage of the tour.
pub type Stage = fn(&mut World, &mut Notes, u32) -> Act;

/// Tour state.
#[derive(Resource, Default)]
pub struct Tour {
    pub stage: usize,
    pub frame: u32,
    pub notes: Notes,
    pub begun: Option<Instant>,
    pub worst: Vec<(f32, String)>,
    pub frames: Vec<f32>,
}

impl Tour {
    /// Takes the tour out of the world, leaving an empty one in its place.
    fn clone_out(&mut self) -> Tour {
        std::mem::take(self)
    }

    /// Puts `tour` back.
    fn clone_in(&mut self, tour: Tour) {
        *self = tour;
    }
}

/// Returns whether the stage `name` of `group` runs: `core` stages always do, the others when `PROBE_STAGES`
/// names their group (all groups but `zoo`, `postcards` and `long` when it is empty or `all`), unless `SMOKE_ONLY` leaves the stage out
/// or `SMOKE_SKIP` lists it.
fn wanted(group: &str, name: &str) -> bool {
    let list = std::env::var("PROBE_STAGES").unwrap_or_default();
    let groups = |g: &str| list.split(',').any(|x| x == g);
    let own = |g: &str| g == "zoo" || g == "postcards" || g == "long";
    let by_group =
        group == "core" || groups(group) || (!own(group) && (list.is_empty() || groups("all")));
    let only = std::env::var("SMOKE_ONLY").unwrap_or_default();
    let skipped = std::env::var("SMOKE_SKIP").unwrap_or_default();
    let by_name = group == "core" || only.is_empty() || only.split(',').any(|keep| keep == name);
    by_group && by_name && !skipped.split(',').any(|skip| skip == name)
}

/// Runs the current stage and moves on when it says so.
pub fn run(world: &mut World) {
    let mut tour = world.resource_mut::<Tour>().into_inner().clone_out();
    let Some((name, group, stage)) = STAGES.get(tour.stage).copied() else {
        tour.frame += 1;
        if tour.frame == 30 {
            report(&tour);
            world.write_message(AppExit::Success);
        }
        world.resource_mut::<Tour>().clone_in(tour);
        return;
    };
    tour.frame += 1;
    let act = match !wanted(group, name) {
        true => Act::Next,
        false => stage(world, &mut tour.notes, tour.frame),
    };
    let advance = |tour: &mut Tour, why: &str| {
        summary(tour, name, why);
        (tour.stage, tour.frame) = (tour.stage + 1, 0);
    };
    match act {
        Act::Wait => {}
        Act::Shoot(file) => {
            shot(world, &file);
            advance(&mut tour, &format!("shot {file}"));
        }
        Act::Next => advance(&mut tour, "done"),
        Act::Fail(why) => advance(&mut tour, &format!("FAILED {why}")),
    }
    world.resource_mut::<Tour>().clone_in(tour);
}

/// Logs how stage `name` ended, with its frame times and its worst frames.
pub fn summary(tour: &mut Tour, name: &str, why: &str) {
    let wall = tour
        .begun
        .replace(Instant::now())
        .map_or(0.0, |t| t.elapsed().as_secs_f32());
    let frames = std::mem::take(&mut tour.frames);
    let mean = frames.iter().sum::<f32>() / frames.len().max(1) as f32;
    let max = frames.iter().copied().fold(0.0, f32::max);
    info!(
        "SMOKE stage {name}: {why}; {} frames in {wall:.1} s wall, main thread mean {mean:.1} ms max {max:.1} ms",
        tour.frame
    );
    let mut worst = std::mem::take(&mut tour.worst);
    worst.sort_by(|a, b| b.0.total_cmp(&a.0));
    for (millis, what) in worst.iter().take(3) {
        info!("SMOKE   slow frame {millis:.1} ms, {what}");
    }
}

/// Logs the worst frames of the whole run.
pub fn report(_: &Tour) {
    info!("SMOKE steps timed: {METER_STEPS:?}");
}
