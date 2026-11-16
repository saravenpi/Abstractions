//! Tests of structure placement and of every kind's mesh size.

use bevy::prelude::*;

use super::brush::Build;
use super::site::{CELL, Kind, Site, structure_in_cell};
use super::{DRAW, Terrain};
use crate::world::Surface;

/// Lists the structures of a 3 km square.
fn square(terrain: &Terrain) -> Vec<Site> {
    let n = (1500.0 / CELL) as i32 + 1;
    (-n..n)
        .flat_map(|x| (-n..n).map(move |y| IVec2::new(x, y)))
        .filter_map(|c| structure_in_cell(terrain, c))
        .collect()
}

/// The same seed gives the same structures.
#[test]
fn placement_is_deterministic() {
    for seed in 1..=3 {
        assert_eq!(square(&Terrain::new(seed)), square(&Terrain::new(seed)));
    }
}

/// Every seed has structures and none starts in the sea.
#[test]
fn structures_exist_on_dry_land() {
    for seed in 1..=3 {
        let t = Terrain::new(seed);
        let sites = square(&t);
        assert!(!sites.is_empty(), "seed {seed} has no structure");
        for s in sites {
            assert_ne!(t.surface(s.origin.x, s.origin.y), Surface::Water, "{s:?}");
        }
    }
}

/// Every kind draws between 60 and 400 triangles.
#[test]
fn every_kind_has_a_sane_triangle_count() {
    let t = Terrain::new(1);
    for kind in Kind::ALL {
        let site = Site {
            kind,
            origin: Vec2::ZERO,
            yaw: 0.7,
            floor: 5.0,
            depth: 2.5,
            variant: 3,
        };
        let mut b = Build::new(&t, Vec2::ZERO, site);
        DRAW[kind as usize](&mut b);
        let tris = b.mesh.into_mesh().count_vertices() / 3;
        assert!((60..=400).contains(&tris), "{kind:?} has {tris} triangles");
    }
}
