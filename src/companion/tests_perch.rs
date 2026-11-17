//! The perch: the place the companion chooses around the player, and when it leaves it.

use bevy::prelude::*;

use super::perch::Perch;
use super::spot::Stance;
use crate::space::Anchor;
use crate::universe::BodyId;
use crate::world::Terrain;

/// The companion holds its place while the player turns the head, and only moves once the
/// player has kept its back to it for a while: it then comes round to the front.
#[test]
fn the_perch_does_not_follow_the_head() {
    let terrain = Terrain::new(3);
    let anchor = Anchor::surface(BodyId::default(), &terrain, Vec3::ZERO);
    let mut perch = Perch::new(9);
    let looking = |yaw: f32| Stance {
        eye: Vec3::Y * 2.0,
        forward: Quat::from_rotation_y(yaw) * Vec3::NEG_Z,
        up: Vec3::Y,
    };
    perch.advance(&anchor, &looking(0.0), Vec3::ZERO, 0.02);
    let chosen = perch.side;
    assert!(chosen.dot(Vec3::NEG_Z) > 0.3 && chosen.y.abs() < 1e-5);
    for step in 0..100 {
        perch.advance(&anchor, &looking(step as f32 * 0.012), Vec3::ZERO, 0.02);
        assert!(perch.side.distance(chosen) < 1e-4, "it swung with the head");
    }
    let away = looking(std::f32::consts::PI);
    (0..250).for_each(|_| perch.advance(&anchor, &away, Vec3::ZERO, 0.02));
    assert!(perch.side.dot(away.forward) > 0.3, "it stayed behind");
}
