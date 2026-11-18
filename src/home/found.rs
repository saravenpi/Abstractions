//! Founding the base: from the globe of the home planet to its site and its named points.

use std::sync::Arc;

use bevy::math::DVec3;
use bevy::prelude::*;

use super::base::Base;
use super::container;
use super::ground::starts;
use super::lot::Lot;
use super::site::{self, SLACK};
use super::tune::{CONTAINER, PAD, PAD_LIFT, SEARCH, WAKE};
use crate::universe::BodyId;
use crate::world::{Globe, Terrain};

impl Base {
    /// Returns the base of the home planet `body` whose ground is `globe`: near the spawn the
    /// terrain finds around a mild northern latitude, at a longitude the seed picks, or around the
    /// next longitude when no ground near that spawn takes the base. Every spawn is tried within
    /// the relief limits before any is tried with the looser ones. The same globe always gives the
    /// same base.
    pub fn on(globe: &Arc<Globe>, body: BodyId) -> Self {
        let mut spawns = Vec::new();
        let strict = starts(globe.seed()).find_map(|dir| {
            let spawn = spawn_of(Self::around(globe, dir).view());
            spawns.push(spawn);
            Self::near(globe, body, spawn, SLACK[0])
        });
        let loose = || {
            let mut each = spawns.iter();
            each.find_map(|spawn| Self::near(globe, body, *spawn, SLACK[1]))
        };
        strict.or_else(loose).unwrap_or_else(|| {
            let first = spawns.first().copied().unwrap_or(DVec3::Y * globe.radius());
            let view = Self::around(globe, first).view().clone();
            Self::settled(&view, body, Vec2::X * SEARCH.0)
        })
    }

    /// Returns the planet position of the first spawn the base is looked for around on `globe`, the ground
    /// of its home planet: where a new game stands while the base is found, near where it will stand.
    pub fn landing(globe: &Arc<Globe>) -> DVec3 {
        let first = starts(globe.seed()).next().unwrap_or(DVec3::Y);
        spawn_of(Self::around(globe, first).view())
    }

    /// Returns the base on the best site around the planet position `spawn` within `slack` times
    /// the relief limits, or nothing when no ground within reach takes it.
    pub fn near(globe: &Arc<Globe>, body: BodyId, spawn: DVec3, slack: f32) -> Option<Self> {
        let around = Self::around(globe, spawn);
        let at = site::find(around.view(), slack)?;
        Some(Self::settled(around.view(), body, at))
    }

    /// Returns the lot around the planet position `spawn`, with any east, in which the site is
    /// looked for.
    fn around(globe: &Arc<Globe>, spawn: DVec3) -> Lot {
        let dir = spawn.normalize();
        Lot::new(globe, dir, dir.any_orthonormal_vector())
    }

    /// Returns the base laid out at the point `at` of the metric `view` it was looked for in.
    fn settled(view: &Terrain, body: BodyId, at: Vec2) -> Self {
        let (up, east) = (view.up(at.x, at.y), view.chart().east(at.x, at.y));
        Self::laid(&Lot::new(view.globe(), up, east), body)
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

/// Returns the planet position of the spawn the terrain finds around the origin of `view`.
fn spawn_of(view: &Terrain) -> DVec3 {
    let at = view.find_spawn();
    view.embed(at.x, at.y, at.z)
}
