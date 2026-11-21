//! Horizon tests: the attitude read from the local up, and the ball of the flight display with
//! the marks of the course and of the body.

use bevy::prelude::*;

use super::attitude::{attitude, project};
use super::faces::{BALL, REACH, flight};
use super::pixels::{Ink, Pixels};

/// Returns the flight display with no figure, the attitude `level` and the marks `marks`.
fn ball(level: Option<IVec2>, marks: [Option<IVec2>; 2]) -> Pixels {
    flight([(None, None); 2], level, marks)
}

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

/// The ball puts the ground under the horizon: dark sky above, grey ground below, and in a
/// climb the horizon goes down the ball.
#[test]
fn the_ball_shows_the_horizon() {
    let centre = BALL.0;
    let level = ball(Some(IVec2::ZERO), [None; 2]);
    assert!(!level.is_lit(centre.x + 1, centre.y - 8));
    assert_eq!(level.ink(centre.x + 1, centre.y + 9), Ink::Dim);
    assert_eq!(level.ink(centre.x + 9, centre.y), Ink::Lit);
    let climbing = ball(Some(IVec2::new(20, 0)), [None; 2]);
    assert!(!climbing.is_lit(centre.x + 1, centre.y + 9));
    let banked = ball(Some(IVec2::new(0, 10)), [None; 2]);
    assert!(
        banked.is_lit(centre.x + 10, centre.y + 2) && !banked.is_lit(centre.x - 10, centre.y - 2)
    );
    assert_eq!(ball(None, [None; 2]).count(), 0);
}

/// The marks show the course and the body where they are against the nose, on a ball that has
/// no horizon away from a surface.
#[test]
fn the_marks_point_at_the_course() {
    let centre = BALL.0;
    assert_eq!(project(Vec3::NEG_Z, REACH), IVec2::ZERO);
    let up = project(Vec3::Y, REACH);
    assert_eq!(up, IVec2::new(0, 6));
    let ahead = ball(None, [Some(IVec2::ZERO), None]);
    assert!(ahead.is_lit(centre.x, centre.y - 2) && ahead.is_lit(centre.x + 2, centre.y));
    assert!(!ahead.is_lit(centre.x + 1, centre.y + 9));
    let high = ball(None, [Some(up), None]);
    assert!(high.is_lit(centre.x + 1, centre.y - 6) && !high.is_lit(centre.x + 1, centre.y + 6));
    let body = ball(None, [None, Some(up)]);
    assert!(body.is_lit(centre.x + 2, centre.y - 6) && !body.is_lit(centre.x, centre.y - 6));
}
