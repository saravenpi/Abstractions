//! Tests of the animal looked at: one near the crosshair within range is, one beyond the range or
//! off the cone is not unless it stands close by.

use bevy::prelude::*;

use super::pick::{CONE, NEAR, RANGE, looked};

/// Returns the entity of index `n`.
fn animal(n: u32) -> Entity {
    Entity::from_raw_u32(n + 1).expect("a valid index")
}

/// The eyes of the tests and the way they look.
const SIGHT: (Vec3, Vec3) = (Vec3::new(0.0, 1.6, 0.0), Vec3::NEG_Z);

/// An animal within range near the crosshair is looked at; one beyond the range, or within range
/// but off the cone and not close by, is not; one close by is, wherever it stands ahead.
#[test]
fn the_looked_at_animal_is_near_the_crosshair() {
    let at = |x: f32, z: f32| Vec3::new(x, 1.4, z);
    let one = |spot: Vec3| looked([(animal(0), spot)].into_iter(), SIGHT);
    assert_eq!(one(at(0.3, -8.0)), Some(animal(0)));
    assert_eq!(one(at(0.0, -(RANGE + 2.0))), None);
    let wide = (CONE + 0.3).tan() * 9.0;
    assert_eq!(one(at(wide, -9.0)), None);
    assert_eq!(one(at(NEAR * 0.6, -NEAR * 0.5)), Some(animal(0)));
    let many = [(animal(1), at(3.0, -4.0)), (animal(2), at(0.2, -10.0))];
    assert_eq!(looked(many.into_iter(), SIGHT), Some(animal(2)));
}
