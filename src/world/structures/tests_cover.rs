//! Tests that every structure is solid where it is drawn: no drawn piece large enough to run into lies
//! farther from a collider, or higher above the ground, than a body is wide.

use bevy::prelude::*;

use super::audit::positions;
use super::brush::Build;
use super::camp::{Layout, draw_layout};
use super::cover::{NEAR, bare};
use super::site::{CELL, Kind, structure_in_cell};
use super::tags::Tag;
use super::tests::built;
use super::village::{ALL_HOUSES, House};
use super::{DRAW, Terrain, compound};
use crate::world::Biome;

/// The biomes whose village styles differ.
const BIOMES: [Biome; 5] = [
    Biome::Meadow,
    Biome::Taiga,
    Biome::Desert,
    Biome::Tropical,
    Biome::Cherry,
];

/// Returns the drawn points of `b` no collider covers, each with the first vertex of its triangle. Spared
/// are what the ground carries, within [`NEAR`] above it or painted on it, and what is drawn soft.
fn uncovered(b: &Build) -> Vec<(usize, Vec3)> {
    let verts = positions(&b.mesh.clone().into_mesh());
    let tris = verts.as_chunks::<3>().0;
    let Some(solid) = compound(&b.parts) else {
        return tris.iter().map(|t| t[0]).enumerate().collect();
    };
    let soft = |i: usize| {
        let mut runs = b
            .tags
            .iter()
            .filter(|t| matches!(t.0, Tag::Soft | Tag::Paved));
        runs.any(|t| (t.1..t.2).contains(&(i * 3)))
    };
    let grounded = |p: Vec3| p.y <= b.terrain.height(b.center.x + p.x, b.center.y + p.z) + NEAR;
    let found = bare(tris, &solid, |i, p| soft(i) || grounded(p));
    found.into_iter().map(|(i, p)| (i * 3, p)).collect()
}

/// Returns one line per drawn piece of `b` that has uncovered points: how many, and the box around them in
/// the frame of the site.
fn listed(b: &Build, found: &[(usize, Vec3)]) -> String {
    let piece = |at: usize| {
        let of = b.audit.iter().find(|a| (a.1..a.2).contains(&at));
        of.map_or(at, |a| a.1)
    };
    let back = Quat::from_rotation_y(-b.site.yaw);
    let flat = b.site.origin - b.center;
    let local = |p: Vec3| back * (p - Vec3::new(flat.x, b.site.floor, flat.y));
    let mut pieces: Vec<(usize, usize, Vec3, Vec3)> = Vec::new();
    for (at, p) in found.iter().map(|(at, p)| (piece(*at), local(*p))) {
        match pieces.iter_mut().find(|known| known.0 == at) {
            Some(known) => *known = (at, known.1 + 1, known.2.min(p), known.3.max(p)),
            None => pieces.push((at, 1, p, p)),
        }
    }
    let line = |(at, n, lo, hi): &(usize, usize, Vec3, Vec3)| {
        format!("\n  vertex {at}: {n} points in {lo:.2} .. {hi:.2}")
    };
    pieces.iter().map(line).collect()
}

/// What the builds checked so far leave bare: each distinct list of bare pieces, with the first build that
/// showed it and how many builds did.
#[derive(Default)]
struct Found(Vec<(String, String, usize)>);

impl Found {
    /// Checks the build `b`, named `what`.
    fn check(&mut self, b: &Build, what: &str) {
        let found = uncovered(b);
        if found.is_empty() {
            return;
        }
        let list = listed(b, &found);
        match self.0.iter_mut().find(|known| known.1 == list) {
            Some(known) => known.2 += 1,
            None => self.0.push((what.into(), list, 1)),
        }
    }

    /// Fails with every list of bare pieces when a build had any.
    fn assert_none(&self) {
        let line = |(what, list, count): &(String, String, usize)| {
            format!("\n{what}, and {} more like it:{list}", count - 1)
        };
        let all: String = self.0.iter().map(line).collect();
        assert!(self.0.is_empty(), "drawn pieces have no collider:{all}");
    }
}

/// Every structure kind, drawn alone in five variants, is solid where it is drawn.
#[test]
fn every_kind_is_solid_where_drawn() {
    let (t, mut found) = (Terrain::new(1), Found::default());
    for (kind, variant) in Kind::ALL
        .into_iter()
        .flat_map(|k| (0..5).map(move |v| (k, v)))
    {
        let mut b = built(&t, kind, variant);
        DRAW[kind as usize](&mut b);
        found.check(&b, &format!("{kind:?} variant {variant}"));
    }
    found.assert_none();
}

/// Every camp layout, in five variants, is solid where it is drawn.
#[test]
fn every_camp_is_solid_where_drawn() {
    let (t, mut found) = (Terrain::new(1), Found::default());
    for (layout, variant) in Layout::ALL
        .into_iter()
        .flat_map(|l| (0..5).map(move |v| (l, v)))
    {
        let mut b = built(&t, Kind::Camp, variant);
        draw_layout(&mut b, layout);
        found.check(&b, &format!("{layout:?} camp variant {variant}"));
    }
    found.assert_none();
}

/// Every house kind, in every village style and eight lot variants, is solid where it is drawn.
#[test]
fn every_house_is_solid_where_drawn() {
    let (t, mut found) = (Terrain::new(1), Found::default());
    let styles = BIOMES.into_iter().flat_map(|b| (0..8).map(move |v| (b, v)));
    for (house, (biome, variant)) in ALL_HOUSES
        .into_iter()
        .flat_map(|h| styles.clone().map(move |s| (h, s)))
    {
        let mut b = built(&t, Kind::Village, variant);
        House::draw_variant(house, &mut b, biome, variant);
        found.check(&b, &format!("{house:?} in {biome:?} variant {variant}"));
    }
    found.assert_none();
}

/// Every structure the first two worlds place within 1.5 km of their origin is solid where it is drawn, on
/// the ground it really stands on.
#[test]
fn every_placed_structure_is_solid_where_drawn() {
    let (n, mut found) = ((1500.0 / CELL) as i32 + 1, Found::default());
    for seed in 1..=2 {
        let t = Terrain::new(seed);
        let cells = (-n..n).flat_map(|x| (-n..n).map(move |y| IVec2::new(x, y)));
        for site in cells.filter_map(|c| structure_in_cell(&t, c)) {
            let mut b = Build::new(&t, site.origin, site);
            DRAW[site.kind as usize](&mut b);
            found.check(&b, &format!("seed {seed} {site:?}"));
        }
    }
    found.assert_none();
}
