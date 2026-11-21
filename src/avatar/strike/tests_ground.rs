//! Tests of a spear that falls under the drawn ground where no collider is: it lands on the ground
//! there instead of falling through it and flying on forever, out of reach.

use bevy::math::{DQuat, DVec3};
use bevy::prelude::*;

use super::throw::under;
use crate::space::{Anchor, Frame};
use crate::universe::BodyId;
use crate::world::Terrain;

/// A point a metre under the drawn ground lands on the ground straight above it, with the local up;
/// a point a metre over it flies on.
#[test]
fn a_spear_under_the_ground_lands_on_it() {
    let terrain = Terrain::new(3);
    let anchor = Anchor {
        frame: Frame::Surface(BodyId::default()),
        origin: DVec3::ZERO,
        rotation: DQuat::IDENTITY,
        flat: true,
    };
    let (x, z) = (40.0, -25.0);
    let ground = terrain.mesh_height(x, z);
    let (point, up) = under(&anchor, &terrain, Vec3::new(x, ground - 1.0, z)).expect("it lands");
    assert!((point - Vec3::new(x, ground, z)).length() < 1e-2, "{point}");
    assert!((up - Vec3::Y).length() < 1e-4, "{up}");
    assert!(under(&anchor, &terrain, Vec3::new(x, ground + 1.0, z)).is_none());
}
