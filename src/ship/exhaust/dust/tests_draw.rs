//! Tests of how a puff of dust is drawn: where it was on the planet through a rebase, and as a
//! round disc turned to the camera, never a streak.

use bevy::math::DVec3;
use bevy::prelude::*;

use super::puff::pose;
use super::rig::{RADIUS, aloft, anchored};
use crate::effects::soft_disc;
use crate::space::{Shift, rebased};

/// A rebase of the render anchor leaves a puff where it was on the planet: it is drawn where the
/// rebase carries the ground under it, and it reads back to the same planet position.
#[test]
fn a_rebase_leaves_a_puff_where_it_was_on_the_planet() {
    let up = DVec3::new(0.2, 1.0, 0.1).normalize();
    let old = anchored(up);
    let far = old.to_render(old.origin + DVec3::X * 2000.0);
    let new = rebased(&old, RADIUS, far).expect("a rebase 2 km away");
    let mote = aloft(up + DVec3::X * 1e-3, 3.0, (Vec3::new(4.0, 1.0, 0.0), 0.3));
    let (before, after) = (
        pose(&old, Quat::IDENTITY, &mote),
        pose(&new, Quat::IDENTITY, &mote),
    );
    let carried = Shift::between(&old, &new).point(before.translation);
    assert!(
        carried.distance(after.translation) < 0.01,
        "{carried} {}",
        after.translation
    );
    assert!(new.to_frame(after.translation).distance(mote.at) < 0.01);
}

/// However fast and whichever way it flies, a puff is drawn the same size across every way and
/// turned to the camera, never stretched along its motion.
#[test]
fn no_puff_is_drawn_as_a_streak() {
    let turn = Quat::from_euler(EulerRot::YXZ, 0.7, -0.4, 0.1);
    let anchor = anchored(DVec3::Y);
    for flow in [
        Vec3::ZERO,
        Vec3::new(0.0, -30.0, 0.0),
        Vec3::new(25.0, 5.0, -12.0),
    ] {
        let mut mote = aloft(DVec3::Y, 4.0, (flow, 0.3));
        mote.age = 1.0;
        let drawn = pose(&anchor, turn, &mote);
        assert!(
            drawn.scale.x > 0.0 && drawn.scale == Vec3::splat(drawn.scale.x),
            "{drawn:?}"
        );
        assert_eq!(drawn.rotation, turn);
    }
}

/// The disc a puff is drawn on is round, as wide as it is tall, flat, and lit from above.
#[test]
fn a_puff_is_a_round_disc() {
    let disc = soft_disc();
    let points = disc
        .attribute(Mesh::ATTRIBUTE_POSITION)
        .and_then(|values| values.as_float3())
        .expect("positions");
    let wide = |axis: usize| points.iter().map(|p| p[axis].abs()).fold(0.0, f32::max);
    assert!(
        (wide(0) - wide(1)).abs() < 0.05 && wide(2) == 0.0,
        "{} {}",
        wide(0),
        wide(1)
    );
    let normals = disc
        .attribute(Mesh::ATTRIBUTE_NORMAL)
        .and_then(|values| values.as_float3())
        .expect("normals");
    assert!(normals.iter().all(|n| *n == [0.0, 1.0, 0.0]));
}
