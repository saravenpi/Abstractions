//! Tests of the ground a new player has already seen: the 2 km around the home base are
//! uncovered as soon as the base is known, and no ground far from it is.

use bevy::ecs::system::RunSystemOnce;
use bevy::math::DVec3;
use bevy::prelude::*;

use super::home;
use crate::home::Base;
use crate::map::explored::grid::{depth_for, leaf_of};
use crate::map::explored::{Explored, REACH_CAP, REACH_FOOT};
use crate::universe::{BodyId, EARTH_RADIUS};

/// Returns a base on the ground of a planet the size of Earth, in unit direction `dir`.
fn base(dir: DVec3) -> Base {
    let east = dir.cross(DVec3::Y).normalize();
    let centre = dir * EARTH_RADIUS;
    Base {
        body: BodyId::default(),
        centre,
        up: dir,
        east,
        north: dir.cross(east),
        wake: centre,
        pad: centre,
        heading: east,
        door: centre,
    }
}

/// Returns the ground explored once the home base at `dir` is found in a new world.
fn started(dir: DVec3) -> Explored {
    let mut world = World::new();
    world.insert_resource(Explored::default());
    world.insert_resource(base(dir));
    world.run_system_once(home).expect("the system runs");
    world.resource::<Explored>().clone()
}

/// The ground under the base and everything the player saw of it on arrival, out to more than
/// 1.5 km on every side, is explored from the start, which is further than a walk uncovers.
#[test]
fn the_ground_around_the_base_is_explored_at_the_start() {
    let dir = DVec3::new(0.5, 0.4, -0.76).normalize();
    let explored = started(dir);
    let leaves = explored.leaves(BodyId::default()).expect("ground");
    let depth = depth_for(EARTH_RADIUS);
    let east = dir.cross(DVec3::Y).normalize();
    let north = dir.cross(east);
    assert!(explored.version(BodyId::default()) > 0);
    assert!(leaves.contains(leaf_of(dir, depth)));
    const { assert!(REACH_CAP > 1500.0 && REACH_CAP > REACH_FOOT) };
    for turn in 0..16 {
        let angle = f64::from(turn) * std::f64::consts::TAU / 16.0;
        let way = east * angle.cos() + north * angle.sin();
        for metres in [300.0, 900.0, 1500.0] {
            let at = (dir + way * (metres / EARTH_RADIUS)).normalize();
            assert!(
                leaves.contains(leaf_of(at, depth)),
                "{metres} m, turn {turn}"
            );
        }
    }
}

/// Ground the player has not been near stays unexplored: nothing 3 km from the base is uncovered.
#[test]
fn the_ground_far_from_the_base_is_not_explored() {
    let dir = DVec3::new(0.5, 0.4, -0.76).normalize();
    let explored = started(dir);
    let leaves = explored.leaves(BodyId::default()).expect("ground");
    let depth = depth_for(EARTH_RADIUS);
    let east = dir.cross(DVec3::Y).normalize();
    for metres in [3000.0, 10_000.0, 500_000.0] {
        let at = (dir + east * (metres / EARTH_RADIUS)).normalize();
        assert!(!leaves.contains(leaf_of(at, depth)), "{metres} m");
    }
}
