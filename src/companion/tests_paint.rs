//! Tests of the painted faces and of where the eyes turn.

use bevy::prelude::*;

use super::attention::eye_shift;
use super::face::paint::{HEIGHT, WIDTH, lit, paint, rgba, symmetric};
use super::face::{Expression, Frame, Lid};

/// Returns the fewest and the most lit pixels an open-eyed frame of an expression may have.
fn lit_range(expr: Expression) -> (u32, u32) {
    match expr {
        Expression::Neutral => (40, 48),
        Expression::Happy => (24, 36),
        Expression::Curious => (56, 66),
        Expression::Surprised => (60, 76),
        Expression::Sleepy => (16, 30),
        Expression::Worried => (24, 34),
        Expression::Love => (26, 64),
    }
}

/// Returns the frame of `expr` with open eyes looking straight ahead.
fn open(expr: Expression, beat: bool) -> Frame {
    Frame {
        expr,
        beat,
        ..default()
    }
}

/// Every expression lights the expected number of pixels on both of its frames, and its two
/// frames differ unless it is a calm one.
#[test]
fn frames_light_the_expected_pixels() {
    for expr in Expression::ALL {
        let (low, high) = lit_range(expr);
        for beat in [false, true] {
            let count = lit(&paint(&open(expr, beat)));
            assert!((low..=high).contains(&count), "{expr:?} {beat} lit {count}");
        }
        let calm = matches!(expr, Expression::Neutral | Expression::Curious);
        assert_eq!(paint(&open(expr, false)) == paint(&open(expr, true)), calm);
    }
    let shut = Frame {
        lid: Lid::Closed,
        ..default()
    };
    let half = Frame {
        lid: Lid::Half,
        ..default()
    };
    assert!(lit(&paint(&shut)) < lit(&paint(&half)));
    assert!(lit(&paint(&half)) < lit(&paint(&default())));
}

/// Neutral, happy, surprised, worried and love faces are mirror images of themselves. The curious one is not.
#[test]
fn faces_are_symmetric_where_they_should() {
    let even = [
        Expression::Neutral,
        Expression::Happy,
        Expression::Surprised,
        Expression::Worried,
        Expression::Love,
    ];
    for (expr, beat) in even.into_iter().flat_map(|e| [(e, false), (e, true)]) {
        assert!(symmetric(&paint(&open(expr, beat))), "{expr:?} {beat}");
    }
    assert!(!symmetric(&paint(&open(Expression::Curious, false))));
}

/// Returns the mean column and row of the lit pixels of a picture.
fn centre(frame: &Frame) -> Vec2 {
    let canvas = paint(frame);
    let lit_at = (0..HEIGHT)
        .flat_map(|y| (0..WIDTH).map(move |x| (x, y)))
        .filter(|(x, y)| (canvas[*y] >> (WIDTH - 1 - x)) & 1 == 1);
    let sum: Vec2 = lit_at
        .clone()
        .map(|(x, y)| Vec2::new(x as f32, y as f32))
        .sum();
    sum / lit_at.count() as f32
}

/// Looking left, right, up and down moves the eyes that way without losing a pixel.
#[test]
fn gaze_moves_the_eyes() {
    let rest = centre(&default());
    for (gaze, axis) in [
        (IVec2::X, Vec2::X),
        (IVec2::NEG_X, Vec2::NEG_X),
        (IVec2::Y, Vec2::NEG_Y),
        (IVec2::NEG_Y, Vec2::Y),
    ] {
        let frame = Frame { gaze, ..default() };
        assert_eq!(lit(&paint(&frame)), lit(&paint(&default())));
        assert!((centre(&frame) - rest).dot(axis) > 0.8, "{gaze}");
    }
    assert_eq!(eye_shift(Vec3::NEG_Z), IVec2::ZERO);
    assert_eq!(eye_shift(Vec3::new(1.0, 0.0, -1.0)), IVec2::NEG_X);
    assert_eq!(eye_shift(Vec3::new(-1.0, 0.0, -1.0)), IVec2::X);
    assert_eq!(eye_shift(Vec3::new(0.0, 1.0, -1.0)), IVec2::Y);
    assert_eq!(eye_shift(Vec3::new(0.0, -1.0, -1.0)), IVec2::NEG_Y);
}

/// The picture is thirty-two by sixteen opaque pixels, bright where lit and nearly black elsewhere.
#[test]
fn picture_bytes_match_the_canvas() {
    let canvas = paint(&default());
    let bytes = rgba(&canvas);
    assert_eq!(bytes.len(), WIDTH * HEIGHT * 4);
    let bright = bytes.chunks(4).filter(|pixel| pixel[0] == 255).count();
    assert_eq!(bright as u32, lit(&canvas));
    assert!(
        bytes
            .chunks(4)
            .all(|pixel| pixel[3] == 255 && (pixel[0] == 255 || pixel[0] < 16))
    );
}
