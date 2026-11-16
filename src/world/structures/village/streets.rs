//! Visual-only ground: the plaza disc, the street strips and the props closing each street.

use std::f32::consts::TAU;

use bevy::prelude::*;

use super::layout::{Layout, PLAZA};
use super::lots::ROAD_HALF;
use super::props;
use super::style::Style;
use crate::world::structures::brush::Build;

/// Height of the ground strips above the terrain.
const LIFT: f32 = 0.1;

/// Returns the chunk-space point on the ground at `p`, lifted slightly.
fn ground(b: &Build, p: Vec2) -> Vec3 {
    b.world_pt(p, b.terrain.height(p.x, p.y) + LIFT)
}

/// Draws a flat ground quad between four world points, following the terrain.
fn strip(b: &mut Build, c: [Vec2; 4], rgb: [f32; 3]) {
    let color = b.color(rgb, 1.0);
    b.quad_out(c.map(|p| ground(b, p)), Vec3::Y, color);
}

/// Draws the packed-earth plaza disc around the village center.
fn plaza(b: &mut Build, c: Vec2, st: &Style) {
    let ring = |r: f32, i: u32| c + Vec2::from_angle(i as f32 / 12.0 * TAU) * r;
    let color = b.color(st.road, 0.92);
    for i in 0..12 {
        b.quad_out(
            [
                ground(b, ring(0.0, i)),
                ground(b, ring(3.2, i)),
                ground(b, ring(3.2, i + 1)),
                ground(b, ring(0.0, i + 1)),
            ],
            Vec3::Y,
            color,
        );
        strip(
            b,
            [
                ring(3.2, i),
                ring(PLAZA, i),
                ring(PLAZA, i + 1),
                ring(3.2, i + 1),
            ],
            st.road,
        );
    }
}

/// Draws a footpath of width 0.9 from `a` to `c`, one quad per 2 m following the terrain.
fn path(b: &mut Build, a: Vec2, c: Vec2, rgb: [f32; 3]) {
    let len = a.distance(c);
    if len < 0.6 {
        return;
    }
    let dir = (c - a) / len;
    let side = Vec2::new(-dir.y, dir.x) * 0.45;
    let n = (len / 2.0).ceil() as u32;
    for i in 0..n {
        let p = a + dir * len * (i as f32 / n as f32);
        let q = a + dir * len * ((i + 1) as f32 / n as f32);
        strip(b, [p - side, p + side, q + side, q - side], rgb);
    }
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
    plaza(b, lay.site.origin, &lay.style);
    for pts in &lay.streets {
        for w in pts.windows(2) {
            let dir = (w[1] - w[0]).normalize();
            let n = Vec2::new(-dir.y, dir.x) * ROAD_HALF;
            strip(b, [w[0] - n, w[0] + n, w[1] + n, w[1] - n], lay.style.road);
        }
    }
    let dust = lay.style.road.map(|c| c * 0.85);
    for (from, to) in &lay.paths {
        path(b, *from, *to, dust);
    }
    lamps(b, lay);
    for end in &lay.ends {
        props::end_prop(b, end);
    }
}
