//! The sea mesh: one flat grid in chart metres, with 2 m cells near the focus and wider and wider
//! cells out to where the coarse planet mesh takes over. The shader wraps it onto the sphere.

use bevy::asset::RenderAssetUsages;
use bevy::mesh::{Indices, PrimitiveTopology};
use bevy::prelude::*;

/// Side of a fine cell in meters, and the grid the mesh snaps to.
pub const CELL: f32 = 2.0;

/// Half side in meters of the fine zone.
pub const FINE_HALF: f32 = 96.0;

/// Cell width in meters and number of cells of each ring around the fine zone, inside out.
/// They take the half side of the whole sea to 1552 m.
///
/// A straight cell edge sags below the sphere by its length squared over eight radii. The 14 m
/// cells keep that under 2 cm out to 208 m on the smallest planet, and the widest cells stay
/// above the flat sea of the coarse planet mesh.
const RINGS: [(f32, usize); 7] = [
    (4.0, 1),
    (8.0, 1),
    (16.0, 1),
    (14.0, 6),
    (32.0, 2),
    (64.0, 2),
    (128.0, 9),
];

/// Returns the grid lines along one axis, in rising order: a fine line every cell, then the rings.
pub fn lines() -> Vec<f32> {
    let steps = (2.0 * FINE_HALF / CELL) as usize;
    let widths = RINGS
        .iter()
        .flat_map(|&(width, count)| std::iter::repeat_n(width, count));
    let outer: Vec<f32> = widths
        .scan(FINE_HALF, |edge, width| {
            *edge += width;
            Some(*edge)
        })
        .collect();
    let mut lines: Vec<f32> = outer.iter().rev().map(|edge| -edge).collect();
    lines.extend((0..=steps).map(|i| -FINE_HALF + i as f32 * CELL));
    lines.extend(outer);
    lines
}

/// Builds the flat sea mesh, centered on the origin, with positions only.
pub fn sea_mesh() -> Mesh {
    let lines = lines();
    let width = lines.len();
    let positions: Vec<[f32; 3]> = lines
        .iter()
        .flat_map(|&z| lines.iter().map(move |&x| [x, 0.0, z]))
        .collect();
    let mut indices = Vec::with_capacity((width - 1) * (width - 1) * 6);
    for row in 0..width - 1 {
        for col in 0..width - 1 {
            let a = (row * width + col) as u32;
            let b = a + 1;
            let c = a + width as u32;
            let d = c + 1;
            indices.extend([a, c, b, b, c, d]);
        }
    }
    Mesh::new(
        PrimitiveTopology::TriangleList,
        RenderAssetUsages::default(),
    )
    .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, positions)
    .with_inserted_indices(Indices::U32(indices))
}

/// Snaps a chart position to the fine grid so vertices never swim while the sea follows the focus.
pub fn snap(p: Vec3) -> Vec2 {
    Vec2::new((p.x / CELL).round() * CELL, (p.z / CELL).round() * CELL)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The mesh stays under the triangle budget and the snap lands on cell multiples.
    #[test]
    fn mesh_is_cheap_and_snaps() {
        let mesh = sea_mesh();
        let triangles = mesh.indices().map_or(0, |i| i.len() / 3);
        assert!(triangles > 18_000 && triangles <= 40_000, "{triangles}");
        let snapped = snap(Vec3::new(13.3, 5.0, -7.9));
        assert_eq!(snapped, Vec2::new(14.0, -8.0));
        assert_eq!(snapped.x % CELL, 0.0);
    }
}
