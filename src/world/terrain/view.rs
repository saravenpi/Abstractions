//! The terrain as a view of a globe: its chart, and the way in and out of planet space.

use std::sync::Arc;

use bevy::math::DVec3;
use bevy::prelude::*;

use super::Terrain;
use crate::world::globe::{Chart, Globe, Grain, Tangent};
use crate::world::{SEA_LEVEL, Surface};

impl Terrain {
    /// Returns the planet this terrain is a view of.
    pub fn globe(&self) -> &Arc<Globe> {
        &self.globe
    }

    /// Returns the chart that maps `(x, z)` onto the globe.
    pub fn chart(&self) -> Chart {
        self.chart
    }

    /// Returns a metric view centred on chart position `(x, z)`, which keeps its coordinates
    /// there and its x and z directions. The flat terrain is already metric and returns itself.
    pub fn around(&self, x: f32, z: f32) -> Terrain {
        if matches!(self.chart, Chart::Flat) {
            return self.clone();
        }
        let (origin, east) = (self.chart.dir(x, z), self.chart.east(x, z));
        let offset = Vec2::new(x, z).as_dvec2();
        let tangent = Tangent::new(origin, east, self.globe.radius(), offset);
        Terrain::on(self.globe.clone(), Chart::Tangent(tangent))
    }

    /// Returns the position in metres from the planet centre of the point `y` metres above sea
    /// level at chart position `(x, z)`. On the flat terrain this is `(x, y, z)` itself.
    pub fn embed(&self, x: f32, y: f32, z: f32) -> DVec3 {
        match self.chart {
            Chart::Flat => Vec3::new(x, y, z).as_dvec3(),
            chart => chart.dir(x, z) * (self.globe.radius() + f64::from(y)),
        }
    }

    /// Returns the surface a channel bed puts at chart position `(x, z)`: a river bed, or the rock of a lava
    /// flow, wherever a flow runs on the globe, and nothing elsewhere or on the legacy plane, which has none.
    /// The terrain meshes paint the bed with this rule, and the placers keep out of a river with it. A view
    /// too coarse to show a river has none.
    pub fn bed(&self, x: f32, z: f32) -> Option<Surface> {
        match self.chart {
            chart if chart != Chart::Flat && self.grain == Grain::Fine => {
                self.globe.mark(chart.dir(x, z))
            }
            _ => None,
        }
    }

    /// Returns true when a river bed lies at chart position `(x, z)` above the sea, where the meshes paint
    /// it: shallow water to anything that would stand, grow or be built there. Where a river meets the sea
    /// its bed is sea floor.
    pub fn river_at(&self, x: f32, z: f32) -> bool {
        self.bed(x, z) == Some(Surface::River) && self.height(x, z) > SEA_LEVEL
    }

    /// Returns the unit direction away from the planet centre at chart position `(x, z)`.
    pub fn up(&self, x: f32, z: f32) -> DVec3 {
        self.chart.dir(x, z)
    }

    /// Returns the chart position `(x, z)` and the height `y` above sea level of a position
    /// given in metres from the planet centre: the inverse of [`Terrain::embed`].
    pub fn locate(&self, pos: DVec3) -> Vec3 {
        if matches!(self.chart, Chart::Flat) {
            return pos.as_vec3();
        }
        let at = self.chart.locate(pos.normalize());
        Vec3::new(at.x, (pos.length() - self.globe.radius()) as f32, at.y)
    }
}
