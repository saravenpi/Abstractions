//! The perch: the place around the player the companion chose for itself.
//!
//! It is a bearing held in the world, not in the view, so turning the head never swings the
//! companion around. It moves on when the player has kept its back to it for a while, when it
//! hangs in the way of a walking player, or simply when it feels like it.

use bevy::math::DVec3;
use bevy::prelude::*;

use super::rng::Rng;
use super::spot::Stance;
use super::tune::{PERCH_ARC, PERCH_PATIENCE, PERCH_REST, STILL_SPEED};
use crate::space::Anchor;

/// Cosine of the angle from the way the player faces beyond which the perch is behind its back.
const BEHIND: f32 = -0.15;

/// Cosine of the angle from the course of a walking player within which the perch is in the way.
const IN_THE_WAY: f32 = 0.93;

/// Chance that a move it makes on its own crosses to the other side of the player.
const CROSS: f32 = 0.35;

/// Where the companion hangs around the player.
#[derive(Debug, Clone, Copy)]
pub struct Perch {
    /// Direction from the eyes to its place in the frame of the anchor, zero before the first choice.
    bearing: DVec3,
    /// The same direction in render space: horizontal and of unit length once chosen.
    pub side: Vec3,
    /// Seconds it has been out of place: behind the player or in its way.
    off: f32,
    /// Seconds before it moves somewhere else by itself.
    rest: f32,
    /// Random numbers for its choices.
    rng: Rng,
}

impl Perch {
    /// Returns a perch not chosen yet, whose choices follow `seed`.
    pub fn new(seed: u64) -> Self {
        let mut rng = Rng::new(seed);
        Self {
            bearing: DVec3::ZERO,
            side: Vec3::ZERO,
            off: 0.0,
            rest: rng.range(PERCH_REST.0, PERCH_REST.1),
            rng,
        }
    }

    /// Returns the way the player faces for the companion: its course when it walks, else its view.
    fn facing(stance: &Stance, drift: Vec3) -> (Vec3, bool) {
        let flat = drift - stance.up * drift.dot(stance.up);
        let moving = flat.length() > STILL_SPEED * 2.0;
        (
            flat.try_normalize()
                .filter(|_| moving)
                .unwrap_or_else(|| stance.level()),
            moving,
        )
    }

    /// Chooses a new place: off to one side of `ahead`, on the side `sign` unless it crosses over.
    fn choose(&mut self, stance: &Stance, ahead: Vec3, sign: f32, cross: bool) -> Vec3 {
        let sign = if cross { -sign } else { sign };
        let angle = self.rng.range(PERCH_ARC.0, PERCH_ARC.1);
        self.off = 0.0;
        self.rest = self.rng.range(PERCH_REST.0, PERCH_REST.1);
        ahead * angle.cos() + ahead.cross(stance.up) * angle.sin() * sign
    }

    /// Advances by `dt` seconds beside a player at `stance` moving at `drift`, and keeps the
    /// bearing through any change of the render anchor.
    pub fn advance(&mut self, anchor: &Anchor, stance: &Stance, drift: Vec3, dt: f32) {
        let held = (anchor.rotation * self.bearing).as_vec3();
        let held = (held - stance.up * held.dot(stance.up)).try_normalize();
        let (ahead, moving) = Self::facing(stance, drift);
        let along = held.map_or(1.0, |dir| dir.dot(ahead));
        let misplaced = along < BEHIND || (moving && along > IN_THE_WAY);
        self.off = if misplaced { self.off + dt } else { 0.0 };
        self.rest -= dt;
        let patience = if moving {
            PERCH_PATIENCE.0
        } else {
            PERCH_PATIENCE.1
        };
        let sign = held.map_or(1.0, |dir| dir.dot(ahead.cross(stance.up)).signum());
        let side = match held {
            Some(dir) if self.off < patience && self.rest > 0.0 => dir,
            _ => {
                let cross = held.is_some() && self.off < patience && self.rng.next() < CROSS;
                self.choose(stance, ahead, sign, cross)
            }
        };
        self.side = side;
        self.bearing = anchor.rotation.inverse() * side.as_dvec3();
    }
}
