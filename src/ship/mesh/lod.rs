//! The ship seen from far away: one mesh with the plating, the trim and the glass painted into
//! its vertex colours, without the fittings that are too small to see. One draw instead of
//! seven, for the many ships that are in view and far.

use bevy::asset::RenderAssetUsages;
use bevy::mesh::{Indices, PrimitiveTopology};
use bevy::prelude::*;

use super::parts::{Paint, parts};
use super::{Skin, material};
use crate::ship::tones::paint;

/// Metres from the camera beyond which a ship is drawn from the far mesh.
pub const FAR: f32 = 90.0;

/// Returns the linear colour of a paint, with the glass as the dark of its tint.
fn base(paint: Paint) -> LinearRgba {
    let skin = match paint {
        Paint::Hull => Skin::Hull,
        Paint::Trim => Skin::Trim,
        Paint::Glass => return LinearRgba::rgb(0.03, 0.05, 0.08),
        _ => Skin::Dark,
    };
    material(skin).base_color.to_linear()
}

/// Builds the far mesh of the ship: the parts that are kept far, flat shaded and coloured.
pub fn far_mesh() -> Mesh {
    let (mut positions, mut normals, mut colours) = (Vec::new(), Vec::new(), Vec::new());
    for part in parts().iter().filter(|part| part.far) {
        let base = base(part.paint);
        for (tri, tone) in part.solid.tris.iter().zip(&part.solid.tones) {
            let normal = (tri[1] - tri[0]).cross(tri[2] - tri[0]).normalize_or_zero();
            let tint = paint(*tone);
            let colour = [
                base.red * tint[0],
                base.green * tint[1],
                base.blue * tint[2],
                1.0,
            ];
            positions.extend(tri.map(|p| p.to_array()));
            normals.extend([normal.to_array(); 3]);
            colours.extend([colour; 3]);
        }
    }
    let count = positions.len();
    Mesh::new(
        PrimitiveTopology::TriangleList,
        RenderAssetUsages::default(),
    )
    .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, positions)
    .with_inserted_attribute(Mesh::ATTRIBUTE_NORMAL, normals)
    .with_inserted_attribute(Mesh::ATTRIBUTE_UV_0, vec![[0.5, 0.5]; count])
    .with_inserted_attribute(Mesh::ATTRIBUTE_TANGENT, vec![[1.0, 0.0, 0.0, 1.0]; count])
    .with_inserted_attribute(Mesh::ATTRIBUTE_COLOR, colours)
    .with_inserted_indices(Indices::U32((0..count as u32).collect()))
}
