//! The ground the planet gives where no collider of the world is loaded yet: the triangles the
//! terrain is drawn with there. A body over it stands, walks and lands on it as on a collider, so
//! a glide that outruns the streaming of the detail chunks never passes under the ground it sees.

use avian3d::prelude::*;
use bevy::prelude::*;

use super::floor::Floor;
use super::ground::{Ground, SNAP_DISTANCE};
use super::{BODY_LENGTH, BODY_RADIUS};
use crate::space::{Anchor, up};
use crate::world::{Chunks, Drawn, Terrain};

/// Metres under the bare ground beyond which a body is not landing on it but lost under it: the
/// rescue puts that one back, see `lost`. A collider stops what crosses it, not what is inside.
const DEEP: f32 = 0.5;

/// Metres the bare ground may lift a body per metre it moved along the ground in a step, the
/// rise of the steepest walkable slope, about 50 degrees; and the metres it may lift it in any
/// step. It only ever gives back what the body moved into it: it never throws it up.
const LIFT: (f32, f32) = (1.2, 0.02);

/// Returns the most the bare ground may lift a body that moved `fell` metres down and `along`
/// metres along the ground in a step, see [`LIFT`].
pub(super) fn most_lift(fell: f32, along: f32) -> f32 {
    fell.max(0.0) + LIFT.0 * along.max(0.0) + LIFT.1
}

/// The planet's own ground, read where no fixed collider is loaded around a body.
pub(super) struct Bare<'a, 'w, 's> {
    /// The render anchor.
    pub(super) anchor: &'a Anchor,
    /// The terrain of the planet.
    pub(super) terrain: &'a Terrain,
    /// The fixed colliders of the world.
    pub(super) floor: &'a Floor<'w, 's>,
    /// The streamed ground, which says which triangles are drawn where. A world built by hand,
    /// as in a test, has none: its colliders are where it wants them and it has no bare ground.
    pub(super) chunks: Option<&'a Chunks>,
    /// The collider of the body and its turn: any fixed collider it would meet in the column
    /// over and under it rules the bare ground out.
    pub(super) shape: (&'a Collider, Quat),
}

impl Bare<'_, '_, '_> {
    /// Returns how far the soles of a body centred at render position `at` are above the ground
    /// drawn there, negative under it, when no fixed collider is loaded around it. Nothing where
    /// one is, which the body stands on instead, or on a world with no ground.
    pub(super) fn gap(&self, at: Vec3) -> Option<f32> {
        let chunks = self.chunks.filter(|_| self.terrain.globe().has_ground())?;
        if self.floor.around(self.shape, at, up(self.anchor, at)) {
            return None;
        }
        let chart = self.terrain.locate(self.anchor.to_frame(at));
        let flat = Vec2::new(chart.x, chart.z);
        let drawn = chunks.drawn_at(self.terrain, flat).unwrap_or(Drawn::FINE);
        let soles = chart.y - BODY_LENGTH * 0.5 - BODY_RADIUS;
        Some(soles - drawn.height(self.terrain, flat))
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
    /// started on or over it and would end under it is lifted back toward it, by no more than it
    /// moved into it, see [`most_lift`], and loses the part of its velocity that went down. It
    /// lands on the surface and follows it, and is never pushed up faster than it came.
    pub(super) fn keep(&self, from: Vec3, out: &mut MoveAndSlideOutput) {
        let over = self.gap(from).is_some_and(|gap| gap >= -DEEP);
        let Some(gap) = self.gap(out.position).filter(|gap| over && *gap < 0.0) else {
            return;
        };
        let vertical = up(self.anchor, out.position);
        let moved = out.position - from;
        let fell = -moved.dot(vertical);
        let along = moved.reject_from(vertical).length();
        out.position += vertical * (-gap).min(most_lift(fell, along));
        let into = out.projected_velocity.dot(vertical).min(0.0);
        out.projected_velocity -= vertical * into;
    }
}
