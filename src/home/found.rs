//! Founding the base: from the globe of the home planet to its site and its named points.

use std::sync::Arc;

use bevy::math::DVec3;
use bevy::prelude::*;

use super::base::Base;
use super::container;
use super::lot::Lot;
use super::site;
use super::tune::{CONTAINER, PAD, PAD_LIFT, SEARCH, WAKE};
use crate::universe::BodyId;
use crate::world::globe::Face;
use crate::world::{Globe, Terrain};

impl Base {
    /// Returns the base of the home planet `body` whose ground is `globe`: near the spawn the
    /// terrain finds for a new game. The same globe always gives the same base.
    pub fn on(globe: &Arc<Globe>, body: BodyId) -> Self {
        let north = Terrain::on(globe.clone(), globe.face_chart(Face::PosY));
        let at = north.find_spawn();
        Self::near(globe, body, north.embed(at.x, at.y, at.z))
    }

    /// Returns the base on the best site around the planet position `spawn`.
    pub fn near(globe: &Arc<Globe>, body: BodyId, spawn: DVec3) -> Self {
        let dir = spawn.normalize();
        let around = Lot::new(globe, dir, dir.any_orthonormal_vector());
        let view = around.view();
        let at = site::find(view).unwrap_or(Vec2::X * SEARCH.0);
        let lot = Lot::new(globe, view.up(at.x, at.y), view.chart().east(at.x, at.y));
        Self::laid(&lot, body)
    }

    /// Returns the lot of the base on `globe`, which must be the ground of its planet.
    pub(super) fn lot(&self, globe: &Arc<Globe>) -> Lot {
        Lot::new(globe, self.up, self.east)
    }

    /// Returns the base laid out on `lot`.
    fn laid(lot: &Lot, body: BodyId) -> Self {
        let axes = lot.axes();
        let grounded =
            |at: Vec2, lift: f32| lot.planet(Vec3::new(at.x, lot.ground(at.x, at.y) + lift, at.y));
        let heading = Quat::from_rotation_y(PAD.yaw) * Vec3::NEG_Z;
        Self {
            body,
            centre: lot.planet(Vec3::ZERO),
            up: axes.y_axis,
            east: axes.x_axis,
            north: -axes.z_axis,
            wake: grounded(CONTAINER.point(WAKE), 0.0),
            pad: grounded(PAD.at, PAD_LIFT),
            heading: axes * heading.as_dvec3(),
            door: lot.planet(container::pose(lot).transform_point3(container::DOOR)),
        }
    }
}
