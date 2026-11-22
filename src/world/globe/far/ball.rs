//! The ball: the surface of a planet the sky draws once the world has handed the planet over,
//! baked into one square map per face of the cube sphere.
//!
//! Every texel is one cell of the coarse nodes of the world, built through the same ground rules
//! and the same colour lookup, so a planet keeps its colours across the hand-over, coast for coast,
//! and the mean lean of the facets of that cell, so its mountains catch the light as the flat
//! shaded ground of the world does.
//! The finest map has the cells of the nodes of level 4, which the world draws just under the
//! hand-over. A face reads only its arguments and is safe on any thread.

use std::sync::Arc;

use bevy::math::DVec3;
use bevy::mesh::VertexAttributeValues;
use bevy::prelude::*;

use crate::world::Terrain;
use crate::world::colormap;
use crate::world::globe::{ChunkId, Face, Globe};
use crate::world::mesh::{Frame, fill_node};
use crate::world::sea::sea_glow;
use crate::world::shapes::{FlatMesh, srgb};

/// Texels along one side of a face map at each level a body is baked at: a dot, a disc, and a
/// disc filling the screen at the hand-over.
pub const SIDES: [usize; 3] = [64, 256, 1024];

/// Cells along one side of a node read into a map: those of the coarse nodes of the world.
pub const NODE_CELLS: usize = 64;

/// One face of the ball.
pub struct Map {
    /// Texels along one side.
    pub side: usize,
    /// The texels row by row from `v = -1`, each from `u = -1`: the linear colour of the cell and
    /// in w the share of it the flat sea covers.
    pub texels: Vec<Vec4>,
    /// The tilt of each texel in the same order: the mean normal of its facets less the mean
    /// direction up from the centre of the planet, both counted by area, in the planet frame,
    /// and 0 in w. Level ground and the flat sea have none.
    pub tilts: Vec<Vec4>,
    /// Sum over the face of the colour and the sea of every cell times its area in square metres.
    pub total: Vec4,
    /// Area of the face in square metres.
    pub area: f32,
}

/// What a planet shows as a dot.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Ball {
    /// Linear colour of the whole surface, each cell counted by its area.
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

/// The cells of a face being read: the colour lookup, the texture coordinates of the flat sea,
/// and for every texel the sum of colour and sea times area, of tilt times area, and the area.
struct Sums {
    face: Face,
    side: usize,
    image: Image,
    sea: [f32; 2],
    paint: Vec<Vec4>,
    tilt: Vec<Vec4>,
    area: Vec<f32>,
}

impl Sums {
    /// Starts reading `face` of `globe` into `side` by `side` texels.
    fn new(globe: &Globe, face: Face, side: usize) -> Self {
        let [r, g, b] = globe.lands().sea_rgb();
        Self {
            face,
            side,
            image: colormap::image(),
            sea: colormap::uv(srgb(r, g, b)),
            paint: vec![Vec4::ZERO; side * side],
            tilt: vec![Vec4::ZERO; side * side],
            area: vec![0.0; side * side],
        }
    }

    /// Returns the texel of the face that the direction `dir` falls in.
    fn index(&self, dir: DVec3) -> usize {
        let (u, v) = self.face.project(dir);
        let cell = |t: f64| (((t + 1.0) * 0.5 * self.side as f64) as usize).min(self.side - 1);
        cell(v) * self.side + cell(u)
    }

    /// Adds every triangle of the node mesh `mesh`, in the planet frame, to the texel its middle
    /// falls in, by its area: its colour, its sea and how far its normal leans from up.
    fn add(&mut self, mesh: &Mesh) {
        let (
            Some(VertexAttributeValues::Float32x3(points)),
            Some(VertexAttributeValues::Float32x2(uvs)),
        ) = (
            mesh.attribute(Mesh::ATTRIBUTE_POSITION),
            mesh.attribute(Mesh::ATTRIBUTE_UV_0),
        )
        else {
            return;
        };
        for (corners, uv) in points.as_chunks::<3>().0.iter().zip(uvs.iter().step_by(3)) {
            let [a, b, c] = corners.map(Vec3::from);
            let normal = (b - a).cross(c - a);
            let area = normal.length();
            let at = self.index((a + b + c).as_dvec3());
            let wet = f32::from(u8::from(*uv == self.sea));
            self.paint[at] += (texel(&self.image, *uv) * area).extend(wet * area);
            self.tilt[at] += (normal - (a + b + c).normalize_or_zero() * area).extend(0.0);
            self.area[at] += area;
        }
    }

    /// Returns the map: each texel its mean colour and tilt.
    fn finish(self) -> Map {
        let (total, area) = (
            self.paint.iter().sum::<Vec4>(),
            self.area.iter().sum::<f32>(),
        );
        Map {
            side: self.side,
            texels: mend(&self.paint, &self.area, self.side),
            tilts: mend(&self.tilt, &self.area, self.side),
            total,
            area,
        }
    }
}

/// Returns the mean of each texel of `sums`, a square of `side` texels whose areas are `area`,
/// and for a texel no cell fell in the mean of its neighbours that have one, or of the face.
fn mend(sums: &[Vec4], area: &[f32], side: usize) -> Vec<Vec4> {
    let whole = sums.iter().sum::<Vec4>() / area.iter().sum::<f32>().max(f32::MIN_POSITIVE);
    let wide = side as isize;
    let own = |i: isize, j: isize| {
        let inside = (0..wide).contains(&i) && (0..wide).contains(&j);
        let at = (j * wide + i) as usize;
        (inside && area[at] > 0.0).then(|| sums[at] / area[at])
    };
    let mean = |at: usize| {
        let (i, j) = ((at % side) as isize, (at / side) as isize);
        own(i, j).unwrap_or_else(|| {
            let around = [(i - 1, j), (i + 1, j), (i, j - 1), (i, j + 1)].map(|(i, j)| own(i, j));
            let (sum, count) = around
                .into_iter()
                .flatten()
                .fold((Vec4::ZERO, 0.0), |(s, n), t| (s + t, n + 1.0));
            if count > 0.0 { sum / count } else { whole }
        })
    };
    (0..sums.len()).map(mean).collect()
}

/// Bakes `face` of `globe` into a map of `side` texels a side: the nodes that tile the face with
/// [`NODE_CELLS`] cells each, or one node of `side` cells for a smaller map, one texel a cell.
pub fn face(globe: &Arc<Globe>, face: Face, side: usize) -> Map {
    let cells = side.min(NODE_CELLS);
    let nodes = (side / cells).max(1) as u32;
    let level = nodes.trailing_zeros() as u8;
    let terrain = Terrain::on(globe.clone(), globe.face_chart(face));
    let mut sums = Sums::new(globe, face, side);
    for (x, y) in (0..nodes).flat_map(|y| (0..nodes).map(move |x| (x, y))) {
        let id = ChunkId { face, level, x, y };
        let mut mesh = FlatMesh::default();
        fill_node(&mut mesh, &terrain, (id, cells), &Frame::PLANET, false);
        sums.add(&mesh.into_mesh());
    }
    sums.finish()
}

/// Returns what `globe` shows as a dot from the `maps` of its six faces.
pub fn ball(globe: &Globe, maps: &[Map]) -> Ball {
    let total = maps.iter().map(|map| map.total).sum::<Vec4>();
    let area = maps.iter().map(|map| map.area).sum::<f32>();
    let mean = total / area.max(f32::MIN_POSITIVE);
    let glow = sea_glow(Some(globe));
    Ball {
        mean: mean.truncate(),
        sea: mean.w,
        glow: Vec3::new(glow.red, glow.green, glow.blue),
    }
}
