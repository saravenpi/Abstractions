//! Tests of the explored ground: the leaves, the reach, the network lists and the save.

use std::collections::HashSet;

use abstractions_net::leaves::{key, parts};
use bevy::math::DVec3;

use super::grid::{centre, depth_for, leaf_of};
use super::{Explored, REACH_CAP, REACH_FOOT, reach};
use crate::universe::{EARTH_RADIUS, Universe};

/// The depth of a body gives leaves of at most 500 m, and an Earth takes depth 15.
#[test]
fn a_depth_gives_leaves_of_at_most_500_metres() {
    assert_eq!(depth_for(EARTH_RADIUS), 15);
    for radius in [1.5e6, 3.0e6, 1.2e7, 7.0e7] {
        let depth = depth_for(radius);
        let leaf = std::f64::consts::FRAC_PI_2 * radius / (1u64 << depth) as f64;
        assert!(leaf <= 500.0, "{leaf} m at {radius}");
        assert!(leaf > 200.0, "{leaf} m at {radius}");
    }
}

/// Every leaf of a sample over the six faces holds its own centre.
#[test]
fn every_leaf_holds_its_centre() {
    let depth = 15;
    for face in 0..6 {
        for x in (0..1u64 << depth).step_by(997) {
            for y in (0..1u64 << depth).step_by(1009) {
                let morton = super::morton::spread(x) | (super::morton::spread(y) << 1);
                let leaf = key(face, depth, morton);
                assert_eq!(leaf_of(centre(leaf), depth), leaf, "face {face} {x} {y}");
            }
        }
    }
}

/// Returns the leaves a walk of 2 km along the equator reveals with `reach` metres, on an Earth,
/// and the directions it stood at.
fn walk(reach: f64) -> (HashSet<u64>, Vec<DVec3>) {
    let body = Universe::new(3).home();
    let mut explored = Explored::default();
    let stood: Vec<DVec3> = (0..=200)
        .map(|step| {
            let angle = f64::from(step) * 10.0 / EARTH_RADIUS;
            DVec3::new(angle.cos(), 0.0, -angle.sin())
        })
        .collect();
    for dir in &stood {
        explored.reveal(body, *dir, reach / EARTH_RADIUS, EARTH_RADIUS);
    }
    let leaves = explored.leaves(body).expect("revealed");
    (leaves.iter().collect(), stood)
}

/// Walking 2 km reveals the leaves under the path and those within the reach of it, and no
/// others.
#[test]
fn a_walk_reveals_the_leaves_under_the_path_and_no_others() {
    let (found, stood) = walk(REACH_FOOT);
    let depth = depth_for(EARTH_RADIUS);
    assert!(
        stood
            .iter()
            .all(|dir| found.contains(&leaf_of(*dir, depth)))
    );
    let far = (REACH_FOOT.max(0.75 * 305.0 * 2.0f64.sqrt()) + 1.0) / EARTH_RADIUS;
    for leaf in &found {
        let near = stood
            .iter()
            .any(|dir| centre(*leaf).angle_between(*dir) <= far);
        assert!(near, "leaf {leaf} is off the path");
    }
    assert!((8..=60).contains(&found.len()), "{} leaves", found.len());
}

/// A disc over the corner of a cube reveals leaves of three faces.
#[test]
fn a_disc_over_a_corner_takes_three_faces() {
    let body = Universe::new(3).home();
    let mut explored = Explored::default();
    let corner = DVec3::ONE.normalize();
    explored.reveal(body, corner, REACH_CAP / EARTH_RADIUS, EARTH_RADIUS);
    let faces: HashSet<u64> = explored
        .leaves(body)
        .expect("revealed")
        .iter()
        .map(|leaf| parts(leaf).0)
        .collect();
    assert_eq!(faces.len(), 3);
}

/// The reach is 300 m on the ground, the horizon higher up, and never past 2 km.
#[test]
fn the_reach_grows_with_height_to_a_cap() {
    assert_eq!(reach(-5.0, EARTH_RADIUS), REACH_FOOT);
    assert!((reach(100.0, EARTH_RADIUS) - 1100.0).abs() < 1.0);
    assert_eq!(reach(5000.0, EARTH_RADIUS), REACH_CAP);
}
