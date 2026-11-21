//! Tests of the trail of a javelin: it is laid only while the javelin flies fast, it fades within its
//! life once the javelin landed, its rows are spaced in time whatever the frame rate, and each row
//! lies across the path, square to the line of sight.

use bevy::prelude::*;

use super::super::ribbon::fill;
use super::super::trail::{ROWS, Trail};
use super::streak::across;
use super::tune::{STREAK_EVERY, STREAK_HALF, STREAK_LIFE, STREAK_PEAK};

/// A javelin flying at throwing speed lays a trail; once it lies still nothing new is laid, the
/// newest rows are brighter than the oldest, and within the life of the trail it is gone.
#[test]
fn the_trail_lives_only_while_it_flies() {
    let mut trail = Trail::lasting(STREAK_LIFE, STREAK_EVERY);
    let frame = 1.0 / 60.0;
    for n in 0..20 {
        trail.feed([Vec3::X * n as f32, Vec3::Y], 24.0, frame);
    }
    assert!(trail.len() >= 15, "{}", trail.len());
    let (mut positions, mut colours) = ([[0.0; 3]; ROWS * 2], [[0.0; 4]; ROWS * 2]);
    fill(&trail, [STREAK_PEAK, 1.0], &mut positions, &mut colours);
    let oldest = (trail.len() - 1) * 2;
    assert!(colours[0][3] > colours[oldest][3] && colours[0][3] <= STREAK_PEAK);
    let laid = trail.len();
    trail.feed([Vec3::ZERO; 2], 0.0, frame);
    assert!(trail.len() <= laid);
    for _ in 0..(STREAK_LIFE / frame) as usize + 2 {
        trail.feed([Vec3::ZERO; 2], 0.0, frame);
    }
    assert!(trail.is_empty());
}

/// At a high frame rate the rows are kept a sixtieth of a second apart, so the ring holds the
/// whole life of the trail.
#[test]
fn the_rows_are_spaced_in_time() {
    let mut trail = Trail::lasting(STREAK_LIFE, STREAK_EVERY);
    for _ in 0..240 {
        trail.feed([Vec3::ZERO, Vec3::Y], 24.0, 1.0 / 240.0);
    }
    let span = trail.row(trail.len() - 1).map_or(0.0, |row| row.age);
    assert!(
        span > 0.8 * STREAK_LIFE.min(ROWS as f32 * STREAK_EVERY),
        "{span}"
    );
}

/// A row lies across the tail, square to the path and to the line of sight, as wide as the trail.
#[test]
fn a_row_faces_the_eye() {
    let (tail, way, eye) = (
        Vec3::new(3.0, 2.0, -10.0),
        Vec3::NEG_Z,
        Vec3::new(0.0, 1.6, 0.0),
    );
    let [a, b] = across(tail, way, eye);
    let side = b - a;
    assert!((side.length() - 2.0 * STREAK_HALF).abs() < 1e-5);
    assert!(side.dot(way).abs() < 1e-5 && side.dot(eye - tail).abs() < 1e-4);
    assert!(((a + b) / 2.0).distance(tail) < 1e-5);
}
