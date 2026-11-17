//! True distances on a terrain view: between chart points, to the cube edges, and the size of a chart unit.

use std::f64::consts::SQRT_2;

use bevy::math::{DVec2, DVec3};
use bevy::prelude::*;

use crate::world::Terrain;
use crate::world::globe::Chart;

/// Distance in meters kept between the footprint of a site and any cube edge.
pub const EDGE: f32 = 150.0;

/// Returns the distance in meters along the ground between the chart points `a` and `b`.
pub fn metres(terrain: &Terrain, a: Vec2, b: Vec2) -> f32 {
    if matches!(terrain.chart(), Chart::Flat) {
        return a.distance(b);
    }
    let (from, to) = (terrain.up(a.x, a.y), terrain.up(b.x, b.y));
    (from.cross(to).length().atan2(from.dot(to)) * terrain.globe().radius()) as f32
}

/// Returns the distance in meters from chart point `p` to the nearest edge of its cube face: negative off
/// the face, and without limit on a chart that has no edges.
pub fn edge_distance(terrain: &Terrain, p: Vec2) -> f32 {
    let Chart::Face { face, scale } = terrain.chart() else {
        return f32::INFINITY;
    };
    let (u, v) = (f64::from(p.x) / scale, f64::from(p.y) / scale);
    if u.abs().max(v.abs()) >= 1.0 {
        return -1.0;
    }
    let [normal, a, b] = face.axes();
    let dir = face.dir(u, v);
    let sine = (dir.dot(normal) - dir.dot(a).abs().max(dir.dot(b).abs())) / SQRT_2;
    (sine.asin() * terrain.globe().radius()) as f32
}

/// Returns the largest number of chart units one meter covers around chart point `p`: 1 on a metric chart.
pub fn stretch(terrain: &Terrain, p: Vec2) -> f32 {
    if matches!(terrain.chart(), Chart::Flat) {
        return 1.0;
    }
    let step = |d: Vec2| metres(terrain, p, p + d * 8.0) / 8.0;
    1.0 / step(Vec2::X).min(step(Vec2::Y)).max(1e-3)
}

/// Returns the chart point of `to` that lies at the chart point `p` of the view `from` of the same planet.
pub fn transfer(from: &Terrain, to: &Terrain, p: Vec2) -> Vec2 {
    if from.chart() == to.chart() {
        return p;
    }
    to.locate(from.embed(p.x, 0.0, p.y)).xz()
}

/// Returns the chart position of the planet position `pos` in 64 bits, so that an offset from a nearby chart
/// point keeps its precision far from the middle of a cube face.
pub fn chart_point(terrain: &Terrain, pos: DVec3) -> DVec2 {
    match terrain.chart() {
        Chart::Face { face, scale } => {
            let (u, v) = face.project(pos.normalize());
            DVec2::new(u * scale, v * scale)
        }
        _ => terrain.locate(pos).xz().as_dvec2(),
    }
}
