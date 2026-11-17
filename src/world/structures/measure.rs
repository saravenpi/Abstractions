//! Test-only measurement on triangle lists: the faces that fight for the same plane.

use bevy::prelude::*;

/// How deep two coplanar faces must overlap, in meters, before they count as fighting.
const DEPTH: f32 = 0.004;

/// A triangle index with the unit normal and the offset of its plane.
type Plane = (usize, Vec3, f32);

/// Returns the unit normal and plane offset of a triangle, or None when it has no area.
pub fn plane(t: &[Vec3; 3]) -> Option<(Vec3, f32)> {
    let n = (t[1] - t[0]).cross(t[2] - t[0]);
    (n.length() > 1e-5).then(|| (n.normalize(), n.normalize().dot(t[0])))
}

/// Returns the part of triangle `a` that lies inside triangle `b`, both in the plane of normal `n` and wound
/// around it, as a convex polygon.
fn shared(n: Vec3, a: &[Vec3; 3], b: &[Vec3; 3]) -> Vec<Vec3> {
    let mut poly = a.to_vec();
    for i in 0..3 {
        let (from, edge) = (b[i], b[(i + 1) % 3] - b[i]);
        let side = |p: Vec3| n.dot(edge.cross(p - from));
        let mut kept = Vec::new();
        for (k, p) in poly.iter().enumerate() {
            let q = poly[(k + 1) % poly.len()];
            let (sp, sq) = (side(*p), side(q));
            kept.extend((sp >= 0.0).then_some(*p));
            kept.extend(((sp >= 0.0) != (sq >= 0.0)).then(|| *p + (q - *p) * (sp / (sp - sq))));
        }
        poly = kept;
    }
    poly
}

/// Returns how deep the convex polygon `poly` is: twice its area over its outline, about its narrow width.
fn depth(poly: &[Vec3]) -> f32 {
    let turn = |i: usize| (poly[i] - poly[0]).cross(poly[(i + 1) % poly.len()] - poly[0]);
    let area = (1..poly.len().max(1)).map(turn).sum::<Vec3>().length() * 0.5;
    let outline: f32 = (0..poly.len())
        .map(|i| poly[i].distance(poly[(i + 1) % poly.len()]))
        .sum();
    2.0 * area / outline.max(1e-6)
}

/// Returns true when the point `p` of the plane of triangle `t`, whose normal is `n`, lies inside it.
fn inside(t: &[Vec3; 3], n: Vec3, p: Vec3) -> bool {
    (0..3).all(|i| n.dot((t[(i + 1) % 3] - t[i]).cross(p - t[i])) >= -1e-5)
}

/// Returns true when the overlap `poly` of two fighting faces of normal `n` shows: some point of it is not
/// covered by a face of the same plane that looks the other way, as between two boxes that touch.
fn shows(planes: &[Plane], tris: &[[Vec3; 3]], poly: &[Vec3], n: Vec3) -> bool {
    let d = n.dot(poly[0]);
    let facing = |p: &&Plane| p.1.dot(n) < -0.999_99 && (p.2 + d).abs() < 2e-3;
    let covers: Vec<&Plane> = planes.iter().filter(facing).collect();
    let middle = poly.iter().sum::<Vec3>() / poly.len() as f32;
    let spots = poly
        .iter()
        .map(|p| middle + (*p - middle) * 0.8)
        .chain([middle]);
    let bare = |p: Vec3| !covers.iter().any(|c| inside(&tris[c.0], c.1, p));
    spots.into_iter().any(bare)
}

/// Returns the pairs of triangles of `verts`, as triangle indices, that face the same way in the same plane,
/// overlap by more than `DEPTH` and show: the faces a renderer cannot order, which flicker.
pub fn fights(verts: &[Vec3]) -> Vec<(usize, usize)> {
    let tris = verts.as_chunks::<3>().0;
    let mut planes: Vec<Plane> = tris
        .iter()
        .enumerate()
        .filter_map(|(i, t)| plane(t).map(|(n, d)| (i, n, d)))
        .collect();
    planes.sort_by(|a, b| a.1.x.total_cmp(&b.1.x));
    let mut out = Vec::new();
    for (k, (i, n, d)) in planes.iter().enumerate() {
        let near = planes[k + 1..].iter().take_while(|p| p.1.x - n.x < 2e-3);
        let same = near.filter(|(_, m, e)| n.dot(*m) > 0.999_99 && (d - e).abs() < 1.5e-3);
        let clash = |j: usize| {
            let poly = shared(*n, &tris[*i], &tris[j]);
            poly.len() >= 3 && depth(&poly) > DEPTH && shows(&planes, tris, &poly, *n)
        };
        out.extend(same.filter(|(j, ..)| clash(*j)).map(|(j, ..)| (*i, *j)));
    }
    out
}
