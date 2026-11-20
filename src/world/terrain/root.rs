//! Where a face view is rooted: the origin its chart positions are offsets from, the whole cells that origin
//! stands for, and the view every reader of one cell shares.

use std::sync::Arc;

use bevy::math::DVec3;
use bevy::prelude::*;

use super::Terrain;
use crate::world::globe::{BLOCK, Chart, Face, Globe};

/// The view a cell of a chart grid is worked out in, the same whatever view asked for it: rooted on the
/// block that holds the cell, so two readers of one cell always find the same thing there.
pub struct Home {
    /// The view rooted on the block of the cell. A chart without an origin is its own home.
    pub view: Terrain,
    /// The cell in the grid of that view.
    pub cell: IVec2,
    /// The cell counted from the middle of the chart: the same in every view of the face.
    pub id: IVec2,
    /// What to add to a chart position of the home view to get the same place in the view that asked.
    pub offset: Vec2,
}

impl Terrain {
    /// Returns the view of `globe` through the chart of the cube face that holds planet position `pos`,
    /// rooted on the block nearest to it: the chart positions around `pos` are small and precise.
    pub fn near(globe: &Arc<Globe>, pos: DVec3) -> Terrain {
        let dir = pos.normalize_or(DVec3::Y);
        let chart = globe.face_chart(Face::locate(dir).0).near(dir);
        Terrain::on(globe.clone(), chart)
    }

    /// Returns the origin of the chart in whole cells of `size` chart units: what to add to a cell of this
    /// view to get the cell counted from the middle of the chart. `size` must divide a block.
    pub fn shift(&self, size: f32) -> IVec2 {
        (self.chart.origin() / f64::from(size)).round().as_ivec2()
    }

    /// Returns the home of cell `cell` of the grid of `size` chart units of this view.
    pub fn home(&self, cell: IVec2, size: f32) -> Home {
        let id = cell + self.shift(size);
        let per = (BLOCK / f64::from(size)).round() as i32;
        let block = id.div_euclid(IVec2::splat(per));
        let origin = block.as_dvec2() * BLOCK;
        let faced = matches!(self.chart, Chart::Face { .. });
        Home {
            view: Terrain {
                chart: self.chart.rooted(origin),
                ..self.clone()
            },
            cell: if faced { id - block * per } else { cell },
            id,
            offset: (self.chart.rooted(origin).origin() - self.chart.origin()).as_vec2(),
        }
    }

    /// Returns this view rooted again near unit direction `dir`, once `dir` lies more than a block from
    /// the origin of its face chart, or nothing while it is nearer.
    pub fn followed(&self, dir: DVec3) -> Option<Terrain> {
        let far = self.chart.locate(dir).abs().max_element() > BLOCK as f32;
        let chart = self.chart.near(dir);
        (far && chart != self.chart).then(|| Terrain {
            chart,
            ..self.clone()
        })
    }
}
