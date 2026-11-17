//! The point by point warp of chart geometry onto the globe, kept for the tests that compare it with the
//! rigid placement the game uses.

use avian3d::parry::math::Pose;
use avian3d::parry::shape::SharedShape;
use avian3d::prelude::*;
use bevy::mesh::VertexAttributeValues;
use bevy::prelude::*;

use super::Embedding;
use crate::world::globe::Chart;

impl Embedding {
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
