//! The keys of the flight gear, as intents: the second jump press works the wings, the run key
//! the boots, and the ground keeps its jump and its run.

use bevy::prelude::*;

use super::stage::{app, body};
use crate::gear::tune::REF_GRAVITY;
use crate::gear::{Gear, Mode, descent_target};
use crate::player::Player;
use crate::player::asks::gather;
use crate::player::controller::{Motion, drive};
use crate::sim::{Intent, IntentOf, PlayerId};

/// Returns a world with one body standing on flat ground, moved by intents alone.
pub(super) fn world() -> App {
    let mut app = app(&[(PlayerId::LOCAL, 0.0)]);
    app.add_systems(Update, (gather, drive).chain());
    app
}

/// Holds `asked` for `frames` frames and returns the motion and the gear of the body after
/// them, with whether its boots were fired on any of them.
pub(super) fn hold(app: &mut App, frames: usize, asked: &[Intent]) -> (Motion, Gear, bool) {
    let mut fired = false;
    for _ in 0..frames {
        for &intent in asked {
            let player = PlayerId::LOCAL;
            app.world_mut().write_message(IntentOf { player, intent });
        }
        app.update();
        fired |= body(app, PlayerId::LOCAL).1.boots;
    }
    let (_, motion, gear) = body(app, PlayerId::LOCAL);
    (motion, gear, fired)
}

/// Puts the body 190 m higher, at rest, with nothing under it.
fn lift(app: &mut App) {
    let world = app.world_mut();
    let mut bodies = world.query_filtered::<(&mut Transform, &mut Motion), With<Player>>();
    for (mut transform, mut motion) in bodies.iter_mut(world) {
        transform.translation.y += 190.0;
        motion.velocity = Vec3::ZERO;
    }
}

/// Checks that the body of `app`, the run held for three more seconds, sinks at the descent
/// speed of the boots with the wings folded, and falls freely again once the run is let go.
fn brakes(app: &mut App) {
    let target = descent_target(REF_GRAVITY);
    let (motion, gear, _) = hold(app, 90, &[Intent::Boost]);
    let sink = -gear.state.velocity.y;
    assert!(motion.boots && !motion.wings, "{motion:?}");
    assert_eq!(gear.state.mode, Mode::BootFall);
    assert!((sink - target).abs() < 0.3, "sinks at {sink}");
    let (loose, gear, _) = hold(app, 30, &[]);
    assert!(!loose.boots && -gear.state.velocity.y > 2.0 * target);
}

/// A jump, then the jump pressed again in the air: the wings spread and the boots never fire,
/// however long it is held. Pressed a third time, the wings fold.
#[test]
fn a_second_jump_press_spreads_the_wings_and_a_third_folds_them() {
    let mut app = world();
    let (up, _, _) = hold(&mut app, 3, &[Intent::Jump]);
    assert!(up.velocity.y > 1.0 && !up.wings, "{up:?}");
    let (free, _, _) = hold(&mut app, 5, &[]);
    assert!(!free.wings && !free.grounded);
    let (out, gear, fired) = hold(&mut app, 3, &[Intent::Jump]);
    assert!(out.wings && !fired && !out.grounded, "{out:?}");
    assert_eq!(gear.state.mode, Mode::Glide);
    hold(&mut app, 1, &[]);
    let (folded, gear, fired) = hold(&mut app, 2, &[Intent::Jump]);
    assert!(!folded.wings && !fired, "{folded:?}");
    assert_eq!(gear.state.mode, Mode::Fall);
}

/// The run pressed in a fall with the wings folded brakes it to the descent speed of the
/// boots. A run already held when the body left the ground does the same once the fall is fast.
#[test]
fn the_run_key_brakes_a_fall_with_the_wings_folded() {
    let mut pressed = world();
    lift(&mut pressed);
    let (free, _, fired) = hold(&mut pressed, 30, &[]);
    assert!(!fired && free.velocity.y < -15.0, "{free:?}");
    brakes(&mut pressed);
    let mut kept = world();
    hold(&mut kept, 2, &[Intent::Boost]);
    lift(&mut kept);
    let (slow, _, fired) = hold(&mut kept, 8, &[Intent::Boost]);
    assert!(!fired && slow.velocity.y < -3.0, "{slow:?}");
    let (_, _, fired) = hold(&mut kept, 30, &[Intent::Boost]);
    assert!(fired, "a fast fall fires the boots of a run still held");
    brakes(&mut kept);
}

/// With the wings out the run key is the boost, and the wings key spreads and folds them as
/// the second jump press does.
#[test]
fn the_run_key_boosts_on_the_wings_and_the_wings_key_still_toggles() {
    let mut app = world();
    lift(&mut app);
    hold(&mut app, 5, &[]);
    let (out, _, _) = hold(&mut app, 1, &[Intent::Wings]);
    let (glide, gear, fired) = hold(&mut app, 30, &[]);
    assert!(out.wings && glide.wings && !fired);
    assert_eq!(gear.state.mode, Mode::Glide);
    let (fly, gear, _) = hold(&mut app, 30, &[Intent::Boost]);
    assert!(fly.boots && fly.wings);
    assert_eq!(gear.state.mode, Mode::Fly);
    hold(&mut app, 1, &[Intent::Wings, Intent::Boost]);
    let (folded, gear, _) = hold(&mut app, 2, &[Intent::Boost]);
    assert!(!folded.wings && folded.boots, "{folded:?}");
    assert_eq!(gear.state.mode, Mode::BootFall);
    let (again, _, _) = hold(&mut app, 1, &[Intent::Jump]);
    assert!(again.wings, "the jump pressed in a fall spreads the wings");
}

/// On the ground the jump jumps and the run runs: a run held through jump after jump goes
/// faster than a walk, leaves the ground, and never fires the boots or spreads the wings.
#[test]
fn on_the_ground_the_jump_and_the_run_are_unchanged() {
    let ahead = Intent::Move {
        forward: 1.0,
        strafe: 0.0,
    };
    let mut app = world();
    let (walk, _, _) = hold(&mut app, 45, &[ahead]);
    let (mut top, mut gear_on) = (0.0_f32, false);
    for _ in 0..4 {
        let (run, gear, fired) = hold(&mut app, 30, &[ahead, Intent::Boost]);
        let (hop, _, lit) = hold(&mut app, 4, &[ahead, Intent::Boost, Intent::Jump]);
        top = top.max(hop.velocity.y);
        gear_on |= fired || lit || run.wings || hop.wings || gear.state.mode != Mode::Off;
        assert!(run.velocity.length() > walk.velocity.length() + 2.0);
    }
    assert!(top > 1.0 && !gear_on, "rose at {top}, gear {gear_on}");
}
