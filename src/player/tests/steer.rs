//! The heading of the movement keys and the turn of the body.

use std::f32::consts::PI;

use bevy::prelude::*;

use crate::player::steer::*;

/// Frame time of the checks.
const DT: f32 = 1.0 / 60.0;

/// Holding the same keys keeps the walk straight while the camera drifts by itself, the
/// player's own turns still steer it, and a change of keys takes the view of that moment.
#[test]
fn keys_hold_their_heading_until_they_change() {
    let mut steer = Steer::default();
    let diagonal = Vec2::new(1.0, 1.0);
    assert_eq!(steer.step(0.4, 0.0, Vec2::ZERO), 0.4);
    assert_eq!(steer.step(0.5, 0.0, diagonal), 0.5);
    for frame in 1..200 {
        let drifted = 0.5 + frame as f32 * 0.01;
        assert_eq!(steer.step(drifted, 0.0, diagonal), 0.5);
    }
    assert!((steer.step(3.0, 0.2, diagonal) - 0.7).abs() < 1e-6);
    assert_eq!(steer.step(3.0, 0.0, Vec2::X), 3.0);
    assert_eq!(steer.step(3.1, 0.0, Vec2::ZERO), 3.1);
}

/// A body turning to its direction of travel gets there without passing it, never faster
/// than the limit, at any frame rate.
#[test]
fn the_body_turns_to_its_travel_without_overshoot() {
    for rate in [30.0, 60.0, 144.0] {
        let (mut turn, mut yaw, dt) = (Turn::default(), 0.0f32, 1.0 / rate);
        let goal = 2.6;
        for _ in 0..(1.2 * rate) as usize {
            let next = turn.step(yaw, Lead::Travel(goal), dt);
            let moved = next - yaw;
            assert!(
                moved > -1e-5 && next < goal + 1e-4,
                "{rate} Hz: {yaw} to {next}"
            );
            assert!(
                moved <= MAX_TURN * dt + 1e-5,
                "{rate} Hz: {} rad/s",
                moved / dt
            );
            yaw = next;
        }
        assert!((yaw - goal).abs() < 0.02, "{rate} Hz ended at {yaw}");
    }
}

/// A body on the view stays exactly on it however fast the view turns, a body off it
/// closes the gap smoothly, and holding leaves the heading alone.
#[test]
fn a_rigid_lead_is_followed_exactly() {
    let mut turn = Turn::default();
    let mut yaw = 1.0;
    for frame in 0..120 {
        let view = 1.0 + frame as f32 * 0.3;
        yaw = turn.step(yaw, Lead::View(view), DT);
        assert_eq!(yaw, view);
    }
    let mut last = wrap(yaw - 0.5).abs();
    for _ in 0..90 {
        yaw = turn.step(yaw, Lead::Nose(0.5), DT);
        let gap = wrap(yaw - 0.5).abs();
        assert!(gap <= last + 1e-6 && last - gap <= MAX_TURN * DT + 1e-5);
        last = gap;
    }
    assert!(last < 1e-3, "still {last} off the nose");
    assert_eq!(turn.step(0.3, Lead::Hold, DT), 0.3);
    assert_eq!(heading_of(Vec3::Y), None);
    assert!((heading_of(Vec3::NEG_X).unwrap() - PI / 2.0).abs() < 1e-6);
}
