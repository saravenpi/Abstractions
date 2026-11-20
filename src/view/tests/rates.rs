//! Smoothing: the same view at every frame rate, and a follow that never swings past its goal.

use bevy::prelude::*;

use super::world::Boxes;
use super::{apart, moment, run, seen, third, first};
use crate::view::smooth::Spring;
use crate::view::{Kind, Moment, Rig};

/// Returns the view of a body after two seconds of the same turning and walking at `hertz`.
fn two_seconds(start: &Moment, hertz: f32) -> ((Vec3, Quat), f32) {
    let mut moment = *start;
    moment.dt = 1.0 / hertz;
    moment.look.yaw_delta = 0.6 * moment.dt;
    moment.look.pitch_delta = -0.15 * moment.dt;
    let mut rig = Rig::default();
    let shown = run(
        &mut rig,
        &mut moment,
        &Boxes::default(),
        (2.0 * hertz) as usize,
    );
    (seen(&shown, &moment.target), shown.pose.fov)
}

/// Asserts that 30, 60 and 144 frames per second end within 1 percent of the same view.
fn same_at_every_rate(start: &Moment) {
    let slow = two_seconds(start, 30.0);
    for hertz in [60.0, 144.0] {
        let fast = two_seconds(start, hertz);
        let (metres, radians) = apart(slow.0, fast.0);
        let scale = slow.0.0.length().max(1.0);
        assert!(metres < 0.01 * scale, "{hertz} Hz is {metres} m away");
        assert!(radians < 0.01, "{hertz} Hz is turned by {radians}");
        assert!((slow.1 - fast.1).abs() < 0.01 * slow.1);
    }
}

/// A walking body is seen the same at every frame rate, in third and in first person.
#[test]
fn walking_view_is_framerate_independent() {
    for prefs in [third(), first()] {
        let mut start = moment(Kind::OnFoot, prefs);
        start.target.velocity = Vec3::new(3.0, 0.0, -2.0);
        same_at_every_rate(&start);
    }
}

/// A banked flying body is seen the same at every frame rate.
#[test]
fn flying_view_is_framerate_independent() {
    for prefs in [third(), first()] {
        let mut start = moment(Kind::Flying, prefs);
        start.target.velocity = Vec3::new(4.0, -3.0, -22.0);
        start.target.bank = 0.4;
        same_at_every_rate(&start);
    }
}

/// The spring closes on its goal from one side at any frame time, and gets there.
#[test]
fn spring_never_overshoots() {
    for dt in [0.001, 1.0 / 144.0, 1.0 / 30.0, 0.1] {
        let mut spring = Spring {
            value: Vec3::X,
            ..default()
        };
        let mut last = 1.0;
        for _ in 0..(3.0 / dt) as usize {
            spring.step(Vec3::ZERO, 9.0, dt);
            assert!(spring.value.x >= 0.0 && spring.value.x <= last);
            last = spring.value.x;
        }
        assert!(last < 1.0e-6);
    }
}

/// The camera falls behind a body that starts to run and closes up when it stops, without
/// ever passing the place it settles at.
#[test]
fn follow_never_overshoots() {
    let (mut rig, world) = (Rig::default(), Boxes::default());
    let mut body = moment(Kind::OnFoot, third());
    let rest = seen(&run(&mut rig, &mut body, &world, 30), &body.target)
        .0
        .z;
    body.target.velocity = Vec3::NEG_Z * 5.0;
    let mut last = rest;
    for _ in 0..120 {
        let behind = seen(&run(&mut rig, &mut body, &world, 2), &body.target).0.z;
        assert!(behind >= last - 1.0e-5, "the lag shrank while running");
        last = behind;
    }
    let lagging = last;
    assert!(lagging > rest + 0.2 && lagging < rest + 0.5 + 1.0e-4);
    body.target.velocity = Vec3::ZERO;
    for _ in 0..120 {
        let behind = seen(&run(&mut rig, &mut body, &world, 2), &body.target).0.z;
        assert!(behind <= last + 1.0e-5 && behind >= rest - 1.0e-4);
        last = behind;
    }
    assert!((last - rest).abs() < 1.0e-3);
}
