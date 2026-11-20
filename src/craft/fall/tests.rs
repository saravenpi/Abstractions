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
