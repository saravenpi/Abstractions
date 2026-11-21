//! Shared checks for tests of a built body: its closed pieces, their volume, what holds what, and
//! whether a set of pieces is one piece.

use std::collections::HashMap;

use bevy::prelude::*;

use super::soup::Soup;

/// A triangle as its three corners.
pub type Tri = [Vec3; 3];

/// A corner rounded to a hundredth of a millimetre.
type Key = [i32; 3];

/// Returns the key of a corner.
fn key(p: Vec3) -> Key {
    p.to_array().map(|v| (v * 1.0e5).round() as i32)
}

/// Returns the root of the set of `i` in the sets of `parent`.
fn find(parent: &mut [usize], i: usize) -> usize {
    if parent[i] != i {
        let root = find(parent, parent[i]);
        parent[i] = root;
    }
    parent[i]
}

/// Returns the pieces of `soup`: the groups of triangles joined through shared corners. Every
/// shape a body is built from is one piece.
pub fn pieces(soup: &Soup) -> Vec<Vec<Tri>> {
    let tris: Vec<Tri> = soup.tris().collect();
    let mut parent: Vec<usize> = (0..tris.len()).collect();
    let mut seen: HashMap<Key, usize> = HashMap::new();
    for (i, corner) in tris.iter().enumerate().flat_map(|(i, t)| t.map(|p| (i, p))) {
        let other = *seen.entry(key(corner)).or_insert(i);
        let (a, b) = (find(&mut parent, i), find(&mut parent, other));
        parent[a] = b;
    }
    let mut groups: HashMap<usize, Vec<Tri>> = HashMap::new();
    for (i, tri) in tris.iter().enumerate() {
        groups.entry(find(&mut parent, i)).or_default().push(*tri);
    }
    let mut all: Vec<Vec<Tri>> = groups.into_values().collect();
    all.sort_by(|a, b| a[0][0].to_array().partial_cmp(&b[0][0].to_array()).unwrap());
    all
}

/// Returns how many edges of `piece` do not belong to exactly two triangles: none when it is closed.
pub fn open_edges(piece: &[Tri]) -> usize {
    let mut count: HashMap<(Key, Key), u32> = HashMap::new();
    for tri in piece {
        for i in 0..3 {
            let (a, b) = (key(tri[i]), key(tri[(i + 1) % 3]));
            *count.entry((a.min(b), a.max(b))).or_default() += 1;
        }
    }
    count.into_values().filter(|n| *n != 2).count()
}

/// Returns the volume `piece` encloses, in cubic meters: positive when its triangles face outward.
pub fn volume(piece: &[Tri]) -> f32 {
    piece.iter().map(|[a, b, c]| a.dot(b.cross(*c)) / 6.0).sum()
}

/// Returns whether `p` is inside the closed `piece`: a ray from it leaves through an odd number
/// of triangles.
pub fn inside(piece: &[Tri], p: Vec3) -> bool {
    let ray = Vec3::new(0.371_3, 0.829_1, 0.417_7);
    let crossed = |[a, b, c]: &Tri| {
        let (e1, e2) = (*b - *a, *c - *a);
        let h = ray.cross(e2);
        let det = e1.dot(h);
        let s = p - *a;
        let (u, q) = (s.dot(h) / det, s.cross(e1));
        let (v, t) = (ray.dot(q) / det, e2.dot(q) / det);
        det.abs() > 1e-12 && u >= 0.0 && v >= 0.0 && u + v <= 1.0 && t > 0.0
    };
    piece.iter().filter(|tri| crossed(tri)).count() % 2 == 1
}

/// Returns whether `a` and `b` are attached: a corner or the middle of a triangle of one is inside
/// the other.
fn attached(a: &[Tri], b: &[Tri]) -> bool {
    let probes = |piece: &[Tri]| -> Vec<Vec3> {
        let middles = piece.iter().map(|t| (t[0] + t[1] + t[2]) / 3.0);
        piece.iter().flatten().copied().chain(middles).collect()
    };
    probes(a).into_iter().any(|p| inside(b, p)) || probes(b).into_iter().any(|p| inside(a, p))
}

/// Returns the index of every piece of `all` that is not attached, directly or through others,
/// to the first one: none when they make one piece.
pub fn floating(all: &[Vec<Tri>]) -> Vec<usize> {
    let mut parent: Vec<usize> = (0..all.len()).collect();
    for i in 0..all.len() {
        for j in 0..i {
            if find(&mut parent, i) != find(&mut parent, j) && attached(&all[i], &all[j]) {
                let (a, b) = (find(&mut parent, i), find(&mut parent, j));
                parent[a] = b;
            }
        }
    }
    let first = find(&mut parent, 0);
    (0..all.len())
        .filter(|i| find(&mut parent, *i) != first)
        .collect()
}
