//! The two games share a stone: after the acts, `a` puts 2 kg of basalt down, `b` sees it on the
//! ground, walks to it and takes it with the interact key, and `a` sees it gone. Each logs a line
//! `MP share` the script checks, and shoots what it sees.

use std::sync::atomic::{AtomicU64, Ordering};

use abstractions::craft::Pack;
use abstractions::craft::ground::{Ground, Shed, Thing};
use abstractions::craft::matter::{Mineral, Stuff};
use abstractions::craft::pack::Stack;
use abstractions::net::Link;
use abstractions::player::{Interact, Verb};

use super::*;

/// Seconds after the start of the script the sharing begins: once every act is over.
const SHARE: f64 = PHASE * PHASES as f64;

/// Seconds the sharing may last before a game gives up.
const SHARE_FOR: f64 = 120.0;

/// Kilograms of basalt `a` puts down.
const STONE: f32 = 2.0;

/// The steps done, one bit each: dropped, seen, pressed, gone or taken.
static DONE: AtomicU64 = AtomicU64::new(0);

/// Returns true the first time step `bit` is done, and marks it done.
fn first(bit: u64) -> bool {
    DONE.fetch_or(1 << bit, Ordering::Relaxed) & (1 << bit) == 0
}

/// Returns the id, place and mass of the first basalt that lies, put down by a player that is not
/// `mine` when `others` is true, else by this one.
fn basalt(world: &World, others: bool) -> Option<(u64, DVec3, f32)> {
    let me = world.get_resource::<Link>().and_then(|link| link.you);
    let stone = abstractions::craft::ground::thing;
    let pile = &world.resource::<Ground>().pile;
    pile.lying
        .iter()
        .filter(|l| (Some(l.by) != me) == others)
        .find(|l| matches!(stone(l), Some(Thing::Heap(s)) if s.stuff == Stuff::Mineral(Mineral::Basalt)))
        .map(|l| (l.id, DVec3::from(l.at), l.mass))
}

/// Puts the basalt down once, as the pack screen does: into the shed, ahead of the robot.
fn drop_stone(world: &mut World) {
    if !first(0) {
        return;
    }
    let stone = Stack {
        stuff: Stuff::Mineral(Mineral::Basalt),
        material: Mineral::Basalt.material(),
        mass: STONE,
    };
    world.resource_mut::<Shed>().0.push(Thing::Heap(stone));
    info!("MP share a dropped {STONE} kg of basalt");
}

/// Plays `a`: puts the stone down, sees it lie, then sees it gone and shoots it.
fn giver(world: &mut World, into: f64) -> Act {
    if into >= 2.0 {
        drop_stone(world);
    }
    match basalt(world, false) {
        Some((id, _, mass)) if first(1) => {
            info!("MP share a sees id {id} lying, {mass:.2} kg");
            shot(world, &format!("40-mp-share-{}-lying", role()));
        }
        None if DONE.load(Ordering::Relaxed) & 2 != 0 && first(3) => {
            info!("MP share a sees the stone gone");
            return Act::Shoot(format!("40-mp-share-{}", role()));
        }
        _ => {}
    }
    Act::Wait
}

/// Stands `b` a metre from the stone at `at`, through its own eyes looking down at it, and presses
/// the interact key while the prompt names the take.
fn reach_for(world: &mut World, at: DVec3, frame: u32) {
    let up = at.normalize();
    let side = up.cross(DVec3::X).normalize_or(DVec3::Z);
    if frame.is_multiple_of(30) {
        view(world, ViewMode::First, ViewMode::Third);
        let feet = ground(world, at + side, 0.0);
        let eye = feet + up * 1.5;
        let pitch = (at - eye).normalize().dot(up).asin() as f32;
        stand(world, feet, at, pitch);
    }
    let board = bound(world, |s| s.controls.keys.ship.board);
    let offered = world.resource::<Interact>().is(Verb::Take);
    key(world, board, offered && frame % 6 < 3);
    if offered && first(2) {
        info!("MP share b is offered the take");
    }
}

/// Returns the kilograms of basalt in the pack.
fn carried(world: &World) -> f32 {
    let pack = world.resource::<Pack>();
    let basalt = pack
        .stacks
        .iter()
        .filter(|s| s.stuff == Stuff::Mineral(Mineral::Basalt));
    basalt.map(|s| s.mass).sum()
}

/// Plays `b`: waits for the stone of `a`, goes to it and takes it, then shoots the pack full.
fn taker(world: &mut World, frame: u32) -> Act {
    let had = carried(world);
    match basalt(world, true) {
        Some((id, at, mass)) => {
            if first(1) {
                info!("MP share b sees id {id} lying, {mass:.2} kg");
                shot(world, &format!("40-mp-share-{}-sees", role()));
            }
            reach_for(world, at, frame);
        }
        None if DONE.load(Ordering::Relaxed) & 2 != 0 && had >= STONE - 0.01 && first(3) => {
            info!("MP share b took the stone into the pack: {had:.2} kg of basalt");
            return Act::Shoot(format!("40-mp-share-{}", role()));
        }
        None => {}
    }
    Act::Wait
}

/// The sharing of a stone between the two games, after the acts. A game that has no start yet, or
/// whose sharing is over, goes on.
pub fn mp_share(world: &mut World, _: &mut Notes, frame: u32) -> Act {
    if !joined() || DONE.load(Ordering::Relaxed) & 8 != 0 {
        return Act::Next;
    }
    let Some(into) = since().map(|seconds| seconds - SHARE) else {
        return Act::Next;
    };
    if into < 0.0 {
        return Act::Wait;
    }
    if into > SHARE_FOR {
        info!("MP share {} gave up after {SHARE_FOR} s", role());
        return Act::Next;
    }
    match second() {
        true => taker(world, frame),
        false => giver(world, into),
    }
}
