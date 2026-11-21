//! Tests of a spear that falls under the drawn ground where no collider is: it meets the ground there
//! instead of falling through it and flying on forever, out of reach.

use bevy::math::{DQuat, DVec3};
use bevy::prelude::*;

use super::javelin::{Ground, Planet, cross};
use crate::space::{Anchor, Frame};
use crate::universe::BodyId;
use crate::world::Terrain;

/// A step that ends a metre under the drawn ground meets it on the ground, with the local up as its
/// normal on flat land; a step a metre over it flies on.
#[test]
fn a_spear_under_the_ground_lands_on_it() {
    let terrain = Terrain::new(3);
    let anchor = Anchor {
        frame: Frame::Surface(BodyId::default()),
        origin: DVec3::ZERO,
        rotation: DQuat::IDENTITY,
        flat: true,
    };
    let ground = Planet {
        anchor: &anchor,
        terrain: &terrain,
        chunks: None,
    };
    let (x, z) = (40.0, -25.0);
    let height = terrain.mesh_height(x, z);
    let from = Vec3::new(x, height + 1.0, z);
    let point = cross(&ground, from, Vec3::new(x, height - 1.0, z)).expect("it lands");
    assert!((point - Vec3::new(x, height, z)).length() < 1e-2, "{point}");
    assert!(ground.clearance(point) >= 0.0);
    assert!(ground.normal(point).dot(Vec3::Y) > 0.5);
    assert!(cross(&ground, from, from + Vec3::Y * 0.5).is_none());
}
