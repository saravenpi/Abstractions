//! Structure kinds, cell placement and the pure `structure_in_cell` rule.

use std::f32::consts::TAU;

use bevy::prelude::*;
use rand::{RngExt, SeedableRng};
use rand_chacha::ChaCha8Rng;

use super::rules::{fit, water_yaw};
use crate::world::{CHUNK_SIZE, Terrain};

/// Side of one structure cell in meters.
pub const CELL: f32 = 160.0;

/// Distance kept between a structure and its cell border.
const MARGIN: f32 = 24.0;

/// Candidate spots tried in one cell.
const TRIES: u32 = 12;

/// Chance that a cell tries to hold a structure at all.
const CHANCE: f64 = 0.5;

/// Every kind of structure, in table order.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Kind {
    /// Standing stones around an altar.
    StoneCircle,
    /// Broken round tower with an open doorway.
    Tower,
    /// Wooden cabin with a pitched roof.
    Cabin,
    /// Tall striped coastal tower with a lantern.
    Lighthouse,
    /// Stepped sandstone pyramid.
    Pyramid,
    /// Tall carved monolith.
    Obelisk,
    /// Wooden pier reaching over the water.
    Dock,
    /// Thatched hut on stilts.
    StiltHut,
    /// Torii gate and small shrine.
    Shrine,
    /// Three cabins around a well.
    Village,
    /// Tents around a fire ring.
    Camp,
}

impl Kind {
    /// Every kind, in table order.
    pub const ALL: [Kind; 11] = [
        Kind::StoneCircle,
        Kind::Tower,
        Kind::Cabin,
        Kind::Lighthouse,
        Kind::Pyramid,
        Kind::Obelisk,
        Kind::Dock,
        Kind::StiltHut,
        Kind::Shrine,
        Kind::Village,
        Kind::Camp,
    ];
}

/// One placed structure.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Site {
    /// What stands here.
    pub kind: Kind,
    /// World position of its origin.
    pub origin: Vec2,
    /// Rotation around the vertical axis in radians.
    pub yaw: f32,
    /// World height of its floor.
    pub floor: f32,
    /// How far the foundation reaches below the floor.
    pub depth: f32,
    /// Random bits for small variations.
    pub variant: u32,
}

/// Builds the random stream of one cell.
fn cell_rng(seed: u64, cell: IVec2) -> ChaCha8Rng {
    let mix = (cell.x as u32 as u64) << 32 | cell.y as u32 as u64;
    ChaCha8Rng::seed_from_u64(seed.wrapping_mul(0x9E37_79B9_7F4A_7C15) ^ mix ^ 0x057A_71C5)
}

/// Returns the site of `kind` at `p` when the ground allows it.
fn try_kind(terrain: &Terrain, kind: Kind, p: Vec2, rng: &mut ChaCha8Rng) -> Option<Site> {
    let (floor, depth) = fit(terrain, kind, p)?;
    let yaw = match kind {
        Kind::Dock => water_yaw(terrain, p, rng.random_range(0..16))?,
        _ => rng.random_range(0.0..TAU),
    };
    Some(Site {
        kind,
        origin: p,
        yaw,
        floor,
        depth,
        variant: rng.random_range(0..u32::MAX),
    })
}

/// Returns the structure of a cell, if it has one. Depends only on the seed and the cell.
pub fn structure_in_cell(terrain: &Terrain, cell: IVec2) -> Option<Site> {
    let mut rng = cell_rng(terrain.seed(), cell);
    if !rng.random_bool(CHANCE) {
        return None;
    }
    let min = cell.as_vec2() * CELL;
    let count = Kind::ALL.len();
    for _ in 0..TRIES {
        let span = MARGIN..CELL - MARGIN;
        let p = min + Vec2::new(rng.random_range(span.clone()), rng.random_range(span));
        let start = rng.random_range(0..count);
        if terrain.height(p.x, p.y) <= 0.0 {
            continue;
        }
        for i in 0..count {
            if let Some(site) = try_kind(terrain, Kind::ALL[(start + i) % count], p, &mut rng) {
                return Some(site);
            }
        }
    }
    None
}

/// Returns the structures whose origin lies inside chunk `coord`.
pub fn sites_in_chunk(terrain: &Terrain, coord: IVec2) -> Vec<Site> {
    let min = coord.as_vec2() * CHUNK_SIZE;
    let max = min + Vec2::splat(CHUNK_SIZE);
    let (c0, c1) = (
        (min / CELL).floor().as_ivec2(),
        (max / CELL).floor().as_ivec2(),
    );
    (c0.x..=c1.x)
        .flat_map(|x| (c0.y..=c1.y).map(move |y| IVec2::new(x, y)))
        .filter_map(|cell| structure_in_cell(terrain, cell))
        .filter(|s| s.origin.cmpge(min).all() && s.origin.cmplt(max).all())
        .collect()
}

/// Returns the origins of every structure standing in chunk `coord` or within `margin` of its edges.
pub fn origins_near(terrain: &Terrain, coord: IVec2, margin: f32) -> Vec<Vec2> {
    let min = coord.as_vec2() * CHUNK_SIZE - Vec2::splat(margin);
    let max = min + Vec2::splat(CHUNK_SIZE + 2.0 * margin);
    let (c0, c1) = (
        (min / CELL).floor().as_ivec2(),
        (max / CELL).floor().as_ivec2(),
    );
    (c0.x..=c1.x)
        .flat_map(|x| (c0.y..=c1.y).map(move |y| IVec2::new(x, y)))
        .filter_map(|cell| structure_in_cell(terrain, cell))
        .map(|site| site.origin)
        .filter(|o| o.cmpge(min).all() && o.cmplt(max).all())
        .collect()
}
