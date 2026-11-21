//! Steering: the player's input always wins over what the rig does by itself.

use bevy::prelude::*;

use super::world::Boxes;
use super::{first, moment, run, third};
use crate::view::tune::{FOLLOW_DELAY, PITCH_LIMIT};
use crate::view::{Kind, Moment, Rig, ViewPrefs};

/// Returns a third person body that walks ahead and to the left: its travel yaw is a quarter of a half turn.
fn drifting() -> Moment {
    let mut body = moment(Kind::OnFoot, third());
    body.target.velocity = Vec3::new(-3.0, 0.0, -3.0);
    body
}

/// Left alone while the body moves, the yaw waits for the delay, then drifts behind the
/// direction of travel without passing it. One input stops the drift at once, and it only
/// comes back after the delay.
#[test]
fn auto_follow_yields_to_input() {
    let (mut rig, world, mut body) = (Rig::default(), Boxes::default(), drifting());
    let waiting = (FOLLOW_DELAY / body.dt) as usize - 2;
    assert_eq!(run(&mut rig, &mut body, &world, waiting).heading.yaw, 0.0);
    let drifted = run(&mut rig, &mut body, &world, 120).heading.yaw;
    assert!(drifted > 0.2 && drifted < std::f32::consts::FRAC_PI_4);
    body.look.yaw_delta = -0.01;
    let taken = run(&mut rig, &mut body, &world, 1).heading.yaw;
    assert!(
        (taken - (drifted - 0.01)).abs() < 1.0e-6,
        "the drift went on under input"
    );
    body.look.yaw_delta = 0.0;
    assert_eq!(run(&mut rig, &mut body, &world, waiting).heading.yaw, taken);
    assert!(run(&mut rig, &mut body, &world, 120).heading.yaw > taken + 0.02);
}

/// The drift can be switched off, never happens in first person and never turns the camera
/// toward a body that comes at it.
#[test]
fn auto_follow_has_limits() {
    let world = Boxes::default();
    let mut off = drifting();
    off.prefs.auto_follow = false;
    assert_eq!(
        run(&mut Rig::default(), &mut off, &world, 300).heading.yaw,
        0.0
    );
    let mut eyes = drifting();
    eyes.prefs = first();
    assert_eq!(
        run(&mut Rig::default(), &mut eyes, &world, 300).heading.yaw,
        0.0
    );
    let mut coming = drifting();
    coming.target.velocity = Vec3::new(0.5, 0.0, 4.0);
    assert_eq!(
        run(&mut Rig::default(), &mut coming, &world, 300)
            .heading
            .yaw,
        0.0
    );
}

/// The look turns the heading by exactly what was asked, and the pitch stops at the limit.
#[test]
fn look_turns_the_heading() {
    for prefs in [third(), first()] {
        let (mut rig, mut body) = (Rig::default(), moment(Kind::OnFoot, prefs));
        body.look.yaw_delta = 0.02;
        body.look.pitch_delta = 0.05;
        let heading = run(&mut rig, &mut body, &Boxes::default(), 100).heading;
        assert!((heading.yaw - 2.0).abs() < 1.0e-4);
        assert_eq!(heading.pitch, PITCH_LIMIT);
    }
}

/// In first person the camera turns with the look in the same frame; in third person it
/// trails a little and catches up.
#[test]
fn third_person_trails_the_look() {
    let turned = |prefs: ViewPrefs, rest: usize| {
        let (mut rig, world) = (Rig::default(), Boxes::default());
        let mut body = moment(Kind::OnFoot, prefs);
        run(&mut rig, &mut body, &world, 5);
        body.look.yaw_delta = 0.2;
        let shown = run(&mut rig, &mut body, &world, 1);
        body.look.yaw_delta = 0.0;
        let shown = if rest > 0 {
            run(&mut rig, &mut body, &world, rest)
        } else {
            shown
        };
        (shown.pose.rotation * Vec3::NEG_Z).angle_between(Vec3::NEG_Z)
    };
    assert!((turned(first(), 0) - 0.2).abs() < 1.0e-5);
    let trailing = turned(third(), 0);
    assert!(trailing > 0.02 && trailing < 0.19);
    assert!((turned(third(), 30) - 0.2).abs() < 1.0e-4);
}
