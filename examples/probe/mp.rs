//! Two games on one server: each meets the other, acts in turn and shoots what it sees of the
//! other. Needs `PROBE_JOIN`; without it every stage ends at once.
//!
//! `PROBE_ROLE` is `a` or `b`. `PROBE_SYNC` is a folder both games share. Each writes
//! `ready-<role>` there once it has met the other, and waits for the file `t0` the script writes
//! when both are ready: a Unix time in seconds. The acts run in phases of [`PHASE`] seconds from
//! it, so two games that load at different speeds still look at each other doing the same thing.
//! `PROBE_T0`, a Unix time, stands in for the file.

use std::sync::atomic::AtomicU32;
use std::time::{SystemTime, UNIX_EPOCH};

use abstractions::avatar::AvatarPose;
use abstractions::net::{LeaveSession, Players, Remote, Session};
use abstractions::ship::Engine;
use abstractions::ship::cockpit::Canopy;

use super::*;

/// Seconds of one phase.
pub const PHASE: f64 = 60.0;

/// Phases of the script.
pub const PHASES: u32 = 8;

/// Distance in metres at which the second player stands from the first.
const APART: f64 = 5.0;

/// The shots taken, one bit per phase and mark.
pub static SHOT: AtomicU32 = AtomicU32::new(0);

/// The keys held down by the script, one bit per key.
pub static HELD: AtomicU32 = AtomicU32::new(0);

/// The two-second tick of the last report of the acts.
pub static TICK: AtomicU32 = AtomicU32::new(u32::MAX);

/// Returns true when this game joined a server.
pub fn joined() -> bool {
    std::env::var("PROBE_JOIN").is_ok()
}

/// Returns true when this game plays the role `b`.
pub fn second() -> bool {
    std::env::var("PROBE_ROLE").is_ok_and(|role| role == "b")
}

/// Returns the folder the two games share.
pub fn sync() -> std::path::PathBuf {
    std::path::PathBuf::from(std::env::var("PROBE_SYNC").unwrap_or_else(|_| ".".into()))
}

/// Returns the start of the script: the file `t0` of the shared folder, else `PROBE_T0`.
pub fn start() -> Option<f64> {
    let text = std::fs::read_to_string(sync().join("t0"))
        .ok()
        .or_else(|| std::env::var("PROBE_T0").ok())?;
    text.trim().parse().ok()
}

/// Returns the seconds since the start of the script, or nothing while it has no start.
pub fn since() -> Option<f64> {
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(0.0, |d| d.as_secs_f64());
    Some(now - start()?)
}

/// Returns the planet position and the pose of the first other robot drawn.
pub fn remote(world: &mut World) -> Option<(DVec3, AvatarPose)> {
    let anchor = *world.get_resource::<Anchor>()?;
    let (at, pose) = world
        .query_filtered::<(&GlobalTransform, &AvatarPose), With<Remote>>()
        .iter(world)
        .next()?;
    Some((anchor.to_frame(at.translation()), *pose))
}

/// Returns the highest engine level shown on the ships of the other players.
fn exhaust(world: &mut World) -> f32 {
    let mut engines = world.query_filtered::<&Engine, With<Remote>>();
    engines
        .iter(world)
        .map(|engine| engine.level)
        .fold(0.0, f32::max)
}

/// Returns the widest opening of a canopy in the world, in radians.
fn canopy(world: &mut World) -> f32 {
    let mut hinges = world.query::<&Canopy>();
    hinges
        .iter(world)
        .map(|hinge| hinge.angle)
        .fold(0.0, f32::max)
}

/// Logs where the player stands and what the ship is, one line a check can compare.
pub fn spot(world: &mut World, label: &str) {
    let me = robot(world);
    let radius = world
        .get_resource::<Planet>()
        .map_or(0.0, |planet| planet.globe.radius());
    let travel = world
        .get_resource::<Location>()
        .map(|location| location.travel);
    let hull = ship(world).map_or("none".to_string(), |hull| {
        format!("frame {:?} landed {}", hull.frame, hull.landed)
    });
    info!(
        "MP spot {label}: pos {:.1} {:.1} {:.1} altitude {:.1} travel {travel:?} ship {hull}",
        me.x,
        me.y,
        me.z,
        me.length() - radius
    );
}

/// Logs what the game sees of the others.
pub fn mp_report(world: &mut World, label: &str) {
    if world.get_resource::<Anchor>().is_none() {
        info!("MP {label}: no anchor yet");
        return;
    }
    let me = robot(world);
    let session = world.resource::<Session>().clone();
    let players = world.resource::<Players>().0.clone();
    let (travel, lift, open) = (
        world.resource::<Location>().travel,
        exhaust(world),
        canopy(world),
    );
    let mine = format!("me {travel:?} exhaust {lift:.2} canopy {open:.2}");
    match remote(world) {
        Some((at, pose)) => info!(
            "MP {label}: {session:?} players {players:?}; remote {:.2} m away, mode {:?} speed {:.2} crouch {:.2} wings {:.2}; {mine}",
            at.distance(me),
            pose.mode,
            pose.speed,
            pose.crouch,
            pose.wings
        ),
        None => info!("MP {label}: {session:?} players {players:?}; no remote robot drawn; {mine}"),
    }
}

/// Turns the player toward the other robot, where it stands.
pub fn face(world: &mut World) {
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
        mp_report(world, &format!("meet frame {frame}"));
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
        f if f > 200 => {
            let _ = std::fs::write(sync().join(format!("ready-{}", role())), "ready");
            info!("MP ready: {} met the other player", role());
            Act::Shoot(format!("20-mp-meet-{}", role()))
        }
        _ => Act::Wait,
    }
}

/// Returns the role as a letter.
pub fn role() -> &'static str {
    ["a", "b"][usize::from(second())]
}

/// Reports the last view, leaves the server and shoots the title it returns to.
pub fn mp_leave(world: &mut World, _: &mut Notes, frame: u32) -> Act {
    if !joined() {
        return Act::Next;
    }
    match frame {
        1 => {
            mp_report(world, "end");
            spot(world, "before leaving");
            world.write_message(LeaveSession);
            Act::Wait
        }
        60 => {
            mp_report(world, "after leaving");
            Act::Shoot(format!("39-mp-left-{}", role()))
        }
        _ => Act::Wait,
    }
}
