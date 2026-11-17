//! The site search: a dry, gentle, inland spot near the terrain spawn, clear of the cube edges.

use std::f32::consts::TAU;
use std::f64::consts::SQRT_2;

use bevy::math::DVec3;
use bevy::prelude::*;

use super::tune::{
    DRY, EDGE_CLEAR, FOOTPRINT, MAX_RELIEF, PAD, PAD_HALF, PAD_RELIEF, RELIEF_COST, SEARCH,
    SHORE_CLEAR,
};
use crate::world::Terrain;

/// Distance in metres between the rings of samples taken across a disc of ground.
const RING_STEP: f32 = 8.0;

/// Circles checked for sea between the footprint and the end of its clearance, and the
/// distance in metres between two samples on a circle.
const SHORE: (u32, f32) = (4, 6.0);

/// How many times the relief limits a later pass of the search allows, when a stricter pass
/// found nothing.
const SLACK: [f32; 2] = [1.0, 2.0];

/// Returns `count` points evenly spaced on the circle of `radius` around `at`.
fn circle(at: Vec2, radius: f32, count: u32) -> impl Iterator<Item = Vec2> {
    (0..count).map(move |i| at + Vec2::from_angle(i as f32 / count as f32 * TAU) * radius)
}

/// Returns the sample points of the disc of `radius` around `at`: its centre and rings.
fn disc(at: Vec2, radius: f32) -> impl Iterator<Item = Vec2> {
    let rings = (radius / RING_STEP).ceil() as u32;
    let ring = move |k: u32| circle(at, radius * k as f32 / rings as f32, (8 * k).max(1));
    (0..=rings).flat_map(ring)
}

/// Returns the distance in metres from the point in direction `dir` of a globe of `radius` to
/// the nearest edge of the cube face that holds it.
pub fn edge_distance(dir: DVec3, radius: f64) -> f64 {
    let mut size = dir.normalize().abs().to_array();
    size.sort_by(f64::total_cmp);
    ((size[2] - size[1]) / SQRT_2).asin() * radius
}

/// Returns the difference in metres between the highest and the lowest ground of the disc of
/// `radius` around `at` in the metric `view`, or nothing when part of it is low or in a pond,
/// or when the difference passes `limit`.
pub fn relief(view: &Terrain, at: Vec2, radius: f32, limit: f32) -> Option<f32> {
    let mut span = (f32::MAX, f32::MIN);
    for p in disc(at, radius) {
        let height = view.height(p.x, p.y);
        span = (span.0.min(height), span.1.max(height));
        if height < DRY || span.1 - span.0 > limit || view.pond_at(p.x, p.y) {
            return None;
        }
    }
    Some(span.1 - span.0)
}

/// Returns true when no sea lies between the footprint around `at` and the end of its clearance.
pub fn inland(view: &Terrain, at: Vec2) -> bool {
    let reach = |k: u32| FOOTPRINT as f32 + SHORE_CLEAR * k as f32 / SHORE.0 as f32;
    (1..=SHORE.0)
        .flat_map(|k| circle(at, reach(k), (TAU * reach(k) / SHORE.1) as u32))
        .all(|p| view.height(p.x, p.y) > 0.0)
}

/// Returns what a site at `at` costs in metres, its distance plus its relief, or nothing when
/// the ground there cannot take the base with `slack` times the relief limits.
fn cost(view: &Terrain, at: Vec2, slack: f32) -> Option<f32> {
    let edge = edge_distance(view.up(at.x, at.y), view.globe().radius());
    if edge < FOOTPRINT + EDGE_CLEAR || view.height(at.x, at.y) < DRY {
        return None;
    }
    let pad = relief(view, at + PAD.at, PAD_HALF * 1.5, PAD_RELIEF * slack)?;
    let whole = relief(view, at, FOOTPRINT as f32, MAX_RELIEF * slack)?;
    inland(view, at).then_some(at.length() + whole * RELIEF_COST.0 + pad * RELIEF_COST.1)
}

/// Returns the cheapest site on rings around the origin of `view`, with `slack` times the
/// relief limits.
fn search(view: &Terrain, slack: f32) -> Option<Vec2> {
    let (first, step, rings) = SEARCH;
    let mut best: Option<(f32, Vec2)> = None;
    for reach in (0..rings).map(|k| first + step * k as f32) {
        if best.is_some_and(|(cost, _)| reach > cost) {
            break;
        }
        let rated = circle(Vec2::ZERO, reach, (TAU * reach / step) as u32)
            .filter_map(|at| Some((cost(view, at, slack)?, at)));
        best = rated.chain(best).min_by(|a, b| a.0.total_cmp(&b.0));
    }
    best.map(|(_, at)| at)
}

/// Returns the site of the base in the metric `view` centred on the terrain spawn: the cheapest
/// fit within the relief limits, else within twice the limits, or nothing when no ground within
/// reach can take the base.
pub fn find(view: &Terrain) -> Option<Vec2> {
    SLACK.into_iter().find_map(|slack| search(view, slack))
}
