//! Choosing the spawn the base is founded around, and the spawn a new game lands at.

use std::sync::Arc;

use bevy::math::DVec3;
use bevy::prelude::*;

use super::base::Base;
use super::ground::{harsh, starts};
use super::site::{self, PASSES};
use super::tune::DRY;
use crate::universe::BodyId;
use crate::world::Globe;

impl Base {
    /// Walks the spawns of the shores in the order of the base: the shores out of the heart of a taiga or a
    /// desert first, then the others. Returns the first spawn `found` takes with what it made of it, and
    /// the first spawn there is, which stands in when none is taken.
    pub(super) fn choose<T>(
        globe: &Arc<Globe>,
        found: impl Fn(DVec3) -> Option<T>,
    ) -> (Option<(DVec3, T)>, Option<DVec3>) {
        let (mut first, mut hearts) = (None, Vec::new());
        let clear = starts(globe).into_iter().find_map(|dir| {
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
    pub(super) fn sited(globe: &Arc<Globe>, spawn: DVec3) -> bool {
        let around = Self::around(globe, spawn);
        PASSES
            .iter()
            .any(|pass| site::find(around.view(), *pass).is_some())
    }

    /// Returns the spawn the terrain finds around the shore `dir` of `globe`, and whether it lies in the
    /// heart of a taiga or a desert, or nothing when no ground near the shore takes a spawn.
    pub(super) fn lay(globe: &Arc<Globe>, dir: DVec3) -> Option<(DVec3, bool)> {
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
    pub(super) fn within(globe: &Arc<Globe>, body: BodyId, spawn: DVec3) -> Option<Self> {
        PASSES
            .iter()
            .find_map(|pass| Self::near(globe, body, spawn, *pass))
    }
}
