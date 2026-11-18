//! Where villages stand: a coarse cell grid, suitable land, and the clearing kept around each village.

use std::f32::consts::TAU;

use bevy::prelude::*;
use rand::{RngExt, SeedableRng};
use rand_chacha::ChaCha8Rng;

use super::layout::build;
use super::trace::walkable;
use crate::world::Terrain;
use crate::world::structures::geo::{EDGE, edge_distance, metres};
use crate::world::structures::memo::{Grid, memo};
use crate::world::structures::site::{CELL, Kind, Site};

/// Side of a village cell in structure cells.
pub const SPAN: i32 = 3;

/// Distance kept between a village center and its cell border.
const MARGIN: f32 = 70.0;

/// Chance that a village cell holds a village.
const CHANCE: f64 = 0.35;

/// Candidate centers tried per cell.
const TRIES: u32 = 14;

/// Radius around a village center that small structures stay out of.
const CLEAR: f32 = 80.0;

/// Reach of a village from its center, including lots, fields and props.
pub const REACH: f32 = 64.0;

/// Returns the share of `[0, 1]` samples around `p` that satisfy `good`.
fn suitable(terrain: &Terrain, p: Vec2, radius: f32) -> bool {
    let h0 = terrain.height(p.x, p.y);
    if h0 < 1.0 || !walkable(terrain, p) {
        return false;
    }
    let mut ring = Vec::new();
    for (k, n) in [(0.4, 8), (0.75, 12), (1.0, 12)] {
        ring.extend((0..n).map(|i| p + Vec2::from_angle(i as f32 / n as f32 * TAU) * radius * k));
    }
    let dry = ring.iter().filter(|q| walkable(terrain, **q)).count();
    let level = ring
        .iter()
        .filter(|q| (terrain.height(q.x, q.y) - h0).abs() < 3.5)
        .count();
    dry * 100 >= ring.len() * 85 && level * 100 >= ring.len() * 75
}

/// Returns true when the village at `site` has at least six buildings of three different kinds.
fn viable(terrain: &Terrain, site: Site) -> bool {
    let lots = build(terrain, site).lots;
    let mut kinds: Vec<_> = lots.iter().map(|l| l.house as usize).collect();
    kinds.sort_unstable();
    kinds.dedup();
    lots.len() >= 6 && kinds.len() >= 3
}

/// Searches village cell `vcell` for a village, on a planet people live on. Each candidate center is judged
/// and laid out on the metric view around it, and must keep the whole village `EDGE` meters away from the
/// cube edges.
fn search(terrain: &Terrain, vcell: IVec2) -> Option<Site> {
    let mix = (vcell.x as u32 as u64) << 32 | vcell.y as u32 as u64;
    let mut rng = ChaCha8Rng::seed_from_u64(
        terrain.seed().wrapping_mul(0xA24B_AED4_963E_E407) ^ mix ^ 0x0711_A6E5,
    );
    if !terrain.globe().lands().settled() || !rng.random_bool(CHANCE) {
        return None;
    }
    let size = SPAN as f32 * CELL;
    let min = vcell.as_vec2() * size;
    (0..TRIES).find_map(|_| {
        let span = MARGIN..size - MARGIN;
        let p = min + Vec2::new(rng.random_range(span.clone()), rng.random_range(span));
        let radius = rng.random_range(35.0..55.0);
        let site = Site {
            kind: Kind::Village,
            origin: p,
            yaw: rng.random_range(0.0..TAU),
            floor: terrain.height(p.x, p.y),
            depth: 2.0,
            variant: rng.random_range(0..u32::MAX),
        };
        let view = terrain.around(p.x, p.y);
        let kept = terrain
            .globe()
            .reserved(terrain.embed(p.x, 0.0, p.y), f64::from(REACH));
        let clear = !kept && edge_distance(terrain, p) >= REACH + EDGE;
        (clear && suitable(&view, p, radius) && viable(&view, site)).then_some(site)
    })
}

/// Returns the village of village cell `vcell` of the chart of `terrain`, if it has one. Depends only on
/// the planet, the chart and the cell.
pub fn village_in_cell(terrain: &Terrain, vcell: IVec2) -> Option<Site> {
    memo(terrain, (Grid::Villages, vcell), || search(terrain, vcell))
}

/// Returns true when a village center lies within `CLEAR` meters of `p`.
pub fn near_village(terrain: &Terrain, p: Vec2) -> bool {
    let home = (p / (SPAN as f32 * CELL)).floor().as_ivec2();
    (-1..=1).any(|dx| {
        (-1..=1).any(|dy| {
            village_in_cell(terrain, home + IVec2::new(dx, dy))
                .is_some_and(|v| metres(terrain, v.origin, p) < CLEAR)
        })
    })
}
