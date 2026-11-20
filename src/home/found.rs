//! Founding the base: from the globe of the home planet to its site and its named points.

use std::sync::Arc;

use bevy::math::DVec3;
use bevy::prelude::*;

use super::base::Base;
use super::container;
use super::ground::{harsh, starts};
use super::lot::Lot;
use super::site::{self, PASSES, Pass};
use super::tune::{CONTAINER, DRY, PAD, PAD_LIFT, SEARCH, WAKE};
use crate::universe::BodyId;
use crate::world::{Globe, Terrain};

impl Base {
    /// Returns the base of the home planet `body` whose ground is `globe`: near the spawn the terrain
    /// finds on the first shore of the drainage grid at a mild latitude that is not in the heart of a taiga
    /// or a desert, or on the next such shore when no ground near that spawn takes the base within any
    /// pass of the search. The shores in such a heart come last. The same globe always gives the same base.
    pub fn on(globe: &Arc<Globe>, body: BodyId) -> Self {
        let (found, first) = Self::choose(globe, |spawn| Self::within(globe, body, spawn));
        found.map(|(_, base)| base).unwrap_or_else(|| {
            let first = first.unwrap_or(DVec3::Y * globe.radius());
            let view = Self::around(globe, first).view().clone();
            Self::settled(&view, body, Vec2::X * SEARCH.0)
        })
    }

    /// Returns the planet position of the spawn the base of the home planet whose ground is `globe` is
    /// looked for around: where a new game stands while the base is found, which is where the base will
    /// stand unless no ground near any spawn takes one, and then the first spawn there is.
    pub fn landing(globe: &Arc<Globe>) -> DVec3 {
        let (found, first) = Self::choose(globe, |spawn| Self::sited(globe, spawn).then_some(()));
        let spawn = found.map(|(spawn, _)| spawn).or(first);
        spawn.unwrap_or(DVec3::Y * globe.radius())
    }

    /// Walks the spawns of the shores in the order of the base: the shores out of the heart of a taiga or a
    /// desert first, then the others. Returns the first spawn `found` takes with what it made of it, and
    /// the first spawn there is, which stands in when none is taken.
    fn choose<T>(
        globe: &Arc<Globe>,
        found: impl Fn(DVec3) -> Option<T>,
    ) -> (Option<(DVec3, T)>, Option<DVec3>) {
        let (mut first, mut hearts) = (None, Vec::new());
        let clear = starts(globe).find_map(|dir| {
            let (spawn, heart) = Self::lay(globe, dir)?;
            first.get_or_insert(spawn);
            if heart {
                hearts.push(spawn);
                return None;
            }
            found(spawn).map(|made| (spawn, made))
        });
        let heart = || hearts.iter().find_map(|s| found(*s).map(|made| (*s, made)));
        (clear.or_else(heart), first)
    }

    /// Returns true when some pass of the search finds a site for the base around `spawn`.
    fn sited(globe: &Arc<Globe>, spawn: DVec3) -> bool {
        let around = Self::around(globe, spawn);
        PASSES
            .iter()
            .any(|pass| site::find(around.view(), *pass).is_some())
    }

    /// Returns the spawn the terrain finds around the shore `dir` of `globe`, and whether it lies in the
    /// heart of a taiga or a desert, or nothing when no ground near the shore takes a spawn.
    fn lay(globe: &Arc<Globe>, dir: DVec3) -> Option<(DVec3, bool)> {
        let lot = Self::around(globe, dir);
        let view = lot.view();
        let at = view.spawn()?;
        (view.height(at.x, at.z) > globe.sea_level() + DRY).then(|| {
            (
                view.embed(at.x, at.y, at.z),
                harsh(view, Vec2::new(at.x, at.z)),
            )
        })
    }

    /// Returns the base around `spawn` in the first pass of the search that finds a site.
    fn within(globe: &Arc<Globe>, body: BodyId, spawn: DVec3) -> Option<Self> {
        PASSES
            .iter()
            .find_map(|pass| Self::near(globe, body, spawn, *pass))
    }

    /// Returns the base on the best site around the planet position `spawn` within `pass`, or nothing
    /// when no ground within reach takes it.
    pub fn near(globe: &Arc<Globe>, body: BodyId, spawn: DVec3, pass: Pass) -> Option<Self> {
        let around = Self::around(globe, spawn);
        let at = site::find(around.view(), pass)?;
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
