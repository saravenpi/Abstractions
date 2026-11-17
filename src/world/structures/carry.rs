//! Carrying finished site geometry from the metric view it was drawn in onto the planet: buildings as rigid
//! pieces upright at their own origin, ground pieces point by point.

use bevy::math::{DQuat, DVec3};
use bevy::mesh::VertexAttributeValues;
use bevy::prelude::*;

use super::brush::Part;
use super::canvas::Hold;
#[cfg(test)]
use super::geo::chart_point;
use super::glow::LightPoint;
use crate::world::Terrain;
use crate::world::mesh::Frame;

/// Where finished geometry goes.
#[derive(Clone)]
pub enum Target {
    /// The chunk space of a chart: `(x, z)` are chart offsets from `center` and `y` is the height above sea
    /// level. The tests warp this space onto the globe point by point to compare it with [`Target::Frame`].
    #[cfg(test)]
    Chart {
        /// The chart the chunk belongs to.
        terrain: Terrain,
        /// Chart position of the middle of the chunk.
        center: Vec2,
    },
    /// A rigid frame fixed to the planet, such as the frame of a chunk.
    Frame(Frame),
}

impl Target {
    /// Returns where the planet position `planet` lies in the target.
    fn place(&self, planet: DVec3) -> Vec3 {
        match self {
            #[cfg(test)]
            Target::Chart { terrain, center } => {
                let flat = (chart_point(terrain, planet) - center.as_dvec2()).as_vec2();
                Vec3::new(flat.x, terrain.locate(planet).y, flat.y)
            }
            Target::Frame(frame) => frame.local(planet),
        }
    }

    /// Returns the rotation from planet axes to the axes of the target at its point `_at`, which only a chart
    /// target depends on.
    fn axes(&self, _at: Vec3) -> DQuat {
        match self {
            #[cfg(test)]
            Target::Chart { terrain, center } => {
                Frame::at(terrain, *center + _at.xz()).rot.inverse()
            }
            Target::Frame(frame) => frame.rot.inverse(),
        }
    }
}

/// The way one run of a build goes from the chunk space of its view to a target.
pub struct Carry<'a> {
    view: &'a Terrain,
    center: Vec2,
    anchor: Option<(Vec2, Frame)>,
    target: Target,
}

impl<'a> Carry<'a> {
    /// Returns the way to `target` of a run held by `hold`, drawn on `view` around the chart point `center`.
    pub fn new(view: &'a Terrain, center: Vec2, hold: Hold, target: Target) -> Self {
        let anchor = match hold {
            Hold::Rigid(at) => Some((at, Frame::at(view, at))),
            Hold::Ground => None,
        };
        Self {
            view,
            center,
            anchor,
            target,
        }
    }

    /// Returns the planet position of the chunk space point `p`, and the rotation from the axes of the view
    /// there to planet axes.
    fn lift(&self, p: Vec3) -> (DVec3, DQuat) {
        match &self.anchor {
            Some((at, frame)) => {
                let flat = (self.center - *at).as_dvec2() + p.xz().as_dvec2();
                let local = DVec3::new(flat.x, f64::from(p.y), flat.y);
                (frame.origin + frame.rot * local, frame.rot)
            }
            None => {
                let at = self.center + p.xz();
                (
                    self.view.embed(at.x, p.y, at.y),
                    Frame::at(self.view, at).rot,
                )
            }
        }
    }

    /// Returns where the chunk space point `p` goes.
    pub fn point(&self, p: Vec3) -> Vec3 {
        self.target.place(self.lift(p).0)
    }

    /// Returns where the chunk space point `p` goes and how directions turn there.
    pub fn pose(&self, p: Vec3) -> (Vec3, Quat) {
        let (planet, turn) = self.lift(p);
        let at = self.target.place(planet);
        (at, (self.target.axes(at) * turn).normalize().as_quat())
    }

    /// Carries the positions and normals of a triangle list, one rotation per triangle.
    pub fn mesh(&self, mesh: &mut Mesh) {
        let Some(VertexAttributeValues::Float32x3(positions)) =
            mesh.attribute_mut(Mesh::ATTRIBUTE_POSITION)
        else {
            return;
        };
        let turns: Vec<Quat> = positions
            .chunks(3)
            .map(|t| self.pose(Vec3::from(t[0])).1)
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

    /// Carries solid parts and lights: each moves to its place and turns with the ground there.
    pub fn solids(&self, parts: &mut [Part], lights: &mut [LightPoint]) {
        for part in parts {
            let (pos, turn) = self.pose(part.pos);
            (part.pos, part.rot) = (pos, turn * part.rot);
        }
        for light in lights {
            (light.pos, light.rot) = self.pose(light.pos);
        }
    }
}
