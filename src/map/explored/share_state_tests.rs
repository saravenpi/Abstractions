//! Tests of what the explored ground hands to the network and to the save.

use abstractions_net::leaves::Run;
use bevy::math::DVec3;

use super::Explored;
use super::grid::{depth_for, leaf_of};
use crate::saves::{Place, WorldSave};
use crate::universe::{EARTH_RADIUS, Universe};

/// Returns the runs of the one leaf at `dir` on an Earth.
fn runs_at(dir: DVec3) -> Vec<Run> {
    let mut explored = Explored::default();
    let body = Universe::new(3).home();
    explored.reveal(body, dir, 0.0, EARTH_RADIUS);
    explored.leaves(body).expect("revealed").runs()
}

/// Leaves uncovered here are handed to the network once; leaves from the server never are.
#[test]
fn only_local_leaves_are_sent_and_only_once() {
    let body = Universe::new(3).home();
    let mut explored = Explored::default();
    explored.merge_runs(body, &runs_at(DVec3::X));
    assert!(explored.take_new(body).is_empty());
    let before = explored.version(body);
    explored.reveal(body, DVec3::X, 0.0, EARTH_RADIUS);
    assert_eq!(explored.version(body), before);
    explored.reveal(body, DVec3::Y, 0.0, EARTH_RADIUS);
    let new = explored.take_new(body);
    assert_eq!(new.len(), 1);
    assert!(new.contains(leaf_of(DVec3::Y, depth_for(EARTH_RADIUS))));
    assert!(explored.take_new(body).is_empty());
    assert_eq!(explored.leaves(body).expect("kept").len(), 2);
}

/// Saved runs load as they were; a file without any is uncovered around its last place.
#[test]
fn saves_load_their_ground() {
    let universe = Universe::new(3);
    let body = universe.home();
    let mut save = WorldSave {
        seed: 3,
        place: Some(Place {
            body,
            dir: [0.0, 0.0, 1.0],
            altitude: 2.0,
        }),
        ..WorldSave::default()
    };
    let radius = universe.planet(body).radius;
    let old = Explored::of(&save);
    let under = leaf_of(DVec3::Z, depth_for(radius));
    assert!(old.leaves(body).expect("uncovered").contains(under));
    let runs = runs_at(DVec3::NEG_Y);
    save.explored = vec![crate::saves::SavedGround {
        body,
        runs: runs.clone(),
    }];
    let loaded = Explored::of(&save);
    assert_eq!(loaded.leaves(body).expect("loaded").runs(), runs);
}
