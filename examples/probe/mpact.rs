//! The acts of the two games: in each phase one player acts and the other looks.

use std::sync::atomic::Ordering;

use super::*;

/// Seconds the acting player stays crouched.
const CROUCH: f64 = 40.0;

/// Seconds the acting player walks forward.
const WALKING: f64 = 20.0;

/// Seconds into the phase the player first goes to the hatch, and the second it presses the
/// board key.
const HATCH: (f64, f64) = (2.0, 6.0);

/// Seconds into the phase between which the throttle is held.
const THROTTLE: (f64, f64) = (10.0, 30.0);

/// Seconds into the phase at which the acting player presses the key that cuts the engine, and
/// the second it presses the board key to step out.
const OUT: (f64, f64) = (32.0, 40.0);

/// Seconds into the phase of each shot: the canopy up, the actor in the act, the end of it.
const MARKS: [f64; 3] = [4.5, 12.0, 24.0];

/// Sends `code` pressed or released, only when its state changes. `bit` says which key it is.
fn hold(world: &mut World, bit: u32, code: KeyCode, down: bool) {
    let was = HELD.load(Ordering::Relaxed) & (1 << bit) != 0;
    if was == down {
        return;
    }
    key(world, code, down);
    match down {
        true => HELD.fetch_or(1 << bit, Ordering::Relaxed),
        false => HELD.fetch_and(!(1 << bit), Ordering::Relaxed),
    };
}

/// Stands beside the hatch of the ship.
fn beside_hatch(world: &mut World) {
    if let Some(hull) = ship(world) {
        let beside = hull.pos + hull.rot * DVec3::new(-2.6, -1.2, -1.8);
        stand(world, beside, hull.pos, 0.0);
    }
}

/// Does what the acting player does in `action` at `into` seconds of its phase.
fn act(world: &mut World, action: u32, into: f64, frame: u32) {
    let keys = |s: &Settings| {
        (
            s.controls.keys.crouch,
            s.controls.keys.forward,
            s.controls.keys.ship.board,
            s.controls.keys.ship.throttle_up,
            s.controls.keys.ship.cut,
        )
    };
    let (crouch, forward, board, throttle, cut) = keys(world.resource::<Persistent<Settings>>());
    let travel = world.resource::<Location>().travel;
    let tap = frame % 6 < 3;
    let climbs = action >= 2;
    let on_foot = travel == Travel::OnFoot;
    hold(world, 0, crouch, action == 0 && into < CROUCH);
    hold(world, 1, forward, action == 1 && into < WALKING);
    if climbs && on_foot && (HATCH.0..HATCH.1).contains(&into) {
        beside_hatch(world);
    }
    let sits = climbs && on_foot && (HATCH.1..OUT.0).contains(&into) && tap;
    let rises = action == 3 && (OUT.0..OUT.1).contains(&into) && tap;
    let stands = climbs && !on_foot && into >= OUT.1 && tap;
    hold(world, 2, board, sits || stands);
    let thrust = action == 3 && (THROTTLE.0..THROTTLE.1).contains(&into);
    hold(world, 3, throttle, thrust);
    hold(world, 4, cut, rises);
}

/// The script: in each phase one player acts and the other looks. Shots at [`MARKS`] seconds
/// into each phase, named by phase and role. A game that has no start yet waits.
pub fn mp_act(world: &mut World, _: &mut Notes, frame: u32) -> Act {
    if !joined() {
        return Act::Next;
    }
    let Some(seconds) = since().filter(|seconds| *seconds >= 0.0) else {
        return match frame {
            f if f > 60_000 => Act::Fail("the script never started".into()),
            _ => Act::Wait,
        };
    };
    let phase = (seconds / PHASE).floor() as u32;
    if phase >= PHASES {
        return Act::Next;
    }
    let into = seconds - f64::from(phase) * PHASE;
    let acts = (phase % 2 == 1) == second();
    let tick = (into / 2.0) as u32;
    if TICK.swap(phase * 100 + tick, Ordering::Relaxed) != phase * 100 + tick {
        mp_report(world, &format!("phase {phase} {into:.0}s acts {acts}"));
    }
    if acts {
        act(world, phase / 2, into, frame);
    }
    if !acts && frame.is_multiple_of(60) {
        face(world);
    }
    let Some(i) = MARKS.iter().rposition(|m| into >= *m) else {
        return Act::Wait;
    };
    let bit = 1 << (phase * 3 + i as u32);
    if SHOT.fetch_or(bit, Ordering::Relaxed) & bit == 0 {
        let who = ["acts", "sees"][usize::from(!acts)];
        shot(world, &format!("3{phase}-{i}-{}-{who}", role()));
    }
    Act::Wait
}
