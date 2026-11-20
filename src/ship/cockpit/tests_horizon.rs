//! Horizon tests: the attitude read from the local up, the ball and the scope.

use bevy::prelude::*;

use super::attitude::{attitude, project};
use super::paint::{BALL, REACH, horizon, scope};

/// The attitude is the pitch up and the roll to the right of the ship against the local up, and
/// nothing when the up has no direction.
#[test]
fn the_attitude_is_read_from_the_local_up() {
    assert_eq!(attitude(Vec3::Y), Some(Vec2::ZERO));
    let up = attitude(Vec3::new(0.0, 0.8_f32.cos(), -0.8_f32.sin())).expect("an attitude");
    assert!((up.x - 0.8).abs() < 1.0e-5 && up.y.abs() < 1.0e-5);
    let bank = attitude(Vec3::new(-0.5_f32.sin(), 0.5_f32.cos(), 0.0)).expect("an attitude");
    assert!((bank.y - 0.5).abs() < 1.0e-5 && bank.x.abs() < 1.0e-5);
    assert_eq!(attitude(Vec3::ZERO), None);
}

/// The ball puts the ground under the horizon: dark sky above, lit ground below, and in a
/// climb the horizon goes down the ball.
#[test]
fn the_ball_shows_the_horizon() {
    let centre = (BALL / 2) as i32;
    let level = horizon(0, 0);
    assert!(!level.is_lit(centre, centre - 4) && level.is_lit(centre, centre + 4));
    let climbing = horizon(20, 0);
    assert!(!climbing.is_lit(centre, centre + 4));
    let banked = horizon(0, 10);
    assert!(banked.is_lit(centre + 5, centre + 1) && !banked.is_lit(centre - 5, centre - 1));
}

/// The scope marks the course and the body where they are against the nose, and is dark when
/// it has neither.
#[test]
fn the_scope_points_at_the_course() {
    let centre = (BALL / 2) as i32;
    assert_eq!(scope(None, None).count(), 0);
    assert_eq!(project(Vec3::NEG_Z, REACH), IVec2::ZERO);
    let up = project(Vec3::Y, REACH);
    assert_eq!(up, IVec2::new(0, 3));
    let ahead = scope(Some(IVec2::ZERO), None);
    assert!(ahead.is_lit(centre, centre) && ahead.is_lit(centre + 1, centre));
    let high = scope(Some(up), None);
    assert!(high.is_lit(centre, centre - 3) && !high.is_lit(centre, centre + 3));
    assert!(scope(None, Some(up)).is_lit(centre + 1, centre - 3));
}
