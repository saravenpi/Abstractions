//! Following a flying player on a round planet: no jump at any speed, through hard turns, anchor
//! rebases, uneven frames and low ground.

use bevy::prelude::*;

use super::tests_follow::Sky;
use crate::space::up;

/// Returns the length of frame `tick` at `fps` frames per second, uneven by the share `jitter`.
fn frame(fps: f32, jitter: f32, tick: usize) -> f32 {
    let wobble = ((tick * 7919) % 13) as f32 / 12.0 - 0.5;
    (1.0 + jitter * wobble) / fps
}

/// Returns the speed and the turn rate of the tour at `t` seconds: a second at rest, up to
/// 80 m/s in four seconds, a hard turn at speed, then a stop in a second and a half.
fn tour(t: f32) -> (f32, f32) {
    let speed = match t {
        t if t < 1.0 => 0.0,
        t if t < 18.0 => ((t - 1.0) * 20.0).min(80.0),
        t => (80.0 - (t - 18.0) * 60.0).max(0.0),
    };
    (speed, if (9.0..13.0).contains(&t) { 1.2 } else { 0.0 })
}

/// Flies the tour `height` metres up at `fps` with uneven frames. Returns the largest move of
/// the companion against the player in a frame, the farthest it was from its place, and the
/// number of anchor rebases crossed.
fn flown(fps: f32, jitter: f32, (stale, height): (bool, f32)) -> (f32, f32, usize) {
    let mut sky = Sky::new(height);
    sky.stale = stale;
    let mut dir = Vec3::new(0.3, 0.0, -1.0).normalize();
    let (mut jump, mut off, mut t, mut tick) = (0.0_f32, 0.0_f32, 0.0_f32, 0);
    while t < 22.0 {
        let dt = frame(fps, jitter, tick);
        let (speed, turn) = tour(t);
        dir = Quat::from_axis_angle(up(&sky.anchor, sky.eye), turn * dt) * dir;
        jump = jump.max(sky.step(sky.level(dir, speed), dt));
        let (goal, clearance) = sky.brain.goal();
        let place = sky.probe().clear(goal, clearance);
        off = off.max(sky.at.translation.distance(place) * f32::from(u8::from(t > 2.0)));
        (t, tick) = (t + dt, tick + 1);
    }
    (jump, off, sky.rebases)
}

/// From rest to 80 m/s, through a hard turn, across dozens of anchor rebases and to a stop, at
/// 30, 60 and 144 uneven frames per second: the companion never moves a metre against the
/// player in one frame, far less in fact, and stays near its place.
#[test]
fn never_jumps_from_rest_to_eighty_metres_a_second_and_back() {
    for (fps, bound) in [(30.0, 0.9), (60.0, 0.45), (144.0, 0.2)] {
        let (jump, off, rebases) = flown(fps, 0.4, (false, 60.0));
        assert!(rebases > 20, "only {rebases} rebases crossed");
        assert!(jump < bound, "at {fps} fps it jumps {jump} m in a frame");
        assert!(off < 7.0, "at {fps} fps it strays {off} m from its place");
    }
}

/// A companion that looks at the player before the controller moved it trails by one frame of
/// travel, so uneven frames at 80 m/s shake it more than one that sees the player after it
/// moved, which the system order now makes sure of.
#[test]
fn seeing_the_player_after_it_moved_shakes_it_less() {
    let (late, _, _) = flown(60.0, 0.4, (true, 60.0));
    let (fresh, _, _) = flown(60.0, 0.4, (false, 60.0));
    assert!(fresh < late, "fresh {fresh} m against stale {late} m");
}
