//! Tests of structure placement and of every kind's mesh size.

use bevy::prelude::*;

use super::brush::Build;
use super::camp::{Layout, draw_layout};
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

/// Every seed has structures and none starts on the sea floor.
#[test]
fn structures_exist_on_dry_land() {
    for seed in 1..=3 {
        let t = Terrain::new(seed);
        let sites = square(&t);
        assert!(!sites.is_empty(), "seed {seed} has no structure");
        for s in sites {
            let surface = t.surface(s.origin.x, s.origin.y);
            assert!(
                !matches!(surface, Surface::Water | Surface::Shallows | Surface::Reef),
                "{s:?}"
            );
        }
    }
}

/// Builds the mesh of a synthetic site of `kind`, drawn by `draw`.
pub fn built<'a>(t: &'a Terrain, kind: Kind, variant: u32) -> Build<'a> {
    let site = Site {
        kind,
        origin: Vec2::ZERO,
        yaw: 0.7,
        floor: 5.0,
        depth: 2.5,
        variant,
    };
    Build::new(t, Vec2::ZERO, site)
}

/// Every small kind and every camp layout draws between 60 and 400 triangles.
#[test]
fn every_kind_has_a_sane_triangle_count() {
    let t = Terrain::new(1);
    for kind in Kind::SMALL.into_iter().filter(|k| *k != Kind::Camp) {
        let mut b = built(&t, kind, 3);
        DRAW[kind as usize](&mut b);
        let tris = b.mesh.into_mesh().count_vertices() / 3;
        assert!((60..=400).contains(&tris), "{kind:?} has {tris} triangles");
    }
    for layout in Layout::ALL {
        let mut b = built(&t, Kind::Camp, 3);
        draw_layout(&mut b, layout);
        let tris = b.mesh.into_mesh().count_vertices() / 3;
        assert!(
            (60..=400).contains(&tris),
            "{layout:?} has {tris} triangles"
        );
    }
}
