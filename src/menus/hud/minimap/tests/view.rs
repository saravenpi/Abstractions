//! The transform from the ground to the map, the rim, the image placement and the range steps.

use std::f32::consts::FRAC_PI_2;

use bevy::math::{Mat2, Vec2};

use crate::menus::hud::minimap::range::{Mode, RANGES, eased, range, step};
use crate::menus::hud::minimap::view::{INSET, fit, heading, pin, rim, to_view};

/// Returns true when two map points are the same within a thousandth of the map.
fn close(a: Vec2, b: Vec2) -> bool {
    a.distance(b) < 1e-3
}

/// With north up, east is to the right. Facing east, what lies east is ahead and what lies
/// north is to the left.
#[test]
fn the_map_turns_with_the_heading() {
    let (east, north) = (Vec2::new(100.0, 0.0), Vec2::new(0.0, 100.0));
    assert!(close(to_view(east, 0.0, 250.0), Vec2::new(0.4, 0.0)));
    assert!(close(to_view(north, 0.0, 250.0), Vec2::new(0.0, 0.4)));
    assert!(close(to_view(east, FRAC_PI_2, 250.0), Vec2::new(0.0, 0.4)));
    assert!(close(
        to_view(north, FRAC_PI_2, 250.0),
        Vec2::new(-0.4, 0.0)
    ));
    assert!((heading(Vec2::X).unwrap() - FRAC_PI_2).abs() < 1e-6);
    assert_eq!(heading(Vec2::Y), Some(0.0));
    assert_eq!(heading(Vec2::ZERO), None);
    let ahead = Vec2::new(3.0, -4.0);
    let turned = to_view(ahead * 20.0, heading(ahead).unwrap(), 250.0);
    assert!(close(turned, Vec2::new(0.0, 0.4)), "{turned}");
}

/// A mark in range stays where it is, and one past the rim waits on the rim in its direction.
/// The north mark is always on the rim.
#[test]
fn a_far_mark_waits_on_the_rim() {
    let edge = 1.0 - INSET;
    assert_eq!(pin(Vec2::new(0.3, -0.2)), (Vec2::new(0.3, -0.2), false));
    let (at, pinned) = pin(Vec2::new(4.0, 2.0));
    assert!(pinned && close(at, Vec2::new(edge, edge * 0.5)), "{at}");
    let (at, pinned) = pin(Vec2::new(-0.5, -30.0));
    assert!(pinned && close(at, Vec2::new(-edge / 60.0, -edge)), "{at}");
    assert!(close(rim(Vec2::Y, 0.0), Vec2::new(0.0, edge)));
    assert!(close(rim(Vec2::Y * 5.0, FRAC_PI_2), Vec2::new(-edge, 0.0)));
    assert!(close(rim(Vec2::Y, -FRAC_PI_2), Vec2::new(edge, 0.0)));
}

/// Returns where the frame shows the ground point `point` of an image that reaches `half`
/// metres from its middle, once the image is sized, turned and moved as `fit` says: the same
/// steps the interface applies, a clockwise turn around the middle and then the move.
fn drawn(point: Vec2, half: f32, (at, turn, radius): (Vec2, f32, f32)) -> Vec2 {
    let fit = fit(half, at, turn, radius);
    let own = Vec2::new(point.x, -point.y) / (2.0 * half);
    let moved = Mat2::from_angle(fit.turn) * own + fit.shift / 100.0;
    let screen = moved * fit.side / 100.0 * 2.0;
    Vec2::new(screen.x, -screen.y)
}

/// The image is as many times wider than the frame as the patch is wider than the view, and
/// every point of it lands where the marks would put it, for any place and heading.
#[test]
fn the_image_follows_the_view() {
    assert_eq!(fit(500.0, Vec2::ZERO, 0.0, 250.0).side, 200.0);
    assert_eq!(fit(500.0, Vec2::ZERO, 0.0, 250.0).shift, Vec2::ZERO);
    let poses = [
        (Vec2::ZERO, 0.0, 250.0),
        (Vec2::new(40.0, -65.0), 0.0, 250.0),
        (Vec2::new(40.0, -65.0), 1.1, 250.0),
        (Vec2::new(-70.0, 12.0), -2.6, 400.0),
    ];
    let points = [Vec2::ZERO, Vec2::new(120.0, 30.0), Vec2::new(-300.0, 410.0)];
    for pose in poses {
        for point in points {
            let wanted = to_view(point - pose.0, pose.1, pose.2);
            let got = drawn(point, 500.0, pose);
            assert!(close(got, wanted), "{pose:?} {point}: {got} for {wanted}");
        }
    }
}

/// The range goes up a step past each speed limit and comes back only well under it, a small
/// planet caps it, and the key goes through the speed, each step and off.
#[test]
fn the_range_follows_the_speed_in_steps() {
    assert_eq!((step(0.0, 0), step(8.0, 0), step(30.0, 0)), (0, 0, 1));
    assert_eq!((step(20.0, 1), step(17.0, 1)), (1, 0));
    assert_eq!((step(130.0, 0), step(90.0, 2), step(80.0, 2)), (2, 2, 1));
    assert_eq!(range(Mode::Auto, 0, 6000.0), Some(RANGES[0]));
    assert_eq!(range(Mode::Auto, 1, 6000.0), Some(RANGES[1]));
    assert_eq!(range(Mode::Fixed(2), 0, 6000.0), Some(2100.0));
    assert_eq!(range(Mode::Fixed(2), 0, 40_000.0), Some(RANGES[2]));
    assert_eq!(range(Mode::Off, 1, 6000.0), None);
    let mut mode = Mode::Auto;
    let seen: Vec<Mode> = std::iter::repeat_with(|| {
        mode = mode.next();
        mode
    })
    .take(5)
    .collect();
    let steps = [Mode::Fixed(0), Mode::Fixed(1), Mode::Fixed(2)];
    assert_eq!(seen[..3], steps);
    assert_eq!(seen[3..], [Mode::Off, Mode::Auto]);
}

/// A map that showed nothing starts on its range. From one step to the next the range moves a
/// little each frame, never past the goal, and lands exactly on it within two seconds.
#[test]
fn the_range_eases_between_steps() {
    assert_eq!(eased(0.0, 250.0, 0.016), 250.0);
    let (mut radius, mut frames) = (250.0_f32, 0);
    while radius != 800.0 {
        let next = eased(radius, 800.0, 1.0 / 60.0);
        assert!(next > radius && next <= 800.0, "{radius} to {next}");
        assert!(next / radius < 1.09, "a jump from {radius} to {next}");
        (radius, frames) = (next, frames + 1);
        assert!(frames < 120, "still at {radius}");
    }
    assert!(frames > 20, "only {frames} frames");
    let (up, down) = (eased(250.0, 800.0, 0.02), eased(800.0, 250.0, 0.02));
    assert!((800.0 / down - up / 250.0).abs() < 1e-3, "{up} and {down}");
}

/// On planets of real size the ranges stay at metre scale, and only a small moon caps them at a
/// share of its radius.
#[test]
fn the_ranges_hold_at_real_planet_sizes() {
    for radius in [1.5e6, 6.371e6, 1.2e7, 7.0e7] {
        for (step, wanted) in RANGES.iter().enumerate() {
            assert_eq!(range(Mode::Fixed(step), 0, radius), Some(*wanted));
        }
    }
    assert_eq!(range(Mode::Fixed(2), 0, 5000.0), Some(1750.0));
}
