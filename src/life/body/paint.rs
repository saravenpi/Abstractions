//! Paint: the tissue colour of a part with its pattern baked in as per-face shades.

use bevy::math::{IVec3, Vec3};

use super::shape::{geometry, sides};
use super::soup::{Face, Soup};
use super::{Part, ShapeKind};
use crate::life::{Pattern, Tissue};
use crate::universe::hash::cell;

/// Returns the flat-shaded, painted triangles of `part` made of `tissue`, in the frame of the part.
pub fn soup(part: &Part, tissue: &Tissue) -> Soup {
    let mut soup = geometry(part);
    let extent = part.size.max_element().max(1e-3);
    for face in &mut soup.faces {
        let k = shade(tissue.pattern, face, extent) * jitter(face);
        face.colour = tissue.colour.map(|c| (c * k).clamp(0.0, 1.0));
    }
    soup
}

/// Returns the number of triangles the mesh of `part` has, without building it.
pub fn triangles(part: &Part) -> usize {
    let n = sides(part);
    match part.shape {
        ShapeKind::Block => 12,
        ShapeKind::Ball => 4 * n,
        ShapeKind::Taper => 4 * n - 2,
        ShapeKind::Spike => 2 * n - 2,
        ShapeKind::Fan => 6,
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

/// Returns the brightness `pattern` gives a face on a part of largest extent `extent`.
fn shade(pattern: Pattern, face: &Face, extent: f32) -> f32 {
    let c = centre(face);
    let odd = |x: f32| (x.floor() as i64).rem_euclid(2) == 1;
    match pattern {
        Pattern::Plain => 1.0,
        Pattern::Stripes if odd((c.y + c.z) / (0.3 * extent)) => 0.7,
        Pattern::Bands if odd(c.x.abs() / (0.15 * extent)) => 0.75,
        Pattern::Spots if grain(face, 0x5207) < 0.22 => 0.55,
        Pattern::Countershade if face.normal.y > 0.35 => 0.72,
        Pattern::Countershade if face.normal.y < -0.35 => 1.18,
        Pattern::Countershade => 0.95,
        _ => 1.0,
    }
}
