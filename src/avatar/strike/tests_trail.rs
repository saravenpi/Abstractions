//! Tests of the trail of a tool: nothing under the speed threshold, a row for each fast frame, and a
//! fade within the life of the trail.

use bevy::prelude::*;

use super::ribbon::{edge, fill};
use super::trail::{ROWS, Trail, glow};
use super::tune::{TRAIL_LIFE, TRAIL_PEAK, TRAIL_SPEED};

/// Seconds in a frame at sixty frames per second.
const FRAME: f32 = 1.0 / 60.0;

/// A tip slower than the threshold lays nothing, however long it moves so.
#[test]
fn the_trail_is_empty_under_the_speed_threshold() {
    let mut trail = Trail::default();
    for _ in 0..120 {
        trail.feed([Vec3::ZERO, Vec3::Y], 0.99 * TRAIL_SPEED, FRAME);
    }
    assert!(trail.is_empty() && glow(TRAIL_SPEED) == 0.0);
    assert_eq!(glow(2.0 * TRAIL_SPEED), 1.0);
}

/// A fast tip lays a row a frame, the newest first, and the rows fade out once the tip slows down.
#[test]
fn a_fast_tip_leaves_a_trail_that_fades() {
    let mut trail = Trail::default();
    for n in 0..5 {
        trail.feed([Vec3::ZERO, Vec3::X * n as f32], 3.0 * TRAIL_SPEED, FRAME);
    }
    assert_eq!(trail.len(), 5);
    assert_eq!(trail.row(0).map(|r| r.edge[1]), Some(Vec3::X * 4.0));
    for _ in 0..(TRAIL_LIFE / FRAME) as usize + 2 {
        trail.feed([Vec3::ZERO; 2], 0.0, FRAME);
    }
    assert!(trail.is_empty());
}

/// The ring never holds more rows than it has, at any frame rate.
#[test]
fn the_ring_is_bounded() {
    let mut trail = Trail::default();
    for _ in 0..1000 {
        trail.feed([Vec3::ZERO, Vec3::Y], 10.0 * TRAIL_SPEED, 1.0 / 1000.0);
    }
    assert!(trail.len() <= ROWS);
}

/// The ribbon sits on the head: the edge runs from the head's inner end to the tip, and a thrust
/// sweeps a line, since both ends of every row lie on the same line.
#[test]
fn the_edge_lies_on_the_blade() {
    let blade = [Vec3::ZERO, Vec3::new(0.0, 0.0, -1.0)];
    let [lo, tip] = edge(blade, 0.2);
    assert!((lo.z + 0.8).abs() < 1e-6 && tip == blade[1]);
    let mut trail = Trail::default();
    trail.feed([lo, tip], 10.0, FRAME);
    let (mut positions, mut colours) = ([[0.0; 3]; ROWS * 2], [[0.0; 4]; ROWS * 2]);
    fill(&trail, [TRAIL_PEAK, 0.25], &mut positions, &mut colours);
    assert_eq!(positions[1], tip.to_array());
    assert!(colours[1][3] > colours[0][3] && colours[3][3] == 0.0);
}
