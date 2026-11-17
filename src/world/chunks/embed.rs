//! Embedding: carries chunk geometry built in chart coordinates into the frame of the chunk on
//! the globe.

use avian3d::parry::math::Pose;
use avian3d::parry::shape::SharedShape;
use avian3d::prelude::*;
use bevy::mesh::VertexAttributeValues;
use bevy::prelude::*;

use crate::world::Terrain;
use crate::world::globe::Chart;
use crate::world::mesh::Frame;
use crate::world::scatter::chunk_center;

/// The one map from chunk space, where `(x, z)` are chart offsets from the chunk centre and `y`
/// is the height above sea level, to the tangent frame at the middle of the chunk.
///
/// Every mesh, collider and child transform a generator builds in chunk space goes through it.
pub struct Embedding {
    terrain: Terrain,
    center: Vec2,
    frame: Frame,
}

impl Embedding {
    /// Returns the embedding of chunk `coord` of the chart of `terrain`.
    pub fn of(terrain: &Terrain, coord: IVec2) -> Self {
        let center = chunk_center(coord);
        Self {
            terrain: terrain.clone(),
            center,
            frame: Frame::at(terrain, center),
        }
    }

    /// Returns the frame of the chunk: the tangent frame at its middle, on the sea level sphere.
    pub fn frame(&self) -> Frame {
        self.frame
    }

    /// Returns the chunk frame position of chunk space point `p`.
    pub fn point(&self, p: Vec3) -> Vec3 {
        let at = self.center + Vec2::new(p.x, p.z);
        self.frame.local(self.terrain.embed(at.x, p.y, at.y))
    }

    /// Returns the rotation that carries directions at chunk space point `p` into the chunk
    /// frame: local up stays up and chart x stays chart x.
    pub fn turn(&self, p: Vec3) -> Quat {
        let here = Frame::at(&self.terrain, self.center + Vec2::new(p.x, p.z));
        (self.frame.rot.inverse() * here.rot).normalize().as_quat()
    }

    /// Embeds a pose: its position, and its rotation turned the way the ground turns there.
    pub fn pose(&self, pos: Vec3, rot: Quat) -> (Vec3, Quat) {
        (self.point(pos), self.turn(pos) * rot)
    }

    /// Embeds the positions and normals of a triangle list, one rotation per triangle.
    pub fn mesh(&self, mesh: &mut Mesh) {
        if matches!(self.terrain.chart(), Chart::Flat) {
            return;
        }
        let Some(VertexAttributeValues::Float32x3(positions)) =
            mesh.attribute_mut(Mesh::ATTRIBUTE_POSITION)
        else {
            return;
        };
        let turns: Vec<Quat> = positions
            .chunks(3)
            .map(|tri| self.turn(Vec3::from(tri[0])))
            .collect();
        for p in positions.iter_mut() {
            *p = self.point(Vec3::from(*p)).to_array();
        }
        if let Some(VertexAttributeValues::Float32x3(normals)) =
            mesh.attribute_mut(Mesh::ATTRIBUTE_NORMAL)
        {
            for (k, n) in normals.iter_mut().enumerate() {
                *n = (turns[k / 3] * Vec3::from(*n)).to_array();
            }
        }
    }

    /// Returns `collider` with every part of a compound moved and turned onto the globe. Other
    /// shapes come back unchanged.
    pub fn collider(&self, collider: &Collider) -> Collider {
        let Some(compound) = collider.shape().as_compound() else {
            return collider.clone();
        };
        let part = |(pose, shape): &(Pose, SharedShape)| {
            let (t, r) = (pose.translation, pose.rotation);
            let at = Vec3::new(t.x, t.y, t.z);
            let (pos, rot) = self.pose(at, Quat::from_xyzw(r.x, r.y, r.z, r.w));
            (pos, rot, Collider::from(shape.clone()))
        };
        Collider::compound(compound.shapes().iter().map(part).collect())
    }
}
