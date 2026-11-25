//! The geometry of each part shape: beveled blocks, multi-ring ellipsoids, curved tapers, curved spikes and arched fans.
//!
//! Tapers, spikes and fans hang along -Y from the socket at the origin. Blocks and balls are centred on it.

use std::f32::consts::TAU;

use bevy::math::{Vec2, Vec3};

use super::soup::Soup;
use super::{Part, PartKind, ShapeKind};

/// Returns the number of sides around the axis of `part`: higher on torsos, heads and limbs for clean silhouettes.
pub fn sides(part: &Part) -> usize {
    match (part.kind, part.shape) {
        (PartKind::Torso | PartKind::Bell | PartKind::Blob, ShapeKind::Ball) => 12,
        (PartKind::Eye, _) => 8,
        (_, ShapeKind::Ball) => 10,
        (_, ShapeKind::Taper) => 8,
        (_, ShapeKind::Spike) => 8,
        (_, ShapeKind::Block) => 8,
        (_, ShapeKind::Fan) => 6,
    }
}

/// Returns the flat white geometry of `part` in its own frame.
pub fn geometry(part: &Part) -> Soup {
    let n = sides(part);
    let half = part.size * 0.5;
    match part.shape {
        ShapeKind::Block => block(half),
        ShapeKind::Ball => ball(half, n, part.kind != PartKind::Eye),
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

/// Returns a chamfered carapace block of half extents `half` centred on the origin: 64 triangles.
fn block(half: Vec3) -> Soup {
    let mut soup = Soup::default();
    let bevel = |y: f32, scale: f32| -> Vec<Vec3> {
        let c = 0.74_f32;
        [
            [c, 1.0],
            [-c, 1.0],
            [-1.0, c],
            [-1.0, -c],
            [-c, -1.0],
            [c, -1.0],
            [1.0, -c],
            [1.0, c],
        ]
        .into_iter()
        .map(|[x, z]| Vec3::new(x * half.x * scale, y, z * half.z * scale))
        .collect()
    };
    let rings = [
        bevel(half.y, 0.78),
        bevel(0.68 * half.y, 1.0),
        bevel(-0.68 * half.y, 1.0),
        bevel(-half.y, 0.78),
    ];
    let poles = [Vec3::Y * half.y, Vec3::NEG_Y * half.y];
    for i in 0..8 {
        let j = (i + 1) % 8;
        soup.tri([poles[0], rings[0][i], rings[0][j]], poles[0]);
        for band in 0..3 {
            let quad = [
                rings[band][i],
                rings[band][j],
                rings[band + 1][j],
                rings[band + 1][i],
            ];
            soup.quad(quad, quad[0] + quad[2]);
        }
        soup.tri([poles[1], rings[3][i], rings[3][j]], poles[1]);
    }
    soup
}

/// Returns a smooth ellipsoid ball of `n` sides: four latitude rings when `fine`, else two for small eyes.
fn ball(half: Vec3, n: usize, fine: bool) -> Soup {
    let mut soup = Soup::default();
    let lats: &[f32] = if fine {
        &[1.01, 0.35, -0.35, -1.01]
    } else {
        &[std::f32::consts::FRAC_PI_6, -std::f32::consts::FRAC_PI_6]
    };
    let rings: Vec<Vec<Vec3>> = lats
        .iter()
        .map(|&a| ring(n, half.y * a.sin(), Vec2::new(half.x, half.z) * a.cos()))
        .collect();
    let poles = [Vec3::Y * half.y, Vec3::NEG_Y * half.y];
    let last = rings.len() - 1;
    for i in 0..n {
        let j = (i + 1) % n;
        soup.tri(
            [poles[0], rings[0][i], rings[0][j]],
            poles[0] + rings[0][i] + rings[0][j],
        );
        for band in 0..last {
            let quad = [
                rings[band][i],
                rings[band][j],
                rings[band + 1][j],
                rings[band + 1][i],
            ];
            soup.quad(quad, quad[0] + quad[2]);
        }
        soup.tri(
            [poles[1], rings[last][i], rings[last][j]],
            poles[1] + rings[last][i] + rings[last][j],
        );
    }
    soup
}

/// Returns a three-ring curved taper of extent `size` hanging from the origin, with a domed socket cap
/// and a closed tip: `6n - 2` triangles.
fn taper(size: Vec3, taper: f32, n: usize) -> Soup {
    let mut soup = Soup::default();
    let base = Vec2::new(size.x, size.z) * 0.5;
    let mid = base * (0.65 + 0.35 * taper);
    let rings = [
        ring(n, 0.0, base),
        ring(n, -0.45 * size.y, mid),
        ring(n, -size.y, base * taper),
    ];
    let dome = Vec3::Y * 0.5 * base.min_element();
    let radial = |p: Vec3| Vec3::new(p.x, 0.0, p.z);
    for i in 0..n {
        let j = (i + 1) % n;
        soup.tri(
            [dome, rings[0][i], rings[0][j]],
            radial(rings[0][i] + rings[0][j]) + dome,
        );
        for band in 0..2 {
            let quad = [
                rings[band][i],
                rings[band][j],
                rings[band + 1][j],
                rings[band + 1][i],
            ];
            soup.quad(quad, radial(quad[0] + quad[2]));
        }
    }
    for i in 1..n - 1 {
        soup.tri([rings[2][0], rings[2][i], rings[2][i + 1]], Vec3::NEG_Y);
    }
    soup
}

/// Returns a curved two-ring cone of extent `size` from the origin to -Y: `4n - 2` triangles.
fn spike(size: Vec3, n: usize) -> Soup {
    let mut soup = Soup::default();
    let base = Vec2::new(size.x, size.z) * 0.5;
    let rings = [ring(n, 0.0, base), ring(n, -0.5 * size.y, base * 0.58)];
    let tip = Vec3::NEG_Y * size.y;
    for i in 0..n {
        let j = (i + 1) % n;
        let band = [rings[0][i], rings[0][j], rings[1][j], rings[1][i]];
        soup.quad(
            band,
            Vec3::new(band[0].x + band[2].x, 0.0, band[0].z + band[2].z),
        );
        soup.tri(
            [tip, rings[1][i], rings[1][j]],
            Vec3::new(
                rings[1][i].x + rings[1][j].x,
                0.0,
                rings[1][i].z + rings[1][j].z,
            ),
        );
    }
    for i in 1..n - 1 {
        soup.tri([rings[0][0], rings[0][i], rings[0][i + 1]], Vec3::Y);
    }
    soup
}

/// Returns a curved 6-segment arched fan in the YZ plane, seen from both sides: 36 triangles.
fn fan(size: Vec3) -> Soup {
    let mut soup = Soup::default();
    let outer = |u: f32| {
        Vec3::new(
            0.03 * size.y * (1.0 - u * u),
            -size.y * (1.0 - 0.25 * u * u),
            0.5 * size.z * u,
        )
    };
    let mid = |p: Vec3, u: f32| p * 0.52 + Vec3::X * 0.025 * size.y * (1.0 - u * u);
    let steps = [-1.0, -2.0 / 3.0, -1.0 / 3.0, 0.0, 1.0 / 3.0, 2.0 / 3.0, 1.0];
    let rim: Vec<Vec3> = steps.into_iter().map(outer).collect();
    let ribs: Vec<Vec3> = rim.iter().zip(steps).map(|(&p, u)| mid(p, u)).collect();
    for w in 0..6 {
        for out in [Vec3::X, Vec3::NEG_X] {
            soup.tri([Vec3::ZERO, ribs[w], ribs[w + 1]], out);
            soup.quad([ribs[w], rim[w], rim[w + 1], ribs[w + 1]], out);
        }
    }
    soup
}
