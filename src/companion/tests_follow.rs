//! A player flying over a round planet with the companion beside it: the world the flight
//! tests of the follow run in, with its anchor rebased as the player goes.

use std::sync::Arc;

use bevy::prelude::*;

use super::brain::Brain;
use super::motion::{Float, blend, follow};
use super::spot::{Probe, Stance};
use super::tune::FLIGHT_RATE;
use crate::space::{Anchor, Shift, rebased, up};
use crate::universe::Universe;
use crate::world::globe::Face;
use crate::world::{Globe, Terrain};

/// Sea level radius of the test planet in metres: small, so the anchor is rebased every 30 m.
const RADIUS: f64 = 3000.0;

/// A player flying over the test planet with the companion beside it.
pub struct Sky {
    /// The test world.
    pub terrain: Terrain,
    /// Its anchor, rebased as the player goes.
    pub anchor: Anchor,
    /// The companion's brain.
    pub brain: Brain,
    /// The companion's pose.
    pub at: Transform,
    /// The companion's motion.
    pub float: Float,
    /// Render position of the player's eyes.
    pub eye: Vec3,
    /// Whether the companion sees the player where it was a frame ago, as a system that runs
    /// before the controller does.
    pub stale: bool,
    /// Anchor rebases so far.
    pub rebases: usize,
}

impl Sky {
    /// Returns a flight `height` metres over the ground, the companion a metre from the eyes.
    pub fn new(height: f32) -> Self {
        let globe = Arc::new(Globe::new(4, RADIUS, 22.0));
        let terrain = Terrain::on(globe.clone(), globe.face_chart(Face::PosX)).around(40.0, -25.0);
        let ground = terrain.height(40.0, -25.0).max(0.0);
        let eye = Vec3::new(40.0, ground + height, -25.0);
        let mut sky = Self {
            anchor: Anchor::surface(Universe::new(4).home(), &terrain, eye),
            terrain,
            brain: Brain::new(7),
            at: Transform::from_translation(eye + Vec3::X),
            float: Float::default(),
            eye,
            stale: false,
            rebases: 0,
        };
        sky.at.translation = sky.eye + Vec3::X;
        sky
    }

    /// Rebases the anchor under the player when it is due, and carries the companion along as
    /// the game does for every free entity.
    pub fn rebase(&mut self) {
        let Some(next) = rebased(&self.anchor, RADIUS, self.eye) else {
            return;
        };
        let shift = Shift::between(&self.anchor, &next);
        self.eye = shift.point(self.eye);
        self.at.translation = shift.point(self.at.translation);
        self.at.rotation = shift.turn(self.at.rotation);
        (self.anchor, self.rebases) = (next, self.rebases + 1);
    }

    /// Returns `dir` flattened onto the horizontal plane at the player, at `speed`.
    pub fn level(&self, dir: Vec3, speed: f32) -> Vec3 {
        let vertical = up(&self.anchor, self.eye);
        (dir - vertical * dir.dot(vertical)).normalize_or_zero() * speed
    }

    /// Returns the reader of the ground.
    pub fn probe(&self) -> Probe<'_> {
        Probe {
            anchor: &self.anchor,
            terrain: &self.terrain,
        }
    }

    /// Moves the flying player by `vel` for `dt` seconds and lets the companion follow. Returns
    /// how far the companion moved against the player in the frame, in metres.
    pub fn step(&mut self, vel: Vec3, dt: f32) -> f32 {
        self.rebase();
        let (before, seen) = (self.at.translation - self.eye, self.eye);
        self.eye += vel * dt;
        let vertical = up(&self.anchor, self.eye);
        let forward = vel.try_normalize().unwrap_or(self.brain.stance.forward);
        let stance = Stance {
            eye: if self.stale { seen } else { self.eye },
            forward,
            up: vertical,
        };
        self.brain.observe(&self.anchor, stance, dt);
        self.brain.side = forward.cross(vertical);
        self.brain.flight += (1.0 - self.brain.flight) * blend(FLIGHT_RATE, dt);
        self.brain.look = self.eye + forward * 10.0;
        let probe = Probe {
            anchor: &self.anchor,
            terrain: &self.terrain,
        };
        follow(&probe, &self.brain, &mut self.at, &mut self.float, dt);
        (self.at.translation - self.eye - before).length()
    }
}
