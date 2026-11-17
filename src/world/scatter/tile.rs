//! Test fixture: one chunk of a globe with the frame its geometry lives in, embedded the way the chunk builder does.

use std::sync::Arc;

use bevy::math::DVec3;
use bevy::prelude::*;

use super::{Placement, chunk_center, scatter_chunk};
use crate::world::globe::{Face, Globe};
use crate::world::mesh::Frame;
use crate::world::solids::tests::triangles;
use crate::world::{CHUNK_SIZE, Terrain};

/// One chunk of a face of a globe, with the tangent frame at its middle that a chunk stores its geometry in.
pub struct Tile {
    /// The face the chunk lies on.
    pub face: Face,
    /// The view of the face of the chunk.
    pub terrain: Terrain,
    /// The chunk coordinate in the chart of the face.
    pub coord: IVec2,
    /// The frame of the chunk.
    pub frame: Frame,
}

impl Tile {
    /// Returns chunk `coord` of `face` of `globe`.
    pub fn new(globe: &Arc<Globe>, face: Face, coord: IVec2) -> Self {
        let terrain = Terrain::on(globe.clone(), globe.face_chart(face));
        let frame = Frame::at(&terrain, chunk_center(coord));
        Self {
            face,
            terrain,
            coord,
            frame,
        }
    }

    /// Returns the position, in metres from the planet centre, of chunk space point `p`.
    pub fn planet(&self, p: Vec3) -> DVec3 {
        let at = chunk_center(self.coord) + p.xz();
        self.terrain.embed(at.x, p.y, at.y)
    }

    /// Returns the chunk frame position of chunk space point `p`, the way the chunk builder embeds it.
    pub fn point(&self, p: Vec3) -> Vec3 {
        self.frame.local(self.planet(p))
    }

    /// Returns the rotation that carries directions at chunk space point `p` into the chunk frame.
    pub fn turn(&self, p: Vec3) -> Quat {
        let here = Frame::at(&self.terrain, chunk_center(self.coord) + p.xz());
        (self.frame.rot.inverse() * here.rot).normalize().as_quat()
    }

    /// Returns the true area of the chunk in square metres, from its four corners on the sea level sphere.
    pub fn area(&self) -> f64 {
        let half = CHUNK_SIZE * 0.5;
        let at = |x: f32, z: f32| self.planet(Vec3::new(x * half, 0.0, z * half));
        let [a, b, c, d] = [at(-1.0, -1.0), at(1.0, -1.0), at(1.0, 1.0), at(-1.0, 1.0)];
        ((b - a).cross(c - a).length() + (c - a).cross(d - a).length()) * 0.5
    }

    /// Returns the scatter of the chunk with the planet position of each prop, big props first.
    pub fn scatter(&self) -> Vec<(Placement, DVec3)> {
        let s = scatter_chunk(&self.terrain, self.coord);
        let all = s.props.into_iter().chain(s.grass);
        all.map(|p| (p, self.planet(p.pos))).collect()
    }

    /// Returns how many of 25 points spread over the chunk are land.
    pub fn dry(&self) -> usize {
        let grid = (0..25).map(|i| Vec2::new((i % 5) as f32 - 2.0, (i / 5) as f32 - 2.0));
        grid.map(|g| chunk_center(self.coord) + g * CHUNK_SIZE * 0.24)
            .filter(|p| self.terrain.height(p.x, p.y) > 0.5)
            .count()
    }
}

/// Returns the lowest and highest reach along `up` and the widest reach across it of the vertices of `mesh`,
/// seen from `origin`: where a prop starts, where it ends and how wide it is.
pub fn extent(mesh: &Mesh, origin: Vec3, up: Vec3) -> [f32; 3] {
    let points = triangles(mesh).into_iter().flatten();
    points.fold([f32::MAX, f32::MIN, 0.0], |[low, high, wide], v| {
        let along = (v - origin).dot(up);
        let across = (v - origin - up * along).length();
        [low.min(along), high.max(along), wide.max(across)]
    })
}
