//! How a [`Build`] hands its work over: runs held rigid or to the ground, the visible ground height on any
//! chart, and the finished output carried to a target.

use bevy::prelude::*;

use super::brush::{Build, Part};
use super::canvas::{Hold, join};
use super::carry::{Carry, Target};
use super::geo::transfer;
use super::glow::LightPoint;
#[cfg(test)]
use crate::world::globe::Chart;

/// What one site adds to its chunk: triangles, solid parts and lights, all in the space of the target.
pub struct Output {
    /// Merged triangles.
    pub mesh: Mesh,
    /// Solid parts.
    pub parts: Vec<Part>,
    /// Lights.
    pub lights: Vec<LightPoint>,
}

impl<'a> Build<'a> {
    /// Closes the run being drawn under the current hold.
    pub fn seal(&mut self) {
        self.mesh
            .seal(self.hold, self.parts.len(), self.lights.len());
    }

    /// Makes what is drawn next follow the ground point by point: streets, paths, crop rows and fences.
    pub fn hug(&mut self) {
        self.seal();
        self.hold = Hold::Ground;
    }

    /// Makes what is drawn next one rigid piece, upright at the origin of the current site.
    pub fn stiffen(&mut self) {
        self.seal();
        self.hold = Hold::Rigid(self.site.origin);
    }

    /// Returns the height the terrain mesh draws at the point `p` of the view of the site: the ground that
    /// thin pieces must touch. The mesh belongs to the chart of the chunk, not to the view.
    pub fn mesh_y(&self, p: Vec2) -> f32 {
        let at = transfer(self.terrain, self.ground, p);
        self.ground.mesh_height(at.x, at.y)
    }

    /// Ends the build and returns its output carried to `target`. On the flat chart, chunk space already is
    /// the target and nothing moves.
    pub fn finish(mut self, target: Target) -> Output {
        self.seal();
        #[cfg(test)]
        let same = matches!(
            (self.terrain.chart(), &target),
            (Chart::Flat, Target::Chart { .. })
        );
        #[cfg(not(test))]
        let same = false;
        let (mut parts, mut lights) = (self.parts, self.lights);
        let (mut from, mut meshes) = ((0, 0), Vec::new());
        for mut run in self.mesh.into_runs() {
            if !same {
                let carry = Carry::new(self.terrain, self.center, run.hold, target.clone());
                carry.mesh(&mut run.mesh);
                carry.solids(
                    &mut parts[from.0..run.parts],
                    &mut lights[from.1..run.lights],
                );
            }
            from = (run.parts, run.lights);
            meshes.push(run.mesh);
        }
        Output {
            mesh: join(meshes),
            parts,
            lights,
        }
    }
}
