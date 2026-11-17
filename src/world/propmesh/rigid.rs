//! Merging props rigidly into the frame of a chunk on the globe: every prop keeps its true shape and size.

use bevy::prelude::*;

use super::merge::Soup;
use crate::world::scatter::{Placement, Rig};
use crate::world::shapes::FlatMesh;

impl Soup {
    /// Appends geometry built in chunk space around `origin`, carried as one rigid piece into the frame of `rig`.
    fn graft(&mut self, part: FlatMesh, origin: Vec3, rig: &Rig) {
        let part = Soup::of(&part.into_mesh());
        let (to, turn) = rig.frame(origin);
        let shift = to - turn * origin;
        let moved = |v: &[f32; 3]| (turn * Vec3::from(*v) + shift).to_array();
        self.positions.extend(part.positions.iter().map(moved));
        let turned = |n: &[f32; 3]| (turn * Vec3::from(*n)).to_array();
        self.normals.extend(part.normals.iter().map(turned));
        self.uvs.extend(part.uvs);
    }

    /// Appends every solid prop, each standing along the local up of `rig` at its origin.
    pub(super) fn stand(&mut self, props: &[Placement], rig: &Rig) {
        for p in props {
            let (origin, rot) = rig.pose(p);
            self.push(p, origin, rot);
        }
    }
}

/// Merges loose parts and solid props into one mesh in the frame of `rig`, parts first.
///
/// Each part is geometry built in chunk space around the origin it comes with, and is carried as one rigid
/// piece: heights above its origin become distances along the local up there.
pub fn merge_parts_on(
    parts: impl Iterator<Item = (Vec3, FlatMesh)>,
    solids: &[Placement],
    rig: &Rig,
) -> Mesh {
    let mut soup = Soup::default();
    for (origin, part) in parts {
        soup.graft(part, origin, rig);
    }
    soup.stand(solids, rig);
    soup.into_mesh()
}
