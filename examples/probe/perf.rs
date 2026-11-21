//! The `perf` stages: the robot holds still at the base, walks, boards, the ship flies low and
//! fast, then hovers in orbit. Every [`EVERY`] frames each logs a `PROBE perf` line with the frame
//! time, the entities, what is in view and its triangles, the lights that cast shadows, the
//! interface writes, and with the render diagnostics the cost of each render pass.

use super::perfcount::{Perf, log_perf, note_frame};
use super::*;

/// Frames between two log lines: a second of play at the pace of the probe on foot.
const EVERY: u32 = 30;

/// Frames each `perf` stage lasts.
const LENGTH: u32 = 180;

/// Height in metres over the ground of the low flight.
const LOW: f64 = 60.0;

/// Metres the low flight covers per frame: 180 m/s at the pace of the probe.
const STRIDE: f64 = 6.0;

/// Counts the frame, logs a line every [`EVERY`] frames and ends the stage after [`LENGTH`].
fn watch(world: &mut World, phase: &str, frame: u32) -> Act {
    if world.get_resource::<Perf>().is_none() {
        world.init_resource::<Perf>();
    }
    note_frame(world);
    if frame.is_multiple_of(EVERY) {
        log_perf(world, phase, (frame, frame >= LENGTH));
    }
    match frame >= LENGTH {
        true => Act::Shoot(format!("perf-{phase}")),
        false => Act::Wait,
    }
}

/// First person, standing still at the wake point of the base, looking at the pad.
pub fn perf_base(world: &mut World, _: &mut Notes, frame: u32) -> Act {
    let Some(base) = world.get_resource::<Base>().copied() else {
        return Act::Fail("no home base".into());
    };
    if frame == 1 {
        view(world, ViewMode::First, ViewMode::First);
        let feet = ground(world, base.wake, 0.0);
        stand(world, feet, base.pad, -0.05);
    }
    watch(world, "base", frame)
}

/// Walking forward from there.
pub fn perf_walk(world: &mut World, _: &mut Notes, frame: u32) -> Act {
    let forward = bound(world, |s| s.controls.keys.forward);
    if frame == 1 {
        key(world, forward, true);
    }
    let act = watch(world, "walk", frame);
    if !matches!(act, Act::Wait) {
        key(world, forward, false);
    }
    act
}

/// The ship flies [`LOW`] metres over the ground at 180 m/s along a great circle, nose a little
/// down. Needs the player aboard: the `perf-board` stage runs before it.
pub fn perf_low(world: &mut World, _: &mut Notes, frame: u32) -> Act {
    if world.get_resource::<Perf>().is_none() {
        world.init_resource::<Perf>();
    }
    let start = match world.resource::<Perf>().start {
        Some(start) => start,
        None => {
            let at = player_at(world).normalize_or(DVec3::Y);
            world.resource_mut::<Perf>().start = Some(at);
            at
        }
    };
    let radius = world.resource::<Planet>().globe.radius();
    let axis = start.any_orthonormal_vector();
    let angle = f64::from(frame) * STRIDE / radius;
    let dir = DQuat::from_axis_angle(axis, angle) * start;
    let height = ground(world, dir, LOW).length() - radius;
    hover(world, dir, height, (axis.cross(dir), 0.1));
    watch(world, "low", frame)
}

/// The ship hovers above the air, nose down at the planet.
pub fn perf_orbit(world: &mut World, _: &mut Notes, frame: u32) -> Act {
    if frame == 1 {
        let at = player_at(world).normalize_or(DVec3::Y);
        let body = world.resource::<Location>().body;
        let high = world.resource::<Universe>().planet(body).atmosphere_height * 1.6;
        hover(world, at, high, (at.any_orthonormal_vector(), 1.2));
    }
    watch(world, "orbit", frame)
}
