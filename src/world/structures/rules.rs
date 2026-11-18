//! Where each structure kind may stand: surfaces, biomes, flatness and the coast rule.

use std::f32::consts::TAU;

use bevy::prelude::*;

use super::site::Kind;
use super::village::REACH;
use crate::world::{Biome, Surface, Terrain};

/// How the floor height of a structure is chosen.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Floor {
    /// Ground height at the origin plus an offset.
    Center(f32),
    /// Highest ground under the footprint, plus a margin and an offset.
    Top(f32),
}

/// Site requirements of one structure kind.
#[derive(Debug, Clone, Copy)]
pub struct Spec {
    /// Footprint radius in meters.
    pub radius: f32,
    /// Allowed ground surfaces.
    pub surfaces: &'static [Surface],
    /// Allowed biomes, empty for any.
    pub biomes: &'static [Biome],
    /// Largest allowed height difference under the footprint.
    pub range: f32,
    /// Floor rule.
    pub floor: Floor,
    /// Lowest allowed ground height.
    pub min_height: f32,
    /// True when water must lie within 25 m.
    pub coast: bool,
}

const G: &[Surface] = &[Surface::Grass];
const GR: &[Surface] = &[Surface::Grass, Surface::Rock];
const GRS: &[Surface] = &[Surface::Grass, Surface::Rock, Surface::Sand];
const GS: &[Surface] = &[Surface::Grass, Surface::Sand];
const HIGH: &[Surface] = &[Surface::Rock, Surface::Snow];
const CAMP: &[Surface] = &[Surface::Grass, Surface::Sand, Surface::Snow];

const fn spec(
    radius: f32,
    surfaces: &'static [Surface],
    biomes: &'static [Biome],
    range: f32,
    floor: Floor,
) -> Spec {
    Spec {
        radius,
        surfaces,
        biomes,
        range,
        floor,
        min_height: 0.5,
        coast: false,
    }
}

const SPECS: [Spec; 10] = [
    spec(
        9.0,
        G,
        &[Biome::Meadow, Biome::Forest],
        1.2,
        Floor::Center(0.0),
    ),
    spec(6.0, GR, &[], 1.0, Floor::Top(0.0)),
    spec(6.0, G, &[Biome::Forest, Biome::Taiga], 0.8, Floor::Top(0.0)),
    Spec {
        coast: true,
        ..spec(5.0, GRS, &[], 1.2, Floor::Top(0.0))
    },
    spec(10.0, GS, &[Biome::Desert], 1.5, Floor::Top(0.0)),
    Spec {
        min_height: 12.0,
        ..spec(4.0, HIGH, &[], 2.0, Floor::Center(0.0))
    },
    Spec {
        min_height: 0.05,
        ..spec(2.0, &[Surface::Sand], &[], 1.5, Floor::Center(0.2))
    },
    spec(5.0, GS, &[Biome::Tropical], 1.0, Floor::Top(1.5)),
    spec(8.0, G, &[Biome::Cherry], 1.2, Floor::Top(0.0)),
    Spec {
        min_height: 0.25,
        ..spec(5.5, CAMP, &[], 1.0, Floor::Center(0.0))
    },
];

/// Length of the pier, which reaches farther from its origin than its footprint radius.
const PIER: f32 = 16.0;

impl Kind {
    /// Returns the site requirements of this kind. Villages have their own placement and no spec.
    pub fn spec(self) -> &'static Spec {
        &SPECS[self as usize]
    }

    /// Returns true for the ruins, which also stand on planets nobody lives on.
    pub fn ruin(self) -> bool {
        matches!(
            self,
            Kind::StoneCircle | Kind::Tower | Kind::Pyramid | Kind::Obelisk
        )
    }

    /// Returns how far, in meters, a site of this kind reaches from its origin.
    pub fn reach(self) -> f32 {
        match self {
            Kind::Village => REACH,
            Kind::Dock => PIER,
            _ => self.spec().radius,
        }
    }
}

/// Returns the center, an outer ring and an inner ring of a footprint.
fn samples(p: Vec2, r: f32) -> Vec<Vec2> {
    let ring = |k: f32, n: u32| {
        (0..n).map(move |i| p + Vec2::from_angle(i as f32 / n as f32 * TAU) * r * k)
    };
    std::iter::once(p)
        .chain(ring(0.9, 8))
        .chain(ring(0.45, 4))
        .collect()
}

/// Returns true when open water lies 14 to 24 m from `p` in some direction.
fn near_water(terrain: &Terrain, p: Vec2) -> bool {
    let sea = terrain.globe().sea_level();
    (0..8).any(|i| {
        let d = Vec2::from_angle(i as f32 / 8.0 * TAU);
        [14.0, 24.0]
            .iter()
            .any(|m| terrain.height(p.x + d.x * m, p.y + d.y * m) <= sea)
    })
}

/// Returns a yaw whose local +z points from `p` toward deep water, scanning from direction `start`.
pub fn water_yaw(terrain: &Terrain, p: Vec2, start: u32) -> Option<f32> {
    let angles = (0..16).map(|i| ((start + i) % 16) as f32 / 16.0 * TAU);
    let deep = terrain.globe().sea_level() - 0.6;
    angles.into_iter().find(|a| {
        let d = Vec2::new(a.sin(), a.cos());
        [9.0, 15.0]
            .iter()
            .all(|m| terrain.height(p.x + d.x * m, p.y + d.y * m) < deep)
    })
}

/// Returns the floor height and foundation depth if `kind` may stand at `p`.
pub fn fit(terrain: &Terrain, kind: Kind, p: Vec2) -> Option<(f32, f32)> {
    let s = kind.spec();
    if !s.biomes.is_empty() && !s.biomes.contains(&terrain.biome(p.x, p.y)) {
        return None;
    }
    let pts = samples(p, s.radius);
    let hs: Vec<f32> = pts.iter().map(|q| terrain.height(q.x, q.y)).collect();
    let (lo, hi) = hs
        .iter()
        .fold((f32::MAX, f32::MIN), |(l, h), v| (l.min(*v), h.max(*v)));
    if lo < s.min_height || hi - lo > s.range {
        return None;
    }
    if !pts
        .iter()
        .step_by(3)
        .all(|q| s.surfaces.contains(&terrain.surface(q.x, q.y)))
    {
        return None;
    }
    if s.coast && !near_water(terrain, p) {
        return None;
    }
    let floor = match s.floor {
        Floor::Center(e) => hs[0] + e,
        Floor::Top(e) => hi + 0.35 + e,
    };
    Some((floor, (floor - lo + 0.5).max(2.5)))
}
