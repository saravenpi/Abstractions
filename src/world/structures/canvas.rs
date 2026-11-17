//! The mesh of a build: runs of triangles, each sealed with the way it is carried onto the planet, and the
//! run being drawn.

use std::ops::{Deref, DerefMut};

use bevy::prelude::*;

use crate::world::shapes::FlatMesh;

/// How a run of geometry is carried from the metric view it was drawn in onto the planet.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Hold {
    /// Moved as one rigid piece that stands upright at this point of the view.
    Rigid(Vec2),
    /// Every point keeps its place on the view and its height above sea level, so it follows the ground.
    Ground,
}

/// A sealed run: its triangles, how it is held, and how many parts and lights existed when it was sealed.
#[derive(Clone)]
pub struct Run {
    /// Triangles of the run, in the chunk space of the view.
    pub mesh: Mesh,
    /// How the run is carried onto the planet.
    pub hold: Hold,
    /// Number of solid parts drawn up to the end of the run.
    pub parts: usize,
    /// Number of lights placed up to the end of the run.
    pub lights: usize,
}

/// The triangles of a build. Drawing goes to the live run, which derefs to a [`FlatMesh`].
#[derive(Default, Clone)]
pub struct Canvas {
    runs: Vec<Run>,
    live: FlatMesh,
}

impl Deref for Canvas {
    type Target = FlatMesh;

    fn deref(&self) -> &FlatMesh {
        &self.live
    }
}

impl DerefMut for Canvas {
    fn deref_mut(&mut self) -> &mut FlatMesh {
        &mut self.live
    }
}

/// Joins triangle list meshes into one, in order.
pub fn join(meshes: impl IntoIterator<Item = Mesh>) -> Mesh {
    let mut all = meshes.into_iter();
    let mut out = all
        .next()
        .unwrap_or_else(|| FlatMesh::default().into_mesh());
    for mesh in all {
        let merged = out.merge(&mesh);
        debug_assert!(merged.is_ok(), "structure meshes share one layout");
    }
    out
}

impl Canvas {
    /// Closes the live run: what was drawn since the last seal is held by `hold`, with the solid parts and
    /// the lights added since, of which there are now `parts` and `lights`.
    pub fn seal(&mut self, hold: Hold, parts: usize, lights: usize) {
        let mesh = std::mem::take(&mut self.live).into_mesh();
        let last = self.runs.last().map_or((0, 0), |r| (r.parts, r.lights));
        if mesh.count_vertices() > 0 || last != (parts, lights) {
            self.runs.push(Run {
                mesh,
                hold,
                parts,
                lights,
            });
        }
    }

    /// Returns the sealed runs, in the order they were drawn. Live triangles are left out.
    pub fn into_runs(self) -> Vec<Run> {
        self.runs
    }

    /// Converts every triangle, sealed or live, into one mesh in the chunk space it was drawn in.
    #[cfg(test)]
    pub fn into_mesh(self) -> Mesh {
        let sealed = self.runs.into_iter().map(|r| r.mesh);
        join(sealed.chain([self.live.into_mesh()]))
    }

    /// Returns the number of vertices drawn so far.
    #[cfg(test)]
    pub fn count(&self) -> usize {
        let sealed: usize = self.runs.iter().map(|r| r.mesh.count_vertices()).sum();
        sealed + self.live.clone().into_mesh().count_vertices()
    }
}
