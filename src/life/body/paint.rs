//! Paint: the tissue colour of a part with its pattern painted across two species tissues.

use bevy::math::{IVec3, Vec3};

use super::shape::sides;
use super::soup::{Face, Soup};
use super::{Part, PartKind, ShapeKind};
use crate::life::{Pattern, Tissue};
use crate::universe::hash::cell;

/// Returns the flat-shaded, painted triangles of `part` made of `tissue`, using single-tissue shading.
pub fn soup(part: &Part, tissue: &Tissue) -> Soup {
    soup_toned(part, tissue, None)
}

/// Returns the flat-shaded, painted triangles of `part` made of `tissue`, blending two-tone patterns
/// toward `accent` when available.
pub fn soup_toned(part: &Part, tissue: &Tissue, accent: Option<&Tissue>) -> Soup {
    let mut soup = super::shape::geometry(part);
    let extent = part.size.max_element().max(1e-3);
    let secondary = accent.map(|a| a.colour);
    for face in &mut soup.faces {
        let base = paint_face(tissue, secondary, face, extent);
        let k = jitter(face) * (0.96 + 0.06 * face.normal.y);
        face.colour = base.map(|c| (c * k).clamp(0.0, 1.0));
    }
    soup
}

/// Returns the number of triangles the mesh of `part` has, without building it.
pub fn triangles(part: &Part) -> usize {
    let n = sides(part);
    match part.shape {
        ShapeKind::Block => 64,
        ShapeKind::Ball if part.kind == PartKind::Eye => 4 * n,
        ShapeKind::Ball => 8 * n,
        ShapeKind::Taper => 6 * n - 2,
        ShapeKind::Spike => 4 * n - 2,
        ShapeKind::Fan => 36,
    }
}

/// Returns the centre of a face.
fn centre(face: &Face) -> Vec3 {
    (face.corners[0] + face.corners[1] + face.corners[2]) / 3.0
}

/// Returns a number from 0 to 1 that depends only on where a face sits, the same for its mirror image.
fn grain(face: &Face, salt: u64) -> f32 {
    let c = centre(face) * 1000.0;
    let at = IVec3::new(
        c.x.abs().round() as i32,
        c.y.round() as i32,
        c.z.round() as i32,
    );
    (cell(salt, at) >> 40) as f32 / (1u64 << 24) as f32
}

/// Returns a small brightness change per face, so that flat facets read apart.
fn jitter(face: &Face) -> f32 {
    1.0 + (grain(face, 0x51DE) - 0.5) * 0.1
}

/// Blends `a` toward `b` by `t` in `[0, 1]`.
fn mix(a: [f32; 3], b: [f32; 3], t: f32) -> [f32; 3] {
    [
        a[0] + (b[0] - a[0]) * t,
        a[1] + (b[1] - a[1]) * t,
        a[2] + (b[2] - a[2]) * t,
    ]
}

/// Returns the patterned linear RGB colour of a face before per-facet lighting and jitter.
fn paint_face(tissue: &Tissue, accent: Option<[f32; 3]>, face: &Face, extent: f32) -> [f32; 3] {
    let c = centre(face);
    let odd = |x: f32| (x.floor() as i64).rem_euclid(2) == 1;
    let alt = |darken: f32| {
        accent.map_or_else(
            || tissue.colour.map(|ch| ch * darken),
            |acc| mix(tissue.colour, acc, 0.85),
        )
    };
    match tissue.pattern {
        Pattern::Plain => tissue.colour,
        Pattern::Stripes if odd((c.y + c.z) / (0.28 * extent)) => alt(0.68),
        Pattern::Bands if odd(c.x.abs() / (0.16 * extent)) => alt(0.72),
        Pattern::Spots if grain(face, 0x5207) < 0.28 => alt(0.54),
        Pattern::Countershade if face.normal.y > 0.30 => tissue.colour.map(|ch| ch * 0.74),
        Pattern::Countershade if face.normal.y < -0.30 => accent.map_or_else(
            || tissue.colour.map(|ch| ch * 1.18),
            |acc| mix(tissue.colour, acc, 0.65),
        ),
        _ => tissue.colour,
    }
}
