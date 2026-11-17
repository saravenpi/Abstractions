//! Visual-only ground: the plaza disc, the street strips and the props closing each street.

use std::f32::consts::TAU;

use bevy::prelude::*;

use super::layout::{Layout, PLAZA};
use super::lots::ROAD_HALF;
use super::props;
use super::style::Style;
use crate::world::structures::brush::Build;

/// Height of the ground strips above the terrain mesh.
const LIFT: f32 = 0.12;

/// Extra height of the footpaths, so that they do not flicker where they meet a street.
const OVER: f32 = 0.015;

/// Longest edge of a ground quad, short enough to follow the 2 m terrain triangles.
const EDGE: f32 = 1.15;

/// Returns the chunk-space point on the visible ground at `p`, raised by `lift`.
fn ground(b: &Build, p: Vec2, lift: f32) -> Vec3 {
    b.world_pt(p, b.mesh_y(p) + lift)
}

/// Draws the ground between four world corners as `n.0` by `n.1` quads with every vertex on the terrain mesh.
fn patch(b: &mut Build, c: [Vec2; 4], n: (u32, u32), coat: ([f32; 3], f32)) {
    let color = b.color(coat.0, 1.0);
    let at = |i: u32, j: u32| {
        let (u, v) = (i as f32 / n.0 as f32, j as f32 / n.1 as f32);
        ground(b, c[0].lerp(c[1], u).lerp(c[3].lerp(c[2], u), v), coat.1)
    };
    let grid: Vec<Vec<Vec3>> = (0..=n.0)
        .map(|i| (0..=n.1).map(|j| at(i, j)).collect())
        .collect();
    for (lo, hi) in grid.iter().zip(&grid[1..]) {
        for j in 0..n.1 as usize {
            b.quad_out([lo[j], hi[j], hi[j + 1], lo[j + 1]], Vec3::Y, color);
        }
    }
}

/// Draws the packed-earth plaza disc around the village center as rings of short quads.
fn plaza(b: &mut Build, c: Vec2, st: &Style) {
    let rings = (PLAZA / EDGE).ceil() as u32;
    for k in 0..rings {
        let (r0, r1) = (
            PLAZA * k as f32 / rings as f32,
            PLAZA * (k + 1) as f32 / rings as f32,
        );
        let n = (TAU * r1 / (EDGE * 1.2)).ceil() as u32;
        let ring = |r: f32, i: u32| c + Vec2::from_angle(i as f32 / n as f32 * TAU) * r;
        for i in 0..n {
            let corners = [ring(r0, i), ring(r1, i), ring(r1, i + 1), ring(r0, i + 1)];
            patch(b, corners, (1, 1), (st.road, LIFT));
        }
    }
}

/// Draws a street strip along `pts`, its edges joined at every sample.
fn road(b: &mut Build, pts: &[Vec2], rgb: [f32; 3]) {
    let last = pts.len() - 1;
    let side = |i: usize| {
        let dir = (pts[(i + 1).min(last)] - pts[i.saturating_sub(1)]).normalize_or_zero();
        Vec2::new(-dir.y, dir.x) * ROAD_HALF
    };
    let across = (ROAD_HALF * 2.0 / EDGE).ceil() as u32;
    for i in 0..last {
        let (p, q, n, m) = (pts[i], pts[i + 1], side(i), side(i + 1));
        let along = (p.distance(q) / EDGE).ceil().max(1.0) as u32;
        patch(
            b,
            [p - n, p + n, q + m, q - m],
            (across, along),
            (rgb, LIFT),
        );
    }
}

/// Draws a footpath of width 0.9 from `a` to `c`, in quads short enough to follow the terrain.
fn path(b: &mut Build, a: Vec2, c: Vec2, rgb: [f32; 3]) {
    let len = a.distance(c);
    if len < 0.6 {
        return;
    }
    let side = Vec2::new(a.y - c.y, c.x - a.x) / len * 0.45;
    let n = (len / EDGE).ceil() as u32;
    patch(
        b,
        [a - side, a + side, c + side, c - side],
        (1, n),
        (rgb, LIFT + OVER),
    );
}

/// Draws the visual-only ground of the village: the plaza, the streets and the footpaths. Every vertex
/// follows the ground on its own.
pub fn surface(b: &mut Build, lay: &Layout) {
    b.hug();
    plaza(b, lay.site.origin, &lay.style);
    for pts in &lay.streets {
        road(b, pts, lay.style.road);
    }
    let dust = lay.style.road.map(|c| c * 0.85);
    for (from, to) in &lay.paths {
        path(b, *from, *to, dust);
    }
    b.stiffen();
}

/// Draws street lamps every 12 m along both sides in turn, and four around the plaza.
fn lamps(b: &mut Build, lay: &Layout) {
    for (si, pts) in lay.streets.iter().enumerate() {
        for k in (5..pts.len() - 1).step_by(6) {
            let side = if (si + k / 6) % 2 == 0 { 1.0 } else { -1.0 };
            let dir = (pts[k + 1] - pts[k]).normalize();
            let normal = Vec2::new(-dir.y, dir.x) * side;
            props::lamp_at(b, pts[k] + normal * (ROAD_HALF + 0.5));
        }
    }
    for i in 0..4 {
        let a = lay.site.yaw + 0.8 + i as f32 * TAU / 4.0;
        props::lamp_at(b, lay.site.origin + Vec2::from_angle(a) * (PLAZA - 0.6));
    }
}

/// Draws the plaza, the streets, the footpaths, the lamps and the closing props of the village.
pub fn draw(b: &mut Build, lay: &Layout) {
    surface(b, lay);
    lamps(b, lay);
    for end in &lay.ends {
        props::end_prop(b, end);
    }
}
