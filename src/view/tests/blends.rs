//! Blends: switching view and changing rig never makes the camera jump.

use bevy::prelude::*;

use super::world::Boxes;
use super::{apart, moment, run, seen, third};
use crate::view::{Kind, Moment, Rig, ViewPrefs};

/// Largest change of the view of a body in one frame at 60 Hz, in metres and in radians. A
/// switch moves the camera 3.7 m in 0.35 s, eased, which peaks at 0.27 m per frame.
const BOUND: (f32, f32) = (0.35, 0.05);

/// Steps `frames` frames and asserts that the view never changes by more than `bound` in
/// one. Returns the last view relative to the target, and its field of view.
fn glide(
    rig: &mut Rig,
    body: &mut Moment,
    frames: usize,
    bound: (f32, f32),
) -> ((Vec3, Quat), f32) {
    let world = Boxes::default();
    let mut view = |body: &mut Moment| {
        let shown = run(rig, body, &world, 1);
        (seen(&shown, &body.target), shown.pose.fov)
    };
    let mut last = view(body);
    for _ in 0..frames {
        let now = view(body);
        let (metres, radians) = apart(last.0, now.0);
        assert!(metres <= bound.0, "the camera jumped by {metres} m");
        assert!(radians <= bound.1, "the camera turned by {radians}");
        assert!((now.1 - last.1).abs() < 0.03, "the view width jumped");
        last = now;
    }
    last
}

/// Switching between first and third person while walking and turning is continuous, even
/// when the switch is reversed half way.
#[test]
fn mode_switch_is_continuous() {
    let mut rig = Rig::default();
    let mut body = moment(Kind::OnFoot, third());
    body.target.velocity = Vec3::new(1.0, 0.0, -4.0);
    body.look.yaw_delta = 0.5 * body.dt;
    for frames in [60, 40, 9, 40, 40] {
        glide(&mut rig, &mut body, frames, BOUND);
        body.prefs.toggle(Kind::OnFoot);
    }
}

/// The switch slides the camera along its own axis, from behind the head to the eyes on its face, while
/// the body fades out; it takes about 0.35 s.
#[test]
fn mode_switch_slides_along_the_view_axis() {
    let (mut rig, world) = (Rig::default(), Boxes::default());
    let mut body = moment(Kind::OnFoot, third());
    let mut last = run(&mut rig, &mut body, &world, 30);
    body.prefs = ViewPrefs::default();
    for frame in 0..24 {
        let now = run(&mut rig, &mut body, &world, 1);
        let (at, was) = (now.pose.position, last.pose.position);
        assert!(at.z <= was.z && at.z >= -0.16001 && at.x >= -1.0e-5 && at.x <= was.x + 1.0e-6);
        assert!(now.fade.0 <= last.fade.0);
        assert!(
            frame < 22 || (now.fade.0 == 0.0 && at.distance(Vec3::new(0.0, 1.6, -0.16)) < 1.0e-4)
        );
        assert!(frame > 15 || at.z > 1.0e-3, "the switch ended early");
        last = now;
    }
}

/// Taking off, flying and dropping back to the ground rig in mid-air are continuous in both
/// views, and the flight rig sees wider and, in third person, sits farther back.
#[test]
fn ground_flight_ground_is_continuous() {
    for (prefs, back) in [(third(), 1.5), (ViewPrefs::default(), -0.05)] {
        let mut rig = Rig::default();
        let mut body = moment(Kind::OnFoot, prefs);
        body.target.velocity = Vec3::new(0.0, -2.0, -14.0);
        body.target.bank = 0.3;
        let ground = glide(&mut rig, &mut body, 90, BOUND);
        body.target.kind = Kind::Flying;
        let flight = glide(&mut rig, &mut body, 120, BOUND);
        assert!(flight.1 > ground.1 + 0.05);
        assert!(flight.0.0.length() >= ground.0.0.length() + back);
        body.target.kind = Kind::OnFoot;
        let landed = glide(&mut rig, &mut body, 120, BOUND);
        assert!(apart(ground.0, landed.0).0 < 0.08);
    }
}

/// Flying banks the horizon by a share of the bank of the body, never all of it, and the third
/// person camera looks a little toward where the body is going.
#[test]
fn flight_banks_partly_and_looks_ahead() {
    for (prefs, share) in [(third(), 0.3), (ViewPrefs::default(), 0.3)] {
        let mut body = moment(Kind::Flying, prefs);
        body.target.velocity = Vec3::new(-8.0, 0.0, -20.0);
        body.target.bank = 0.4;
        let shown = run(&mut Rig::default(), &mut body, &Boxes::default(), 240);
        let right = shown.pose.rotation * Vec3::X;
        assert!(
            (right.y + (0.4_f32 * share).sin()).abs() < 0.01,
            "roll is {}",
            right.y
        );
        let ahead = shown.pose.rotation * Vec3::NEG_Z;
        assert_eq!(
            ahead.x < -0.03,
            body.prefs.on_foot == crate::view::ViewMode::Third
        );
    }
}

/// The ship switches between the cockpit and the chase camera without a jump, at speed. The
/// switch covers 16.6 m in 0.35 s, eased, which peaks at 1.2 m per frame.
#[test]
fn ship_switch_is_continuous() {
    let mut rig = Rig::default();
    let mut ship = moment(Kind::Ship, third());
    ship.target.radius = 4.5;
    ship.target.ground = Vec3::Y;
    ship.target.velocity = Vec3::new(0.0, 5.0, -90.0);
    for frames in [90, 40, 9, 60] {
        glide(&mut rig, &mut ship, frames, (1.3, 0.05));
        ship.prefs.toggle(Kind::Ship);
    }
}
