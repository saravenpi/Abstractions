//! The geometry of each part shape: boxes, low-poly balls, tapers with a domed joint end, spikes and fans.
//!
//! Tapers, spikes and fans hang along -Y from the socket at the origin. Blocks and balls are centred on it.

use std::f32::consts::TAU;

use bevy::math::{Vec2, Vec3};

use super::soup::Soup;
use super::{Part, PartKind, ShapeKind};

/// Returns the number of sides of the rings of a part: eight for big soft bodies, six for crystals
/// and most parts, four for eyes.
pub fn sides(part: &Part) -> usize {
    match part.kind {
        PartKind::Torso | PartKind::Bell | PartKind::Blob if part.shape == ShapeKind::Ball => 8,
        PartKind::Eye => 4,
        _ => 6,
    }
}

/// Returns the flat white geometry of `part` in its own frame.
pub fn geometry(part: &Part) -> Soup {
    let n = sides(part);
    let half = part.size * 0.5;
    match part.shape {
        ShapeKind::Block => block(half),
        ShapeKind::Ball => ball(half, n),
        ShapeKind::Taper => taper(part.size, part.taper.max(0.05), n),
        ShapeKind::Spike => spike(part.size, n),
        ShapeKind::Fan => fan(part.size),
    }
}

/// Returns `n` points on an ellipse of half axes `half` (x, z) at height `y`, symmetric across x = 0.
fn ring(n: usize, y: f32, half: Vec2) -> Vec<Vec3> {
    (0..n)
        .map(|i| (i as f32 + 0.5) / n as f32 * TAU)
        .map(|a| Vec3::new(half.x * a.cos(), y, half.y * a.sin()))
        .collect()
}

/// Returns a box of half extents `half` centred on the origin: twelve triangles.
fn block(half: Vec3) -> Soup {
    let mut soup = Soup::default();
    let corner = |x: f32, y: f32, z: f32| Vec3::new(x, y, z) * half;
    for axis in 0..3 {
        for sign in [-1.0, 1.0] {
            let pick = |u: f32, v: f32| {
                let mut c = [0.0; 3];
                c[axis] = sign;
                c[(axis + 1) % 3] = u;
                c[(axis + 2) % 3] = v;
                corner(c[0], c[1], c[2])
            };
            let quad = [
                pick(-1.0, -1.0),
                pick(1.0, -1.0),
                pick(1.0, 1.0),
                pick(-1.0, 1.0),
            ];
            soup.quad(quad, (quad[0] + quad[2]) * 0.5);
        }
    }
    soup
}

/// Returns a ball of half extents `half` centred on the origin: two rings of `n` sides and two poles.
fn ball(half: Vec3, n: usize) -> Soup {
    let mut soup = Soup::default();
    let wide = Vec2::new(half.x, half.z) * 0.866;
    let rings = [ring(n, 0.5 * half.y, wide), ring(n, -0.5 * half.y, wide)];
    let poles = [Vec3::Y * half.y, Vec3::NEG_Y * half.y];
    for i in 0..n {
        let j = (i + 1) % n;
        let (top, low) = (&rings[0], &rings[1]);
        let band = [top[i], top[j], low[j], low[i]];
        soup.quad(band, band[0] + band[2]);
        soup.tri([poles[0], top[i], top[j]], poles[0] + top[i] + top[j]);
        soup.tri([poles[1], low[i], low[j]], poles[1] + low[i] + low[j]);
    }
    soup
}

/// Returns a taper of extent `size` hanging from the origin, its tip `taper` times its base, with a low dome
/// over the joint end and a flat tip: `4n - 2` triangles.
fn taper(size: Vec3, taper: f32, n: usize) -> Soup {
    let mut soup = Soup::default();
    let base = Vec2::new(size.x, size.z) * 0.5;
    let rings = [ring(n, 0.0, base), ring(n, -size.y, base * taper)];
    let dome = Vec3::Y * 0.5 * base.min_element();
    let radial = |p: Vec3| Vec3::new(p.x, 0.0, p.z);
    for i in 0..n {
        let j = (i + 1) % n;
        let band = [rings[0][i], rings[0][j], rings[1][j], rings[1][i]];
        soup.quad(band, radial(band[0] + band[2]));
        soup.tri(
            [dome, rings[0][i], rings[0][j]],
            radial(rings[0][i] + rings[0][j]) + dome,
        );
    }
    for i in 1..n - 1 {
        soup.tri([rings[1][0], rings[1][i], rings[1][i + 1]], Vec3::NEG_Y);
    }
    soup
}

/// Returns a cone of extent `size` from a flat base at the origin to a point down -Y: `2n - 2` triangles.
fn spike(size: Vec3, n: usize) -> Soup {
    let mut soup = Soup::default();
    let rim = ring(n, 0.0, Vec2::new(size.x, size.z) * 0.5);
    let tip = Vec3::NEG_Y * size.y;
    for i in 0..n {
        let j = (i + 1) % n;
        soup.tri(
            [tip, rim[i], rim[j]],
            Vec3::new(rim[i].x + rim[j].x, 0.0, rim[i].z + rim[j].z),
        );
    }
    for i in 1..n - 1 {
        soup.tri([rim[0], rim[i], rim[i + 1]], Vec3::Y);
    }
    soup
}

/// Returns a thin fan in the YZ plane, `size.y` long down -Y and `size.z` wide, seen from both sides: six
/// triangles.
fn fan(size: Vec3) -> Soup {
    let mut soup = Soup::default();
    let point = |u: f32| Vec3::new(0.0, -size.y * (1.0 - 0.25 * u.abs()), 0.5 * size.z * u);
    let arc = [-1.0, -1.0 / 3.0, 1.0 / 3.0, 1.0].map(point);
    for w in arc.windows(2) {
        for out in [Vec3::X, Vec3::NEG_X] {
            soup.tri([Vec3::ZERO, w[0], w[1]], out);
        }
    }
    soup
}
