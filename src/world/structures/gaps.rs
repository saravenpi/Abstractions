//! Test-only measurements on triangle lists along a vertical line: where it crosses faces and how far
//! upright sheets carry it up.

use bevy::prelude::*;

use super::measure::plane;

/// Returns the heights at which the vertical line through `(x, z)` crosses the triangles `tris`.
pub fn column(tris: &[[Vec3; 3]], x: f32, z: f32) -> Vec<f32> {
    let p = Vec2::new(x, z);
    let cross = |a: Vec2, b: Vec2| a.x * b.y - a.y * b.x;
    let hit = |t: &[Vec3; 3]| {
        let [a, b, c] = t.map(|v| v.xz());
        let area = cross(b - a, c - a);
        let (u, v) = (cross(p - a, c - a) / area, cross(b - a, p - a) / area);
        let inside = area.abs() > 1e-6 && u >= -1e-4 && v >= -1e-4 && u + v <= 1.0 + 1e-4;
        inside.then(|| t[0].y + u * (t[1].y - t[0].y) + v * (t[2].y - t[0].y))
    };
    tris.iter().filter_map(hit).collect()
}

/// Returns, for every upright triangle of `tris` whose plane holds the vertical line through `(x, z)`, the
/// lowest and highest point of that line inside the triangle.
pub fn curtain(tris: &[[Vec3; 3]], x: f32, z: f32) -> Vec<(f32, f32)> {
    let p = Vec3::new(x, 0.0, z);
    let reach = |t: &[Vec3; 3]| {
        let (n, d) = plane(t)?;
        let upright = n.y.abs() < 1e-3 && (n.dot(p) - d).abs() < 0.01;
        let along = Vec3::Y.cross(n);
        let (s, at) = (p.dot(along), t.map(|v| Vec2::new(v.dot(along), v.y)));
        let cuts = [(0, 1), (1, 2), (2, 0)].into_iter().filter_map(|(i, j)| {
            let k = (s - at[i].x) / (at[j].x - at[i].x);
            (0.0..=1.0)
                .contains(&k)
                .then(|| at[i].y + k * (at[j].y - at[i].y))
        });
        let ys: Vec<f32> = cuts.collect();
        let span = (
            ys.iter().copied().fold(f32::MAX, f32::min),
            ys.iter().copied().fold(f32::MIN, f32::max),
        );
        (upright && ys.len() >= 2).then_some(span)
    };
    tris.iter().filter_map(reach).collect()
}

/// Returns the height the vertical line through `(x, z)` reaches when it climbs from `top` through the
/// upright triangles `tris`, each one starting no higher than where the last one ended.
pub fn climb(tris: &[[Vec3; 3]], at: Vec2, top: f32) -> f32 {
    let spans = curtain(tris, at.x, at.y);
    let mut top = top;
    for _ in 0..spans.len() {
        let next = spans.iter().filter(|s| s.0 <= top + 0.01).map(|s| s.1);
        top = next.fold(top, f32::max);
    }
    top
}
