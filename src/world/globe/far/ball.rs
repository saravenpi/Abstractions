//! The ball: the small sphere of a planet the sky draws once the world has handed the planet over.
//!
//! It is the far mesh of the world, baked through the same ground rules and the same colour
//! lookup, then cut down: every block of four by four cells becomes one cell in the mean colour
//! of the block, so a planet keeps its colours across the hand-over, coast for coast. It is a unit
//! sphere: the sky scales it. One ball is baked off the main thread for every solid body of a
//! star system.

use std::sync::Arc;

use bevy::asset::RenderAssetUsages;
use bevy::mesh::{Indices, PrimitiveTopology, VertexAttributeValues};
use bevy::prelude::*;

use super::{CELLS, bake};
use crate::world::colormap;
use crate::world::globe::Globe;
use crate::world::sea::sea_glow;
use crate::world::shapes::srgb;

/// Cells of the far mesh along one side of a cell of the ball.
pub const BLOCK: usize = 4;

/// Triangles of a ball: a sixteenth of the far mesh, about a pixel each where the sky takes the
/// planet over.
pub const TRIANGLES: usize = 3072;

/// A planet baked for the sky.
pub struct Ball {
    /// The unit sphere with the relief of the planet. Each vertex carries the linear colour of its
    /// cell, and in alpha the share of the cell the flat sea covers. Its normals are those of the
    /// smooth sphere: a planet a few pixels wide is lit as a ball, without the flicker of facets.
    pub mesh: Mesh,
    /// Linear colour of the whole surface, each cell counted by its area: what the planet shows
    /// when it is only a dot.
    pub mean: Vec3,
    /// Share of the surface the flat sea covers.
    pub sea: f32,
    /// Linear light the flat sea gives off by itself, none for water.
    pub glow: Vec3,
}

/// Returns the linear colour the colour lookup `image` holds at the texture coordinates `uv`.
pub(super) fn texel(image: &Image, uv: [f32; 2]) -> Vec3 {
    let size = image.texture_descriptor.size;
    let x = (uv[0] * size.width as f32) as usize;
    let y = (uv[1] * size.height as f32) as usize;
    let at = (y * size.width as usize + x) * 4;
    let data = image.data.as_deref().unwrap_or(&[]);
    let Some(&[r, g, b]) = data.get(at..at + 3).and_then(|t| t.as_array()) else {
        return Vec3::ZERO;
    };
    let c = LinearRgba::from(Srgba::rgb_u8(r, g, b));
    Vec3::new(c.red, c.green, c.blue)
}

/// The triangles of the far mesh: three corners each, and the linear colour and the area of each,
/// the area negative for a triangle of the flat sea.
struct Fine {
    points: Vec<[f32; 3]>,
    paint: Vec<(Vec3, f32)>,
}

impl Fine {
    /// Reads the far mesh `mesh` of a planet whose flat sea has the texture coordinates `sea`.
    fn read(mesh: &Mesh, sea: [f32; 2]) -> Self {
        let image = colormap::image();
        let points = match mesh.attribute(Mesh::ATTRIBUTE_POSITION) {
            Some(VertexAttributeValues::Float32x3(points)) => points.clone(),
            _ => Vec::new(),
        };
        let uvs = match mesh.attribute(Mesh::ATTRIBUTE_UV_0) {
            Some(VertexAttributeValues::Float32x2(uvs)) => uvs.clone(),
            _ => Vec::new(),
        };
        let corners = points.as_chunks::<3>().0.iter();
        let paint = corners.zip(uvs.iter().step_by(3)).map(|(corners, uv)| {
            let [a, b, c] = corners.map(Vec3::from);
            let area = (b - a).cross(c - a).length();
            (texel(&image, *uv), if *uv == sea { -area } else { area })
        });
        Self {
            paint: paint.collect(),
            points,
        }
    }

    /// Returns the four corners of the block of cells whose first cell is `(i, j)` of the node that
    /// starts at triangle `node`, and the mean colour of the block with the share of sea in alpha.
    fn block(&self, node: usize, (i, j): (usize, usize)) -> ([Vec3; 4], Vec4) {
        let tri = |i: usize, j: usize| node + (i * CELLS + j) * 2;
        let last = BLOCK - 1;
        let at = |t: usize, corner: usize| Vec3::from(self.points[t * 3 + corner]);
        let corners = [
            at(tri(i, j), 0),
            at(tri(i, j + last), 1),
            at(tri(i + last, j), 2),
            at(tri(i + last, j + last) + 1, 2),
        ];
        let cells = (0..BLOCK).flat_map(|a| (0..BLOCK).map(move |b| (a, b)));
        let tris = cells.flat_map(|(a, b)| [tri(i + a, j + b), tri(i + a, j + b) + 1]);
        let (mut sum, mut wet, mut total) = (Vec3::ZERO, 0.0, 0.0);
        for (colour, area) in tris.map(|t| self.paint[t]) {
            sum += colour * area.abs();
            wet += (-area).max(0.0);
            total += area.abs();
        }
        let total = total.max(f32::MIN_POSITIVE);
        (corners, (sum / total).extend(wet / total))
    }
}

/// The vertices of a ball being built.
#[derive(Default)]
struct Soup {
    points: Vec<[f32; 3]>,
    colours: Vec<[f32; 4]>,
    sum: Vec4,
    area: f32,
}

impl Soup {
    /// Adds the two triangles over the `corners` of a cell, shrunk by `shrink`, in `colour`.
    fn cell(&mut self, corners: [Vec3; 4], colour: Vec4, shrink: f32) {
        let [a, b, c, d] = corners.map(|p| p * shrink);
        let area = (b - a).cross(c - a).length() + (b - c).cross(d - c).length();
        self.points.extend([a, b, c, c, b, d].map(|p| p.to_array()));
        self.colours.extend([colour.to_array(); 6]);
        self.sum += colour * area;
        self.area += area;
    }

    /// Returns the mesh of the ball, with the normals of the smooth sphere.
    fn mesh(self) -> Mesh {
        let smooth = |p: &[f32; 3]| Vec3::from(*p).normalize_or_zero().to_array();
        let normals: Vec<[f32; 3]> = self.points.iter().map(smooth).collect();
        let count = self.points.len();
        Mesh::new(
            PrimitiveTopology::TriangleList,
            RenderAssetUsages::default(),
        )
        .with_inserted_attribute(Mesh::ATTRIBUTE_UV_0, vec![[0.0, 0.0]; count])
        .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, self.points)
        .with_inserted_attribute(Mesh::ATTRIBUTE_NORMAL, normals)
        .with_inserted_attribute(Mesh::ATTRIBUTE_COLOR, self.colours)
        .with_inserted_indices(Indices::U32((0..count as u32).collect()))
    }
}

/// Bakes the ball of `globe`: the far mesh of the world, each block of [`BLOCK`] by [`BLOCK`]
/// cells drawn as one cell in its mean colour, shrunk to a unit sphere. It reads only its
/// argument and is safe on any thread.
pub fn ball(globe: &Arc<Globe>) -> Ball {
    let [r, g, b] = globe.lands().sea_rgb();
    let fine = Fine::read(&bake(globe), colormap::uv(srgb(r, g, b)));
    let shrink = (1.0 / globe.radius()) as f32;
    let mut soup = Soup::default();
    let per_node = CELLS * CELLS * 2;
    let blocks = (0..CELLS / BLOCK).flat_map(|i| (0..CELLS / BLOCK).map(move |j| (i, j)));
    for node in (0..fine.paint.len() / per_node).map(|k| k * per_node) {
        for (i, j) in blocks.clone() {
            let (corners, colour) = fine.block(node, (i * BLOCK, j * BLOCK));
            soup.cell(corners, colour, shrink);
        }
    }
    let mean = soup.sum / soup.area.max(f32::MIN_POSITIVE);
    let glow = sea_glow(Some(globe.as_ref()));
    Ball {
        mean: mean.truncate(),
        sea: mean.w,
        glow: Vec3::new(glow.red, glow.green, glow.blue),
        mesh: soup.mesh(),
    }
}
