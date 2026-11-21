//! Two games on one server: each meets the other, acts in turn and shoots what it sees of the
//! other. Needs `PROBE_JOIN`; without it every stage ends at once.
//!
//! `PROBE_ROLE` is `a` or `b`. `PROBE_T0` is a Unix time in seconds, the same for both games:
//! the acts run in phases of [`PHASE`] seconds from it, so two games that load at different
//! speeds still look at each other doing the same thing.

use std::sync::atomic::{AtomicU32, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

use abstractions::avatar::AvatarPose;
use abstractions::net::{LeaveSession, Players, Remote, Session};

use super::*;

/// Seconds of one phase.
const PHASE: f64 = 60.0;

/// Phases of the script.
const PHASES: u32 = 6;

/// Distance in metres at which the second player stands from the first.
const APART: f64 = 5.0;

/// The shots taken, one bit per phase and mark.
static SHOT: AtomicU32 = AtomicU32::new(0);

/// Returns true when this game joined a server.
fn joined() -> bool {
    std::env::var("PROBE_JOIN").is_ok()
}

/// Returns true when this game plays the role `b`.
fn second() -> bool {
    std::env::var("PROBE_ROLE").is_ok_and(|role| role == "b")
}

/// Returns the seconds since `PROBE_T0`.
fn since() -> f64 {
    let t0 = std::env::var("PROBE_T0")
        .ok()
        .and_then(|t| t.parse::<f64>().ok())
        .unwrap_or(0.0);
    let now = SystemTime::now().duration_since(UNIX_EPOCH).map_or(0.0, |d| d.as_secs_f64());
    now - t0
}

/// Returns the planet position and the pose of the first other robot drawn.
fn remote(world: &mut World) -> Option<(DVec3, AvatarPose)> {
    let anchor = *world.resource::<Anchor>();
    let (at, pose) = world
        .query_filtered::<(&GlobalTransform, &AvatarPose), With<Remote>>()
        .iter(world)
        .next()?;
    Some((anchor.to_frame(at.translation()), *pose))
}

/// Logs what the game sees of the others.
fn report(world: &mut World, label: &str) {
    let me = robot(world);
    let session = world.resource::<Session>().clone();
    let players = world.resource::<Players>().0.clone();
    match remote(world) {
        Some((at, pose)) => info!(
            "MP {label}: {session:?} players {players:?}; remote {:.2} m away, mode {:?} speed {:.2} wings {:.2}",
            at.distance(me),
            pose.mode,
            pose.speed,
            pose.wings
        ),
        None => info!("MP {label}: {session:?} players {players:?}; no remote robot drawn"),
    }
}

/// Turns the player toward the other robot, where it stands.
fn face(world: &mut World) {
    if let Some((at, _)) = remote(world) {
        let me = robot(world);
        let up = me.normalize();
        let pitch = ((at - me).normalize().dot(up).asin() as f32).clamp(-1.2, 1.2);
        stand(world, me, at, pitch);
    }
}

/// Waits for the other robot, then stands [`APART`] metres from the first player, third person.
pub fn mp_meet(world: &mut World, _: &mut Notes, frame: u32) -> Act {
    if !joined() {
        return Act::Next;
    }
    if frame == 1 {
        pace(world, 0.25);
        world.insert_resource(TimeUpdateStrategy::Automatic);
        view(world, ViewMode::Third, ViewMode::Third);
    }
    if frame.is_multiple_of(40) {
        report(world, &format!("meet frame {frame}"));
    }
    if remote(world).is_none() {
        return match frame {
            f if f > 12_000 => Act::Fail("no other player came".into()),
            _ => Act::Wait,
        };
    }
    let Some(base) = world.get_resource::<Base>().copied() else {
        return Act::Fail("no home base".into());
    };
    if second() && frame.is_multiple_of(50) {
        let side = base.wake.normalize().cross(DVec3::Y).normalize_or(DVec3::X);
        let feet = ground(world, base.wake + side * APART, 0.0);
        stand(world, feet, base.wake, 0.0);
    }
    if !second() && frame.is_multiple_of(50) {
        face(world);
    }
    match frame {
        f if f > 200 => Act::Shoot(format!("20-mp-meet-{}", role())),
        _ => Act::Wait,
    }
}

/// Returns the role as a letter.
fn role() -> &'static str {
    ["a", "b"][usize::from(second())]
}

/// The script: in each phase one player acts and the other looks. Shots at 12 s and 30 s into
/// each phase, named by phase and role.
pub fn mp_act(world: &mut World, _: &mut Notes, frame: u32) -> Act {
    if !joined() {
        return Act::Next;
    }
    let seconds = since();
    let phase = (seconds / PHASE).floor().max(0.0) as u32;
    if phase >= PHASES {
        return Act::Next;
    }
    let into = seconds - f64::from(phase) * PHASE;
    let acts = (phase % 2 == 1) == second();
    let forward = bound(world, |s| s.controls.keys.forward);
    let crouch = bound(world, |s| s.controls.keys.crouch);
    let board = bound(world, |s| s.controls.keys.ship.board);
    let action = phase / 2;
    if frame.is_multiple_of(40) {
        report(world, &format!("phase {phase} {into:.0}s acts {acts}"));
    }
    if acts && frame.is_multiple_of(4) {
        match (action, into) {
            (0, t) if t < 40.0 => key(world, crouch, true),
            (0, _) => key(world, crouch, false),
            (1, t) if t < 20.0 => key(world, forward, true),
            (1, _) => key(world, forward, false),
            (2, t) if (2.0..2.6).contains(&t) => beside_hatch(world),
            _ => {}
        }
    }
    if acts && action == 2 && (6.0..7.0).contains(&into) && frame.is_multiple_of(2) {
        key(world, board, into < 6.5);
    }
    if !acts && frame.is_multiple_of(60) {
        face(world);
    }
    let marks = [12.0, 30.0];
    let i = marks.iter().rposition(|m| into >= *m);
    let name = |i: usize| format!("3{phase}-{i}-{}-{}", role(), ["acts", "sees"][usize::from(!acts)]);
    match i {
        Some(i) if SHOT.fetch_or(1 << (phase * 2 + i as u32), Ordering::Relaxed) & (1 << (phase * 2 + i as u32)) == 0 => {
            shot(world, &name(i));
            Act::Wait
        }
        _ => Act::Wait,
    }
}

/// Stands beside the hatch of the ship.
fn beside_hatch(world: &mut World) {
    if let Some(hull) = ship(world) {
        let beside = hull.pos + hull.rot * DVec3::new(-2.6, -1.2, -1.8);
        stand(world, beside, hull.pos, 0.0);
    }
}

/// Reports the last view, leaves the server and shoots the title it returns to.
pub fn mp_leave(world: &mut World, _: &mut Notes, frame: u32) -> Act {
    if !joined() {
        return Act::Next;
    }
    match frame {
        1 => {
            report(world, "end");
            world.write_message(LeaveSession);
            Act::Wait
        }
        60 => {
            report(world, "after leaving");
            Act::Shoot(format!("39-mp-left-{}", role()))
        }
        _ => Act::Wait,
    }
}
