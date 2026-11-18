//! Intents drive the body: the same moves from the keys and from injected intents, and a
//! second player moved by its own intents only.

use bevy::prelude::*;

use super::stage::{app, body};
use crate::player::asks::gather;
use crate::player::controller::drive;
use crate::player::eyes::aim;
use crate::player::keys::send;
use crate::settings::Keymap;
use crate::sim::{Intent, IntentOf, PlayerId};
use crate::view::{Look, Target, ViewMode, ViewPrefs};

/// What the player does on one frame of the script.
#[derive(Debug, Clone, Copy, Default)]
struct Plan {
    /// Forward and strafe to the right, each -1, 0 or 1.
    axes: Vec2,
    /// Whether the jump key is held, the run key held, the wings key pressed this frame.
    keys: (bool, bool, bool),
    /// Turn of the mouse this frame, in radians.
    turn: f32,
    /// Whether the body is seen from behind.
    behind: bool,
}

/// Returns the script for `frame`: walks, a run, turns, a jump, the boots, the wings, a walk back.
fn plan(frame: usize) -> Plan {
    let axes = match frame {
        0..60 => Vec2::X,
        60..120 => Vec2::new(1.0, 1.0),
        120..200 => Vec2::new(1.0, -1.0),
        250..300 => Vec2::new(-1.0, -1.0),
        _ => Vec2::ZERO,
    };
    let held = |from: usize, to: usize| (from..to).contains(&frame);
    Plan {
        axes,
        keys: (
            held(90, 96) || held(200, 206) || held(214, 240),
            held(60, 100),
            frame == 222,
        ),
        turn: 0.02 * f32::from(u8::from(held(130, 170))),
        behind: frame >= 150,
    }
}

/// Holds the keys of `plan` the way the keyboard would, and moves the mouse by its turn.
fn press(app: &mut App, plan: Plan) {
    let map = Keymap::default();
    let world = app.world_mut();
    let mut keys = world.resource_mut::<ButtonInput<KeyCode>>();
    keys.clear();
    let held = [
        (map.forward, plan.axes.x > 0.0),
        (map.back, plan.axes.x < 0.0),
        (map.right, plan.axes.y > 0.0),
        (map.left, plan.axes.y < 0.0),
        (map.jump, plan.keys.0),
        (map.sprint, plan.keys.1),
        (map.wings, plan.keys.2),
    ];
    for (key, down) in held {
        match down {
            true => keys.press(key),
            false => keys.release(key),
        }
    }
    world.resource_mut::<Look>().yaw_delta = plan.turn;
    let mode = [ViewMode::First, ViewMode::Third][usize::from(plan.behind)];
    world.resource_mut::<ViewPrefs>().on_foot = mode;
}

/// Sends what `plan` asks as intents of `player`, built by hand instead of read from keys.
fn tell(app: &mut App, player: PlayerId, plan: Plan) {
    let look = Intent::Look {
        yaw: plan.turn,
        pitch: 0.0,
        turn: plan.turn,
        behind: plan.behind,
    };
    let (forward, strafe) = (plan.axes.x, plan.axes.y);
    let held = [
        Some(Intent::Move { forward, strafe }),
        Some(look),
        plan.keys.0.then_some(Intent::Jump),
        plan.keys.1.then_some(Intent::Boost),
        plan.keys.2.then_some(Intent::Wings),
    ];
    for intent in held.into_iter().flatten() {
        app.world_mut().write_message(IntentOf { player, intent });
    }
}

/// Runs one frame of `plan` from the keys in `keys` and from intents in `told`, checks that
/// both local bodies are in the same state bit for bit, and returns the keyed body's height
/// and whether its boots carry it.
fn both(keys: &mut App, told: &mut App, plan: Plan) -> (f32, bool) {
    press(keys, plan);
    tell(told, PlayerId::LOCAL, plan);
    keys.update();
    told.update();
    let (a, b) = (body(keys, PlayerId::LOCAL), body(told, PlayerId::LOCAL));
    let bits = |at: Vec3| at.to_array().map(f32::to_bits);
    assert_eq!(bits(a.0.translation), bits(b.0.translation));
    assert_eq!(bits(a.1.velocity), bits(b.1.velocity));
    assert!(a.1.velocity.is_finite() && a.0.rotation == b.0.rotation);
    assert_eq!(
        (a.1.yaw.to_bits(), a.1.boots),
        (b.1.yaw.to_bits(), b.1.boots)
    );
    (a.0.translation.y, a.2.state.mode.airborne() && a.1.boots)
}

/// Over 300 frames of walking, running, turning, jumping, firing the boots and spreading the
/// wings, the body moved by the keys and the body moved by intents built by hand are in the
/// same place, bit for bit, on every frame.
#[test]
fn intents_drive_the_body_exactly_as_the_keys_did() {
    let local = [(PlayerId::LOCAL, 0.0)];
    let (mut keys, mut told) = (app(&local), app(&local));
    keys.add_systems(Update, (send, gather, drive).chain());
    told.add_systems(Update, (gather, drive).chain());
    let start = body(&mut keys, PlayerId::LOCAL).0.translation;
    let (mut top, mut flew) = (f32::MIN, false);
    for frame in 0..300 {
        let (height, boosted) = both(&mut keys, &mut told, plan(frame));
        (top, flew) = (top.max(height - start.y), flew || boosted);
    }
    let walked = body(&mut keys, PlayerId::LOCAL)
        .0
        .translation
        .distance(start);
    let shown = format!("walked {walked} m, rose {top} m, flew {flew}");
    assert!(walked > 20.0 && top > 1.0 && flew, "{shown}");
}

/// A second player's body moves by its own intents only: the keys move the local body, the
/// camera follows the local body, and neither touches the other one.
#[test]
fn a_second_player_moves_by_its_own_intents() {
    let other = PlayerId(1);
    let mut app = app(&[(PlayerId::LOCAL, 0.0), (other, 20.0)]);
    app.add_systems(Update, (send, gather, drive, aim).chain());
    let at = |app: &mut App, id| body(app, id).0.translation;
    let (mine, theirs) = (at(&mut app, PlayerId::LOCAL), at(&mut app, other));
    let (ahead, aside) = (Vec2::X, Vec2::Y);
    for _ in 0..60 {
        press(
            &mut app,
            Plan {
                axes: ahead,
                ..Plan::default()
            },
        );
        tell(
            &mut app,
            other,
            Plan {
                axes: aside,
                ..Plan::default()
            },
        );
        app.update();
    }
    let now = at(&mut app, PlayerId::LOCAL);
    let (went, slid) = (now - mine, at(&mut app, other) - theirs);
    assert!(went.z < -5.0 && went.x.abs() < 0.05, "local {went}");
    assert!(slid.x > 5.0 && slid.z.abs() < 0.05, "other {slid}");
    assert_eq!(app.world().resource::<Target>().position, now);
}
