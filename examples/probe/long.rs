//! The long play: what hours of play leave behind, packed into a minute. A world resumed with
//! `PROBE_SAVE` is first watched as it was saved; then the universe clock jumps a day, ten days, a
//! hundred days, a year and ten years ahead, and the ship flies fast over the planet from 50 m to
//! 30 km and back. Every 30 frames a `PROBE long` line logs the clock, the anchor, the camera
//! planes, the fog, the exposure and the distinct visibility ranges handed to Bevy so far: see
//! the `longlog` module.

use super::*;

/// Frames a resumed world is watched as it was saved.
const STAY: u32 = 300;

/// Frames given to boarding the ship.
const BOARD: u32 = 90;

/// Seconds the universe clock jumps ahead by at each step, and the name of the step.
const CLOCKS: [(f64, &str); 6] = [
    (0.0, "now"),
    (86_400.0, "day"),
    (864_000.0, "10days"),
    (8_640_000.0, "100days"),
    (31_557_600.0, "year"),
    (315_576_000.0, "10years"),
];

/// Frames each clock step is held.
const HOLD: u32 = 150;

/// Frames of the fast flight.
const FLY: u32 = 1800;

/// Metres the ship moves over the ground each frame of the flight: 2 km/s at the flight pace.
const STRIDE: f64 = 200.0;

/// Frames between two log lines.
const EVERY: u32 = 30;

/// The stage: watches, boards, ages the clock and flies, logging all along.
pub fn long_play(world: &mut World, notes: &mut Notes, frame: u32) -> Act {
    if frame.is_multiple_of(EVERY) {
        long_report(world, frame);
    }
    let stay = if std::env::var("PROBE_SAVE").is_ok() {
        STAY
    } else {
        0
    };
    let clocks = HOLD * CLOCKS.len() as u32;
    match frame {
        f if f <= stay => held(world, f),
        f if f <= stay + BOARD => seat(world, notes, f - stay),
        f if f <= stay + BOARD + clocks => age(world, notes, f - stay - BOARD),
        f if f <= stay + BOARD + clocks + FLY => fly(world, notes, f - stay - BOARD - clocks),
        _ => Act::Shoot("long-30-after".into()),
    }
}

/// Watches a resumed world where it was saved, and shoots it twice.
fn held(world: &mut World, frame: u32) -> Act {
    if frame == STAY / 2 || frame == STAY {
        hygiene(world, "long play, as saved");
        shot(world, &format!("long-00-saved-{frame}"));
    }
    Act::Wait
}

/// Seats the player in the ship unless already there, then puts the ship 500 m up and notes
/// where it started in `notes.shore`.
fn seat(world: &mut World, notes: &mut Notes, frame: u32) -> Act {
    if world.resource::<Location>().travel == Travel::OnFoot
        && let Act::Fail(why) = board(world, notes, frame)
    {
        return Act::Fail(why);
    }
    if frame == BOARD {
        pace(world, FLIGHT);
        let pad = world.get_resource::<Base>().map_or(DVec3::Y, |b| b.pad);
        let at = ship(world)
            .filter(|s| matches!(s.frame, Frame::Surface(_)))
            .map_or(pad, |s| s.pos);
        notes.shore = Some(at.normalize());
        hover(world, at, 500.0, (at.any_orthonormal_vector(), 0.2));
    }
    Act::Wait
}

/// Jumps the universe clock ahead step by step from where it was, shoots each step, and puts the
/// clock back when the steps are done.
fn age(world: &mut World, notes: &mut Notes, frame: u32) -> Act {
    let now = world.resource::<abstractions::space::Clock>().seconds;
    let start = *notes.day.get_or_insert(now);
    let (step, local) = (((frame - 1) / HOLD) as usize, (frame - 1) % HOLD);
    let (ahead, name) = CLOCKS[step.min(CLOCKS.len() - 1)];
    if local == 0 {
        world.resource_mut::<abstractions::space::Clock>().seconds = start + ahead;
        eprintln!(
            "PROBE long clock set to {:.0} s, {name} ahead",
            start + ahead
        );
    }
    if local == HOLD - 1 {
        hygiene(world, &format!("long play, clock {name}"));
        shot(world, &format!("long-1{step}-clock-{name}"));
    }
    if frame == HOLD * CLOCKS.len() as u32 {
        world.resource_mut::<abstractions::space::Clock>().seconds = start;
        notes.day = None;
    }
    Act::Wait
}

/// Flies the ship fast along a great circle from where it started, climbing to 30 km and back
/// down to 50 m, with a metre of jitter in height every frame, as a hand on the stick gives.
fn fly(world: &mut World, notes: &mut Notes, frame: u32) -> Act {
    let start = notes.shore.unwrap_or(DVec3::Y);
    let radius = world.resource::<Planet>().globe.radius();
    let axis = start.any_orthonormal_vector();
    let angle = f64::from(frame) * STRIDE / radius;
    let dir = DQuat::from_axis_angle(axis, angle) * start;
    let rise = (std::f64::consts::PI * f64::from(frame) / f64::from(FLY)).sin();
    let jitter = if frame.is_multiple_of(2) { 1.0 } else { -1.0 };
    hover(
        world,
        dir,
        50.0 + 30_000.0 * rise * rise + jitter,
        (axis.cross(dir), 0.2),
    );
    if frame == FLY / 2 || frame == FLY {
        hygiene(world, "long play, flight");
        shot(world, &format!("long-2{}-fly", u32::from(frame == FLY)));
    }
    Act::Wait
}
