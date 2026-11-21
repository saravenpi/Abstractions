//! Tests of the fall: the gear lands in the cache, keeps its mass and comes back.

use super::{Left, leave};
use crate::craft::Pack;
use crate::craft::stage::pack;
use crate::universe::BodyId;
use bevy::math::DVec3;

/// What the robot carried is in the cache, none is left in the pack, and a second fall adds to the
/// same cache.
#[test]
fn the_gear_waits_where_it_fell() {
    let mut carried = pack();
    let mass = carried.mass();
    let mut left = Left::default();
    leave(
        &mut left,
        &mut carried,
        (BodyId::default(), DVec3::new(1.0, 2.0, 3.0)),
    );
    let cache = left.0.clone().expect("a cache");
    assert_eq!(carried, Pack::default());
    assert!((cache.pack.mass() - mass).abs() < 1e-4);
    assert_eq!(cache.at, [1.0, 2.0, 3.0]);
    let mut more = pack();
    leave(&mut left, &mut more, (BodyId::default(), DVec3::ZERO));
    let kept = left.0.expect("still a cache");
    assert!((kept.pack.mass() - 2.0 * mass).abs() < 1e-3);
    assert_eq!(kept.at, [1.0, 2.0, 3.0]);
}

/// A robot that fell with its kit wakes in a spare body still kitted: the kit waits in the cache,
/// and no second one is made.
#[test]
fn a_fall_gives_no_second_kit() {
    let universe = crate::universe::Universe::new(3);
    let planet = universe.planet(universe.home());
    let mut carried = Pack::default();
    assert_eq!(carried.equip(&planet.record, planet.gravity), 2);
    let mut left = Left::default();
    leave(&mut left, &mut carried, (BodyId::default(), DVec3::ZERO));
    assert!(carried.kitted && carried.items.is_empty());
    assert_eq!(carried.equip(&planet.record, planet.gravity), 0);
    assert_eq!(left.0.expect("a cache").pack.items.len(), 2);
}
