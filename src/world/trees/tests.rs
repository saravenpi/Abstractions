//! Tests for tree identities, chop progression, save persistence, and stand removal.

use bevy::prelude::*;

use super::{Felled, TREE_CHOPS, TreeId, id_of, is_tree, spare_trees};
use crate::world::propmesh::Plant;
use crate::world::rocks::{Stand, Taken};
use crate::world::scatter::{Placement, Rig};

/// Helper constructing an upright tree placement at `(x, z)`.
fn placement(plant: Plant, x: f32, z: f32) -> Placement {
    Placement {
        plant,
        pos: Vec3::new(x, 0.0, z),
        yaw: 0.25,
        scale: 1.0,
        variant: 0,
        lean: Vec2::ZERO,
    }
}

/// Tree ids are deterministic per chunk and position, and text parsing round-trips cleanly.
#[test]
fn tree_ids_round_trip_and_are_deterministic() {
    let p = placement(Plant::Oak, 12.5, -8.25);
    let a = id_of(99, &p);
    let b = id_of(99, &p);
    let c = id_of(100, &p);
    assert_eq!(a, b);
    assert_ne!(a, c);
    assert_eq!(TreeId::parse(&a.text()), Some(a));
    assert!(is_tree(Plant::Oak) && !is_tree(Plant::Rock) && !is_tree(Plant::Saguaro));
}

/// Barehanded hits take three swings while hatchet-power swings fell a tree in two, and saves persist.
#[test]
fn felled_progresses_and_round_trips_text() {
    let mut felled = Felled::default();
    let id_a = TreeId(0xDEAD_BEEF_0001);
    let id_b = TreeId(0xCAFE_BABE_0002);
    assert!(!felled.chop(id_a, 1));
    assert!(!felled.chop(id_a, 1));
    assert!(felled.chop(id_a, 1));
    assert!(!felled.chop(id_b, 2));
    assert!(felled.chop(id_b, 2));
    assert_eq!(
        (felled.hits(id_a), felled.hits(id_b)),
        (TREE_CHOPS, TREE_CHOPS)
    );
    let saved = felled.to_text();
    let restored = Felled::from_text(&saved);
    assert_eq!(restored, felled);
}

/// Felling a tree removes it from `Stand` and marks the chunk dirty; `spare_trees` omits it on reload.
#[test]
fn stand_fells_tree_and_spare_removes_it() {
    let mut props = vec![
        placement(Plant::Oak, 4.0, 4.0),
        placement(Plant::Pine, 10.0, 10.0),
    ];
    let oak_id = id_of(7, &props[0]);
    let pine_id = id_of(7, &props[1]);
    let mut felled = Felled::default();
    let mut stand = Stand::with_trees(7, &props, &Rig::PLAIN, &Taken::default(), &felled);
    assert_eq!(stand.trees().len(), 2);
    assert_eq!(
        stand.near_tree(Vec3::new(4.0, 1.5, 4.0)).map(|t| t.id),
        Some(oak_id)
    );
    assert!(stand.fell(oak_id, 2));
    assert!(!stand.dirty());
    assert!(stand.fell(oak_id, 2));
    assert!(stand.dirty());
    assert_eq!(stand.trees().len(), 1);
    assert_eq!(stand.trees()[0].id, pine_id);
    felled.mark_gone(oak_id);
    spare_trees(&mut props, 7, &felled);
    assert_eq!(props.len(), 1);
    assert_eq!(id_of(7, &props[0]), pine_id);
}
