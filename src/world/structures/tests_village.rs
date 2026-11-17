//! Tests of village and camp placement, village budgets, collider sanity and mesh winding.

use bevy::prelude::*;

use super::audit::{Piece, has_twins, positions, signed_volume};
use super::brush::{Build, Shape};
use super::camp::{Layout, draw_layout};
use super::site::{CELL, Kind, Site, structure_in_cell};
use super::tests::built;
use super::village::{self, ALL_HOUSES};
use super::{DRAW, Terrain};
use crate::world::{Biome, Surface};

/// Lists the structures of a 4 km square.
fn square(t: &Terrain) -> Vec<Site> {
    let n = (2000.0 / CELL) as i32 + 1;
    (-n..n)
        .flat_map(|x| (-n..n).map(move |y| IVec2::new(x, y)))
        .filter_map(|c| structure_in_cell(t, c))
        .collect()
}

/// Asserts that a village has at least five buildings of three kinds, all on dry ground.
fn check_village(t: &Terrain, site: &Site) {
    let lots = village::houses(t, *site);
    let mut kinds: Vec<usize> = lots.iter().map(|l| l.0 as usize).collect();
    kinds.sort_unstable();
    kinds.dedup();
    assert!(
        lots.len() >= 5 && kinds.len() >= 3,
        "{site:?} has {} buildings, {} kinds",
        lots.len(),
        kinds.len()
    );
    for (_, p) in lots {
        let s = t.surface(p.x, p.y);
        assert!(
            !matches!(s, Surface::Water | Surface::Shallows | Surface::Reef),
            "lot at {p}"
        );
    }
}

/// Seeds 1 to 3 hold villages and camps, and every village has enough varied buildings on dry land.
#[test]
fn villages_and_camps_exist() {
    for seed in 1..=3 {
        let t = Terrain::new(seed);
        let sites = square(&t);
        let villages: Vec<&Site> = sites.iter().filter(|s| s.kind == Kind::Village).collect();
        let camps = sites.iter().filter(|s| s.kind == Kind::Camp).count();
        let counts: Vec<usize> = villages
            .iter()
            .map(|v| village::houses(&t, **v).len())
            .collect();
        eprintln!(
            "seed {seed}: {} villages {counts:?} buildings, {camps} camps",
            villages.len()
        );
        assert!(!villages.is_empty(), "seed {seed} has no village");
        assert!(camps >= 8, "seed {seed} has {camps} camps");
        villages.into_iter().for_each(|v| check_village(&t, v));
    }
}

/// Every village stays within its triangle and collider budgets, with sane collider shapes.
#[test]
fn villages_fit_their_budgets() {
    for seed in 1..=3 {
        let t = Terrain::new(seed);
        for site in square(&t).into_iter().filter(|s| s.kind == Kind::Village) {
            let mut b = Build::new(&t, site.origin, site);
            DRAW[Kind::Village as usize](&mut b);
            let tris = b.mesh.clone().into_mesh().count_vertices() / 3;
            eprintln!(
                "seed {seed} village: {tris} triangles, {} colliders",
                b.parts.len()
            );
            assert!((1500..=17000).contains(&tris), "{tris} triangles");
            assert!(b.parts.len() <= 320, "{} colliders", b.parts.len());
            sane_parts(&b);
        }
    }
}

/// Asserts that every collider of `b` is finite and has no empty side.
fn sane_parts(b: &Build) {
    for p in &b.parts {
        assert!(p.pos.is_finite() && p.rot.is_finite(), "{p:?}");
        let ok = match p.shape {
            Shape::Cuboid(s) => s.is_finite() && s.min_element() > 1e-3,
            Shape::Cylinder(r, h) => r.is_finite() && h.is_finite() && r > 1e-3 && h > 1e-3,
        };
        assert!(ok, "{p:?}");
    }
}

/// Every closed box winds outward and every thin sheet has a twin on its back.
fn check_winding(b: &Build, what: &str) {
    assert!(!b.audit.is_empty(), "{what}: nothing audited");
    let verts = positions(&b.mesh.clone().into_mesh());
    for (piece, s, e) in &b.audit {
        let part = &verts[*s..*e];
        match piece {
            Piece::Closed => assert!(
                part.len() == 36 && signed_volume(part) > 0.0,
                "{what}: inside-out box"
            ),
            Piece::Sheet => assert!(has_twins(part), "{what}: one-sided sheet"),
        }
    }
}

/// Every kind, camp layout and house kind has outward boxes, double-sided sheets and finite colliders.
#[test]
fn meshes_wind_outward() {
    let t = Terrain::new(1);
    for kind in Kind::SMALL.into_iter().filter(|k| *k != Kind::Camp) {
        let mut b = built(&t, kind, 3);
        DRAW[kind as usize](&mut b);
        check_winding(&b, &format!("{kind:?}"));
        sane_parts(&b);
    }
    for layout in Layout::ALL {
        let mut b = built(&t, Kind::Camp, 3);
        draw_layout(&mut b, layout);
        check_winding(&b, &format!("{layout:?}"));
        sane_parts(&b);
    }
    for biome in [
        Biome::Meadow,
        Biome::Taiga,
        Biome::Desert,
        Biome::Tropical,
        Biome::Cherry,
    ] {
        for house in ALL_HOUSES {
            let mut b = built(&t, Kind::Village, 3);
            village::draw_house(&mut b, biome, house);
            check_winding(&b, &format!("{house:?} in {biome:?}"));
            sane_parts(&b);
        }
    }
}

/// Over at least 12 villages of 2 seeds: bench legs stand on their own ground, every road vertex lies just above
/// the terrain mesh and no road triangle is buried, no crop, hedge or post bottom hangs above it, and no door
/// path crosses the circle of another lot.
#[test]
fn villages_follow_the_ground() {
    let cells = || (-6..6).flat_map(|x| (-6..6).map(move |y| IVec2::new(x, y)));
    let mut seen = 0;
    for t in [1, 42].map(Terrain::new) {
        let sites = cells().filter_map(|c| village::village_in_cell(&t, c));
        for site in sites.take(8) {
            let p = village::probe(&t, site);
            let lift = |v: &Vec3| v.y - t.mesh_height(v.x, v.z);
            let stands = |v: &Vec3| (v.y - t.height(v.x, v.z)).abs() <= 0.2;
            assert!(p.benches.iter().all(stands), "bench of {site:?}");
            let hugs = |v: &Vec3| (0.0..=0.15).contains(&lift(v));
            assert!(p.roads.iter().all(hugs), "road of {site:?}");
            let hangs = |v: &Vec3| (0.01..0.5).contains(&lift(v));
            assert!(!p.crops.iter().any(hangs), "crop of {site:?}");
            let sunk = |tri: &[Vec3; 3]| lift(&((tri[0] + tri[1] + tri[2]) / 3.0)) < -0.15;
            assert!(
                !p.roads.as_chunks::<3>().0.iter().any(sunk),
                "buried road of {site:?}"
            );
            assert!(p.doors.iter().all(|m| *m >= 0.0), "door path of {site:?}");
            seen += 1;
        }
    }
    assert!(seen >= 12, "{seen} villages");
}
