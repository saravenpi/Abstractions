//! The ground the planet gives where no collider of the world is loaded yet: the ground the
//! terrain mesh draws. A body over it stands, walks and lands on it as on a collider, so a glide
//! that outruns the streaming of the detail chunks never passes under the ground it sees.

use avian3d::prelude::*;
use bevy::prelude::*;

use super::floor::Floor;
use super::ground::{Ground, SNAP_DISTANCE};
use super::{BODY_LENGTH, BODY_RADIUS};
use crate::space::{Anchor, up};
use crate::world::Terrain;

/// Metres under the bare ground beyond which a body is not landing on it but lost under it: the
/// rescue puts that one back, see `lost`. A collider stops what crosses it, not what is inside.
const DEEP: f32 = 0.5;

/// The planet's own ground, read where no fixed collider is loaded.
pub(super) struct Bare<'a, 'w, 's> {
    /// The render anchor.
    pub(super) anchor: &'a Anchor,
    /// The terrain of the planet.
    pub(super) terrain: &'a Terrain,
    /// The fixed colliders of the world.
    pub(super) floor: &'a Floor<'w, 's>,
    /// Whether the world streams its ground in chunks, whose colliders may lag the body. A
    /// world built by hand, as in a test, has its colliders where it wants them and no bare
    /// ground.
    pub(super) streamed: bool,
}

impl Bare<'_, '_, '_> {
    /// Returns how far the soles of a body centred at render position `at` are above the ground
    /// the terrain mesh draws there, negative under it, when no fixed collider is loaded under
    /// it. Nothing where one is, which the body stands on instead, or on a world with no ground.
    pub(super) fn gap(&self, at: Vec3) -> Option<f32> {
        let globe = self.terrain.globe();
        let loaded = self.floor.under(at, up(self.anchor, at)).is_some();
        if !self.streamed || !globe.has_ground() || loaded {
            return None;
        }
        let chart = self.terrain.locate(self.anchor.to_frame(at));
        let soles = chart.y - BODY_LENGTH * 0.5 - BODY_RADIUS;
        Some(soles - self.terrain.mesh_height(chart.x, chart.z))
    }

    /// Returns the bare ground under a body centred at `at` as the probe finds a collider: flat
    /// along the local up, within the reach of the snap.
    pub(super) fn ground(&self, at: Vec3) -> Option<Ground> {
        let gap = self
            .gap(at)
            .filter(|gap| (-DEEP..=SNAP_DISTANCE).contains(gap))?;
        Some(Ground {
            normal: Vec3::Y,
            gap: gap.max(0.0),
        })
    }

    /// Keeps a body that move and slide moved from `from` to `out` on the bare ground: one that
    /// started on or over it and would end under it is lifted back onto it and loses the part of
    /// its velocity that went into it, as a collider would take it. Whatever its speed, it lands
    /// on the surface and never passes under it.
    pub(super) fn keep(&self, from: Vec3, out: &mut MoveAndSlideOutput) {
        let over = self.gap(from).is_some_and(|gap| gap >= -DEEP);
        let Some(gap) = self.gap(out.position).filter(|gap| over && *gap < 0.0) else {
            return;
        };
        let vertical = up(self.anchor, out.position);
        out.position -= vertical * gap;
        let into = out.projected_velocity.dot(vertical).min(0.0);
        out.projected_velocity -= vertical * into;
    }
}
