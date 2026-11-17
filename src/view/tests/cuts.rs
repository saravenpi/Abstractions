//! Cuts and rebases: a cut resets the smoothing, a rebase changes nothing.

use bevy::prelude::*;

use super::world::{Boxes, Rebased};
use super::{apart, moment, run, seen, third};
use crate::view::{Heading, Kind, Moment, Rig, Target};

/// Returns a third person body that walks and turns, a moment into a switch to flight.
fn busy() -> (Rig, Moment) {
    let mut rig = Rig::default();
    let mut body = moment(Kind::OnFoot, third());
    body.target.velocity = Vec3::new(2.0, 0.0, -4.0);
    body.look.yaw_delta = 0.4 * body.dt;
    run(&mut rig, &mut body, &Boxes::floor(), 45);
    body.target.kind = Kind::Flying;
    run(&mut rig, &mut body, &Boxes::floor(), 8);
    (rig, body)
}

/// After a cut the camera is where a new rig would put it, and it stays there: no blend runs on.
#[test]
fn cut_resets_without_a_blend() {
    let (mut rig, mut body) = busy();
    let world = Boxes::floor();
    body.look.yaw_delta = 0.0;
    let (mut free, mut same) = (rig, body);
    let gliding = run(&mut free, &mut same, &world, 1);
    rig.cut();
    let cut = run(&mut rig, &mut body, &world, 1);
    let at_cut = seen(&cut, &body.target);
    let mut new = Rig::default();
    new.face(cut.heading);
    let direct = new.step(&body, &world);
    assert!(apart(at_cut, seen(&direct, &body.target)).0 < 1.0e-5);
    assert!(apart(at_cut, seen(&gliding, &body.target)).0 > 0.3);
    let later = run(&mut rig, &mut body, &world, 30);
    let (metres, radians) = apart(at_cut, seen(&later, &body.target));
    assert!(metres < 1.0e-4 && radians < 1.0e-4 && (cut.pose.fov - later.pose.fov).abs() < 1.0e-5);
}

/// A cut can set the look angles, and boarding the ship is a cut by itself.
#[test]
fn cut_takes_a_heading_and_boarding_cuts() {
    let (mut rig, mut body) = busy();
    rig.cut();
    rig.face(Heading {
        yaw: 1.0,
        pitch: 0.2,
    });
    body.look.yaw_delta = 0.0;
    let shown = run(&mut rig, &mut body, &Boxes::floor(), 1);
    assert_eq!(
        shown.heading,
        Heading {
            yaw: 1.0,
            pitch: 0.2
        }
    );
    body.target.kind = Kind::Ship;
    body.target.radius = 4.5;
    let seated = run(&mut rig, &mut body, &Boxes::default(), 1);
    assert_eq!(seated.heading, Heading::default());
    let again = Rig::default().step(&body, &Boxes::default());
    assert!(apart(seen(&seated, &body.target), seen(&again, &body.target)).0 < 1.0e-5);
}

/// Returns `target` after a rebase that turns render space by `turn` and moves it by `shift`.
fn rebased(target: &Target, turn: Quat, shift: Vec3) -> Target {
    Target {
        position: turn * target.position + shift,
        up: turn * target.up,
        forward: turn * target.forward,
        velocity: turn * target.velocity,
        spin: turn * target.spin,
        ground: turn * target.ground,
        ..*target
    }
}

/// A rebase in the middle of a blend, with the ground and a wall in view, leaves the camera
/// where it was relative to the target, on that frame and on the ones after.
#[test]
fn rebase_leaves_the_relative_view_unchanged() {
    let (mut kept, mut body) = busy();
    let world = Boxes::floor().with([-9.0, 0.0, 1.0], [-2.0, 4.0, 9.0]);
    let (turn, shift) = (
        Quat::from_axis_angle(Vec3::new(0.3, 0.9, 0.2).normalize(), 0.4),
        Vec3::new(7.0, -3.0, 5.0),
    );
    let moved_world = Rebased {
        inner: &world,
        turn,
        shift,
    };
    let mut moved = kept;
    for _ in 0..90 {
        let plain = run(&mut kept, &mut body, &world, 1);
        let mut twin = body;
        twin.target = rebased(&body.target, turn, shift);
        let other = moved.step(&twin, &moved_world);
        let (metres, radians) = apart(seen(&plain, &body.target), seen(&other, &twin.target));
        assert!(
            metres < 1.0e-5 && radians < 1.0e-5,
            "moved {metres} m, {radians} rad"
        );
        assert!((plain.pose.fov - other.pose.fov).abs() < 1.0e-6);
    }
}
