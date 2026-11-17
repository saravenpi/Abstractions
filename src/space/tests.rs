//! Space tests: universe positions carry cells, and the anchor maps frames to render space.

use std::sync::Arc;

use bevy::math::{DQuat, DVec3, IVec3, Vec3};

use super::*;
use crate::universe::{GALAXY_CELL, Universe};
use crate::world::globe::Face;
use crate::world::{Globe, Terrain};

/// Moving past a cell edge carries into the cell index and differences stay exact.
#[test]
fn unipos_carries_cells() {
    let a = UniPos::new(IVec3::ZERO, DVec3::new(GALAXY_CELL - 10.0, 5.0, 0.0));
    let b = a + DVec3::new(25.0, -8.0, 0.0);
    assert_eq!(b.cell, IVec3::new(1, -1, 0));
    assert_eq!(b - a, DVec3::new(25.0, -8.0, 0.0));
    assert_eq!(a.distance(b), (25.0f64 * 25.0 + 64.0).sqrt());
    assert_eq!((b - DVec3::new(25.0, -8.0, 0.0)).cell, a.cell);
}

/// The anchor of the legacy flat world is the identity: render coordinates are world coordinates.
#[test]
fn flat_anchor_is_the_identity() {
    let terrain = Terrain::new(4);
    let spawn = Vec3::new(120.5, 3.25, -640.0);
    let anchor = Anchor::surface(Universe::new(4).home(), &terrain, spawn);
    let p = Vec3::new(-77.5, 12.0, 901.25);
    assert_eq!(anchor.to_render(p.as_dvec3()), p);
    assert_eq!(anchor.to_frame(p), p.as_dvec3());
    assert_eq!(anchor.rotation_to_render(DQuat::IDENTITY), Quat::IDENTITY);
    assert_eq!(altitude(&anchor, terrain.globe(), p), 12.0);
    assert_eq!(up(&anchor, p), Vec3::Y);
}

/// On a globe the anchor renders the anchored spot at its chart coordinates with +Y up.
#[test]
fn globe_anchor_is_upright_at_its_spot() {
    let globe = Arc::new(Globe::new(4, 6000.0, 22.0));
    for face in Face::ALL {
        let terrain = Terrain::on(globe.clone(), globe.face_chart(face)).around(900.0, -350.0);
        let at = Vec3::new(900.0, 7.5, -350.0);
        let anchor = Anchor::surface(Universe::new(4).home(), &terrain, at);
        assert!(!anchor.flat);
        let near = at + Vec3::new(6.0, 2.0, -9.0);
        let seen = anchor.to_render(terrain.embed(near.x, near.y, near.z));
        assert!((seen - near).length() < 0.02, "{face:?} {seen} {near}");
        assert!((up(&anchor, at) - Vec3::Y).length() < 1e-5);
        assert!((altitude(&anchor, &globe, at) - 7.5).abs() < 1e-3);
        assert!((anchor.to_render(anchor.to_frame(near)) - near).length() < 1e-3);
        let turn = anchor.rotation_to_render(anchor.rotation.inverse());
        assert!(turn.angle_between(Quat::IDENTITY) < 1e-5);
    }
}
