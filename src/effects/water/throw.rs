//! What a churning source leaves this frame: spray flecks, bubbles, wake segments and rings.

use bevy::math::DVec3;
use bevy::prelude::*;

use super::super::rng::Rng;
use super::arms::{Pass, quads};
use super::drops::{Drop, Drops};
use super::foam::{Foam, Quad, lay, sectors};
use super::tune::{
    DROP_LIFE, FOAM_LIFE, FOAM_LIFT, RING_PIECES, RIPPLE_BAND, SPRAY_CARRY, SPRAY_KICK, WAKE_BREAK,
    WAKE_STEP,
};
use super::wash::Flow;
use crate::space::Anchor;

/// One source on this frame, in render space.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Spot {
    /// Where the source is.
    pub at: Vec3,
    /// Unit local up there.
    pub up: Vec3,
    /// Height of the source above the sea in metres, negative under it.
    pub height: f32,
    /// Gravity in metres per second squared.
    pub gravity: f32,
    /// Velocity over the sea in metres per second, along the surface.
    pub velocity: Vec3,
    /// Size of the source in metres.
    pub size: f32,
    /// Foam clock in seconds.
    pub now: f32,
}

impl Spot {
    /// Returns the point of the sea surface under the source, with the foam lift.
    pub fn surface(&self) -> Vec3 {
        self.at - self.up * (self.height - FOAM_LIFT)
    }
}

/// The last point of a wake, kept between frames in the frame of the anchor.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Mark {
    /// Frame position of the point on the sea.
    pub at: DVec3,
    /// Unit direction across the wake there.
    pub side: Vec3,
    /// Unit local up there.
    pub up: Vec3,
    /// Foam clock when it was laid.
    pub birth: f32,
}

/// The pooled drops and the randomness every throw takes from.
pub struct Hand<'a, 'w, 's> {
    /// The pool order.
    pub drops: &'a mut Drops,
    /// The pooled drops.
    pub pool: &'a mut Query<'w, 's, (&'static mut Drop, &'static mut Transform)>,
    /// Randomness.
    pub rng: &'a mut Rng,
}

impl Hand<'_, '_, '_> {
    /// Throws one drop of the kind of `drop` from `at`.
    fn throw(&mut self, at: Vec3, drop: Drop) {
        let Some(entity) = self.drops.take(drop.bubble) else {
            return;
        };
        if let Ok((mut slot, mut transform)) = self.pool.get_mut(entity) {
            *slot = drop;
            transform.translation = at;
        }
    }

    /// Throws `count` spray flecks off the sea under `spot`, up with the wash and along with
    /// the flight (the per-distance spray of Blood Stone,
    /// https://cgw.com/Press-Center/Web-Exclusives/2010/Spy-Game-Blood-Stone-s-water-effects.aspx).
    pub fn spray(&mut self, spot: &Spot, flow: &Flow, count: u32) {
        let base = spot.surface();
        for _ in 0..count {
            let out = Quat::from_axis_angle(spot.up, self.rng.range(0.0, std::f32::consts::TAU))
                * spot.up.any_orthonormal_vector();
            let reach = self.rng.range(0.1, 0.5) * (spot.size + spot.height.max(0.0));
            let lift = flow.surface.max(spot.velocity.length() * 0.3) * SPRAY_KICK;
            let velocity = spot.up * lift * self.rng.range(0.5, 1.0)
                + spot.velocity * SPRAY_CARRY
                + out * lift * 0.4;
            let drop = Drop {
                velocity,
                up: spot.up,
                level: FOAM_LIFT,
                gravity: spot.gravity,
                age: 0.0,
                life: self.rng.range(DROP_LIFE.0, DROP_LIFE.1),
                bubble: false,
            };
            self.throw(base + out * reach, drop);
        }
    }

    /// Lets `count` bubbles go from a jet at `spot`, under the sea.
    pub fn bubbles(&mut self, spot: &Spot, count: u32) {
        for _ in 0..count {
            let jitter = Vec3::new(self.rng.next(), self.rng.next(), self.rng.next()) - 0.5;
            let drop = Drop {
                velocity: -spot.velocity * 0.2 + jitter,
                up: spot.up,
                level: spot.height,
                gravity: spot.gravity,
                age: 0.0,
                life: self.rng.range(DROP_LIFE.0, DROP_LIFE.1) * 2.0,
                bubble: true,
            };
            self.throw(spot.at + jitter * 0.3, drop);
        }
    }
}

/// Lays the next segment of the wake of `spot`, three quads of foam, once it travelled
/// [`WAKE_STEP`] past `mark`, or starts a new wake when there is none or it broke off. Returns
/// the mark to keep.
pub fn wake(
    (meshes, foam, origin): (&mut Assets<Mesh>, &mut Foam, Vec3),
    anchor: &Anchor,
    (spot, strength): (&Spot, f32),
    mark: Option<Mark>,
) -> Option<Mark> {
    let here = spot.surface();
    let side = spot.up.cross(spot.velocity).normalize_or_zero();
    let fresh = Mark {
        at: anchor.to_frame(here),
        side,
        up: spot.up,
        birth: spot.now,
    };
    let Some(last) = mark else {
        return Some(fresh);
    };
    let from = anchor.to_render(last.at);
    let gap = from.distance(here);
    if gap > WAKE_BREAK {
        return Some(fresh);
    }
    if gap < WAKE_STEP {
        return mark;
    }
    let ends = [from - origin, here - origin];
    for quad in quads(ends, [&last, &fresh], &Pass::of(spot, strength)) {
        lay(meshes, foam, &quad);
    }
    Some(fresh)
}

/// Lays one ripple ring on the sea under `spot`, running out at `speed`.
pub fn ring(
    (meshes, foam, origin): (&mut Assets<Mesh>, &mut Foam, Vec3),
    spot: &Spot,
    (speed, strength): (f32, f32),
) {
    let centre = spot.surface() - origin;
    let start = 0.5 * spot.size;
    for dirs in sectors(spot.up, RING_PIECES) {
        let quad = Quad {
            ends: [centre; 2],
            dirs,
            shape: Vec4::new(start, speed, 0.5 * RIPPLE_BAND, 0.0),
            births: [spot.now; 2],
            fade: Vec2::new(strength, FOAM_LIFE),
        };
        lay(meshes, foam, &quad);
    }
}
