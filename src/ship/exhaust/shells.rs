//! The mesh of the plumes: three nested shells, each a ring of vertices per [`RINGS`] with the
//! angle round the axis as the u of its UV, the share along it as the v, and the number of the
//! shell in the red of its vertex colour, which `exhaust.wesl` reads the layers by.

use std::f32::consts::TAU;

use bevy::asset::RenderAssetUsages;
use bevy::mesh::{Indices, PrimitiveTopology};
use bevy::prelude::*;

/// Sides of a shell.
const SIDES: u32 = 8;

/// The rings of a shell along the plume: the share of its length and the share of the radius at
/// the nozzle. It swells just after the nozzle, then narrows to a point.
const RINGS: [(f32, f32); 6] = [
    (0.0, 1.0),
    (0.08, 1.15),
    (0.25, 1.05),
    (0.5, 0.72),
    (0.75, 0.38),
    (1.0, 0.05),
];

/// The shells from the core out: the number the shader reads them by, and their share of the radius.
const SHELLS: [(f32, f32); 3] = [(0.0, 0.42), (0.5, 0.72), (1.0, 1.0)];

/// The vertices and the triangles of the plume mesh while it is made.
#[derive(Default)]
struct Soup {
    /// Where the vertices are.
    points: Vec<[f32; 3]>,
    /// Their normals, straight out from the axis.
    normals: Vec<[f32; 3]>,
    /// Their UVs: the angle round the axis and the share along it.
    uvs: Vec<[f32; 2]>,
    /// Their colours: the number of the shell in the red.
    colours: Vec<[f32; 4]>,
    /// The triangles.
    indices: Vec<u32>,
}

/// Adds the shell `number`, `share` of the unit radius wide, to the soup.
fn shell(soup: &mut Soup, (number, share): (f32, f32)) {
    let first = soup.points.len() as u32;
    for (along, radius) in RINGS {
        for i in 0..=SIDES {
            let turn = i as f32 / SIDES as f32;
            let (sin, cos) = (turn * TAU).sin_cos();
            soup.points
                .push([cos * radius * share, sin * radius * share, along]);
            soup.normals.push([cos, sin, 0.0]);
            soup.uvs.push([turn, along]);
            soup.colours.push([number, 0.0, 0.0, 1.0]);
        }
    }
    let width = SIDES + 1;
    for at in (0..(RINGS.len() as u32 - 1) * width).filter(|at| at % width < SIDES) {
        let a = first + at;
        soup.indices
            .extend([a, a + 1, a + width + 1, a, a + width + 1, a + width]);
    }
}

/// Builds the mesh of every plume: the three shells, each a ring of vertices per [`RINGS`] with
/// the angle round the axis as the u of its UV, the share along it as the v, and the number of
/// the shell in the red of its vertex colour.
pub fn jet_mesh() -> Mesh {
    let mut soup = Soup::default();
    for kind in SHELLS {
        shell(&mut soup, kind);
    }
    Mesh::new(
        PrimitiveTopology::TriangleList,
        RenderAssetUsages::default(),
    )
    .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, soup.points)
    .with_inserted_attribute(Mesh::ATTRIBUTE_NORMAL, soup.normals)
    .with_inserted_attribute(Mesh::ATTRIBUTE_UV_0, soup.uvs)
    .with_inserted_attribute(Mesh::ATTRIBUTE_COLOR, soup.colours)
    .with_inserted_indices(Indices::U32(soup.indices))
}
