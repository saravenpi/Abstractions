//! Tests of the trail ring buffer, the ribbon and the pure parts of the other effects.

use bevy::math::{DQuat, DVec3};
use bevy::prelude::*;

use super::ribbon::{Look, ROWS, Strip, VERTICES, fill, mesh};
use super::trail::{POINTS, SPACING, Trail};
use crate::space::{Anchor, Frame, Shift, rebased};
use crate::universe::Universe;

/// Returns a surface anchor at the top of a planet of `radius` metres.
fn anchor(radius: f64) -> Anchor {
    Anchor {
        frame: Frame::Surface(Universe::new(1).home()),
        origin: DVec3::Y * radius,
        rotation: DQuat::IDENTITY,
        flat: false,
    }
}

/// Returns a plain look from above.
fn look() -> Look {
    Look {
        eye: Vec3::Y * 50.0,
        width: 0.05,
        spread: 3.0,
        life: 1.5,
        tint: Vec3::ONE,
    }
}

/// Flies an emitter along a gentle curve at `speed` and `rate` frames per second for 2 s.
fn flown(speed: f64, rate: u32) -> Trail {
    let mut trail = Trail::default();
    for frame in 0..=rate * 2 {
        let t = f64::from(frame) / f64::from(rate);
        let at = DVec3::new(
            speed * t,
            100.0 + (t * 0.8).sin() * 6.0,
            (t * 0.5).cos() * 9.0,
        );
        trail.feed(at, 1.0, 1.0 / rate as f32);
    }
    trail
}

/// The ring never grows: the mesh keeps its vertex count however long and fast the flight.
#[test]
fn ribbon_keeps_a_fixed_vertex_count() {
    let built = mesh();
    assert_eq!(built.count_vertices(), VERTICES);
    assert_eq!(VERTICES, (POINTS + 1) * 3);
    let trail = flown(90.0, 60);
    assert_eq!(trail.len(), POINTS);
    let mut strip = Strip::default();
    fill(&trail, &look(), |point| point.as_vec3(), &mut strip);
    assert_eq!(strip.positions.len(), VERTICES);
    assert!(
        size_of::<Trail>() < 4096,
        "the trail must stay a small inline buffer"
    );
}

/// Points are laid by distance: the same spacing at 20, 60 and 240 frames per second.
#[test]
fn points_are_spaced_by_distance_not_by_frame() {
    for rate in [20, 60, 240] {
        let trail = flown(70.0, rate);
        assert_eq!(trail.len(), POINTS, "{rate} Hz");
        for index in 0..POINTS - 1 {
            let gap = trail.point(index).at.distance(trail.point(index + 1).at);
            assert!(
                (gap - SPACING).abs() < 0.03 * SPACING,
                "{rate} Hz: gap {gap}"
            );
        }
        assert!(trail.tip().at.distance(trail.point(0).at) <= SPACING + 1e-6);
    }
}

/// Returns a trail laid over 99 m through `anchor` and the render position its emitter ends at.
fn laid(anchor: &Anchor) -> (Trail, Vec3) {
    let mut trail = Trail::default();
    let mut body = Vec3::new(0.0, 30.0, 0.0);
    for _ in 0..90 {
        body += Vec3::new(1.1, 0.0, 0.2);
        trail.feed(anchor.to_frame(body), 1.0, 1.0 / 60.0);
    }
    (trail, body)
}

/// A rebase moves no point in the planet frame, and the trail carries on across it without a
/// gap: the emitter is fed through the new anchor and lands where it was, and the ribbon drawn
/// through the new anchor is the old ribbon carried by the rebase.
#[test]
fn trail_survives_a_rebase() {
    let old = anchor(6000.0);
    let (before, body) = laid(&old);
    let new = rebased(&old, 6000.0, body).expect("the body is past the rebase limit");
    let shift = Shift::between(&old, &new);
    let mut trail = before;
    trail.feed(new.to_frame(shift.point(body)), 1.0, 0.0);
    for index in 0..before.len() {
        assert_eq!(
            trail.point(index).at,
            before.point(index).at,
            "point {index} moved"
        );
    }
    assert!(
        trail.tip().at.distance(before.tip().at) < 1e-3,
        "the emitter jumped"
    );
    let (mut was, mut now) = (Strip::default(), Strip::default());
    let seen = Look {
        eye: shift.point(look().eye),
        ..look()
    };
    fill(&before, &look(), |point| old.to_render(point), &mut was);
    fill(&trail, &seen, |point| new.to_render(point), &mut now);
    for (a, b) in was.positions.iter().zip(&now.positions).skip(3) {
        assert!(shift.point(Vec3::from(*a)).distance(Vec3::from(*b)) < 0.02);
    }
}

/// The glow is soft everywhere: nothing on the edges, nothing at the emitter, nothing at the
/// tail, and no step between rows.
#[test]
fn ribbon_fades_along_and_across() {
    let trail = flown(70.0, 60);
    let mut strip = Strip::default();
    fill(&trail, &look(), |point| point.as_vec3(), &mut strip);
    let middle = |row: usize| strip.colors[row * 3 + 1][3];
    for row in 0..ROWS {
        assert_eq!(strip.colors[row * 3][3], 0.0);
        assert_eq!(strip.colors[row * 3 + 2][3], 0.0);
        assert!((0.0..=1.0).contains(&middle(row)));
    }
    assert_eq!(middle(0), 0.0);
    assert!(middle(ROWS - 1) < 0.01);
    assert!(middle(6) > 0.5);
    assert!((1..ROWS).all(|row| (middle(row) - middle(row - 1)).abs() < 0.4));
    let width = |row: usize| {
        Vec3::from(strip.positions[row * 3]).distance(Vec3::from(strip.positions[row * 3 + 2]))
    };
    assert!(width(40) > width(1), "the ribbon must widen as it ages");
}
