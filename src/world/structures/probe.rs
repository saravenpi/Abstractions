//! Test-only view of a drawn build in the frame of one building: its triangles, boxes and tagged pieces.

use bevy::prelude::*;

use super::audit::{Piece, positions};
use super::brush::Build;
use super::tags::Tag;

/// An axis-aligned box as its low and high corners.
pub type Bx = (Vec3, Vec3);

/// The triangles of a build in the frame of one building, with the pieces and roles recorded while drawing.
pub struct Shot {
    /// Triangle list in the building frame: x along the ridge, y up from the floor, z toward the door.
    pub verts: Vec<Vec3>,
    /// Closed boxes and sheets as vertex ranges.
    pub audit: Vec<(Piece, usize, usize)>,
    /// Roles as vertex ranges.
    pub tags: Vec<(Tag, usize, usize)>,
}

impl Shot {
    /// Reads `b`, whose building stands at the chunk origin turned by `yaw` with its floor at height `floor`.
    pub fn of(b: &Build, yaw: f32, floor: f32) -> Self {
        let back = Quat::from_rotation_y(-yaw);
        let verts = positions(&b.mesh.clone().into_mesh());
        Self {
            verts: verts
                .into_iter()
                .map(|p| back * (p - Vec3::Y * floor))
                .collect(),
            audit: b.audit.clone(),
            tags: b.tags.clone(),
        }
    }

    /// Returns the box around the vertices `start..end`.
    pub fn bounds(&self, start: usize, end: usize) -> Bx {
        let part = &self.verts[start..end];
        let lo = part.iter().fold(Vec3::MAX, |m, p| m.min(*p));
        (lo, part.iter().fold(Vec3::MIN, |m, p| m.max(*p)))
    }

    /// Returns the vertex ranges recorded under `tag`.
    pub fn ranges(&self, tag: Tag) -> Vec<(usize, usize)> {
        let of = self.tags.iter().filter(|t| t.0 == tag);
        of.map(|t| (t.1, t.2)).collect()
    }

    /// Returns true when the vertex `at` was drawn under `tag`.
    pub fn is(&self, tag: Tag, at: usize) -> bool {
        let mut of = self.tags.iter().filter(|t| t.0 == tag);
        of.any(|t| (t.1..t.2).contains(&at))
    }

    /// Returns every closed box, with the index of its first vertex.
    pub fn closed(&self) -> Vec<(usize, Bx)> {
        let boxes = self.audit.iter().filter(|a| a.0 == Piece::Closed);
        boxes.map(|a| (a.1, self.bounds(a.1, a.2))).collect()
    }

    /// Returns the closed boxes drawn under `tag`.
    pub fn boxes(&self, tag: Tag) -> Vec<Bx> {
        let all = self.closed().into_iter();
        all.filter(|(at, _)| self.is(tag, *at))
            .map(|(_, bx)| bx)
            .collect()
    }

    /// Returns the triangles whose first vertex index satisfies `keep`.
    pub fn tris(&self, keep: impl Fn(usize) -> bool) -> Vec<[Vec3; 3]> {
        let all = self.verts.as_chunks::<3>().0.iter().enumerate();
        all.filter(|(i, _)| keep(i * 3)).map(|(_, t)| *t).collect()
    }
}
