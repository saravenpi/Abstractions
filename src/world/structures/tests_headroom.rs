//! Tests of the roof colliders a player meets: a roof under snow carries no more of them than a bare one, and
//! no roof edge hangs at head height over ground a player walks on.

use std::sync::Arc;

use avian3d::prelude::*;
use bevy::prelude::*;

use super::brush::{Build, Part, Shape};
use super::camp::{Layout, draw_layout};
use super::plates::THICK;
use super::site::{Kind, Site};
use super::tests::built;
use super::village::style::style_for;
use super::village::{self, ALL_HOUSES, House};
use super::{DRAW, Terrain, compound};
use crate::player::BODY_RADIUS;
use crate::world::Biome;
use crate::world::globe::Face;
use crate::world::globe::tests_deck::giant;

/// Lowest a roof collider may hang over ground a player walks on, in meters: the capsule is 1.8 m tall.
const HEADROOM: f32 = 1.95;

/// Height in meters under which a roof edge is an obstacle like a wall or a tent, not a roof to walk under.
const LOW: f32 = 1.0;

/// Every village biome.
const BIOMES: [Biome; 5] = [
    Biome::Meadow,
    Biome::Taiga,
    Biome::Desert,
    Biome::Tropical,
    Biome::Cherry,
];

/// Returns, for the roof plate `part`, points just under its low edge and the same points a body radius
/// further under the roof, in chunk space, or nothing for any other collider.
fn low_edge(part: &Part) -> Vec<(Vec3, Vec3)> {
    let Shape::Cuboid(size) = part.shape else {
        return Vec::new();
    };
    if size.y != THICK {
        return Vec::new();
    }
    let corner = |x: f32, z: f32| part.pos + part.rot * (size * Vec3::new(x, -0.5, z));
    let mut under = [(-0.5, -0.5), (0.5, -0.5), (0.5, 0.5), (-0.5, 0.5)].map(|(x, z)| corner(x, z));
    under.sort_by(|a, b| a.y.total_cmp(&b.y));
    let inward = (part.pos - (under[0] + under[1]) * 0.5).with_y(0.0);
    let inward = inward.normalize_or_zero();
    let at = |k: f32| under[0].lerp(under[1], k) + inward * 0.05 - Vec3::Y * 0.03;
    [0.1, 0.3, 0.5, 0.7, 0.9]
        .map(|k| (at(k), at(k) + inward * BODY_RADIUS))
        .to_vec()
}

/// Returns true when a body can stand with its middle under `at`, `clear` meters over its floor: nothing
/// solid of `shape` lies within its radius at knee and at chest height, and the floor carries all of it,
/// `fall` giving the height of a point over what lies under it. A crate under an eave is no floor.
fn standable(shape: &Collider, at: Vec3, clear: f32, fall: &dyn Fn(Vec3) -> f32) -> bool {
    let ring = (0..8).map(|k| Vec2::from_angle(k as f32 * std::f32::consts::FRAC_PI_4));
    let spots = ring.map(|s| s * BODY_RADIUS).chain([Vec2::ZERO]);
    let inside = |p: Vec3| shape.contains_point(Vec3::ZERO, Quat::IDENTITY, p);
    spots.map(|s| at + Vec3::new(s.x, 0.0, s.y)).all(|p| {
        let free = [0.5, 1.0]
            .iter()
            .all(|y| !inside(p.with_y(at.y - clear + y)));
        free && (fall(p) - clear).abs() < 0.25
    })
}

/// Returns the lowest height in meters of a roof edge of `b` over what a player stands on under it, the
/// ground or a collider, among the edges a body can stand under that are neither obstacles under [`LOW`]
/// nor high enough.
fn worst_headroom(b: &Build) -> Option<f32> {
    let shape = compound(&b.parts)?;
    let fall = |p: Vec3| {
        let ground = b.terrain.mesh_height(p.x + b.center.x, p.z + b.center.y);
        let hit = shape.cast_ray(Vec3::ZERO, Quat::IDENTITY, p, Vec3::NEG_Y, 6.0, true);
        hit.map_or(f32::MAX, |(toi, _)| toi).min(p.y - ground)
    };
    let points = b.parts.iter().flat_map(low_edge);
    let clear = points.filter_map(|(edge, under)| {
        let clear = fall(edge);
        standable(&shape, under, clear, &fall).then_some(clear)
    });
    clear
        .filter(|c| (LOW..HEADROOM).contains(c))
        .min_by(f32::total_cmp)
}

/// Returns the metric view around a point of a planet whose ground is level everywhere, the deck of a gas
/// giant at height 0: every roof is measured over the floor it was drawn for, not over a slope.
fn level() -> Terrain {
    let globe = Arc::new(giant());
    Terrain::on(globe.clone(), globe.face_chart(Face::PosY)).around(0.0, 0.0)
}

/// Returns a site of `kind` at the origin of a level view, its floor on the ground.
fn site(kind: Kind) -> Site {
    Site {
        kind,
        origin: Vec2::ZERO,
        yaw: 0.7,
        floor: 0.0,
        depth: 2.5,
        variant: 3,
    }
}

/// A hipped and a gabled taiga roof under snow carry as many colliders as the same roofs without snow.
#[test]
fn a_snowy_roof_has_one_set_of_colliders() {
    let t = Terrain::new(1);
    let s = House::Cottage.shell();
    for pick in [0.0, 0.9] {
        let count = |biome: Biome| {
            let mut b = built(&t, Kind::Village, 3);
            b.cap_roof(&style_for(biome, 3), s.tops(s.h), 1.6, pick);
            b.parts.len()
        };
        let (bare, snowy) = (count(Biome::Meadow), count(Biome::Taiga));
        assert_eq!(snowy, bare, "pick {pick}: {snowy} colliders under snow");
        assert_eq!(bare, 1);
    }
}

/// On level ground, no small structure, camp layout or house of any village style has a roof edge a body can
/// stand under between 1 m and 1.95 m over the ground or the floor under it.
#[test]
fn no_roof_edge_hangs_at_head_height() {
    let t = level();
    let fresh = |kind: Kind| Build::new(&t, Vec2::ZERO, site(kind));
    let check = |b: &Build, what: String| {
        let worst = worst_headroom(b);
        assert!(worst.is_none(), "{what}: a roof edge {worst:?} m up");
    };
    for kind in Kind::SMALL.into_iter().filter(|k| *k != Kind::Camp) {
        let mut b = fresh(kind);
        DRAW[kind as usize](&mut b);
        check(&b, format!("{kind:?}"));
    }
    for layout in Layout::ALL {
        let mut b = fresh(Kind::Camp);
        draw_layout(&mut b, layout);
        check(&b, format!("{layout:?}"));
    }
    for (biome, house) in BIOMES.into_iter().flat_map(|b| ALL_HOUSES.map(|h| (b, h))) {
        let mut b = fresh(Kind::Village);
        village::draw_house(&mut b, biome, house);
        check(&b, format!("{house:?} in {biome:?}"));
    }
}

/// Every village under snow on the flat worlds of seeds 1 to 8 stays within the collider budget of a
/// village, its snowy roofs carrying one set of colliders each.
#[test]
fn snowy_villages_fit_their_collider_budget() {
    let mut found = 0;
    for seed in 1..=8 {
        let t = Terrain::new(seed);
        let cells = (-8..8).flat_map(|x| (-8..8).map(move |y| IVec2::new(x, y)));
        let villages = cells.filter_map(|c| village::village_in_cell(&t, c));
        for site in villages.filter(|s| t.biome(s.origin.x, s.origin.y) == Biome::Taiga) {
            let mut b = Build::new(&t, site.origin, site);
            DRAW[Kind::Village as usize](&mut b);
            eprintln!("snowy village: {} colliders", b.parts.len());
            assert!(b.parts.len() <= 360, "{} colliders", b.parts.len());
            found += 1;
        }
    }
    assert!(found > 0, "no village under snow");
}
