//! The brain: what the companion knows about the player and what it decided this frame.

use bevy::math::DVec3;
use bevy::prelude::*;

use super::attention::Attention;
use super::idle::Idle;
use super::mind::Mind;
use super::pose::Pose;
use super::sense::CompanionSense;
use super::spot::Stance;
use super::tune::{CLEARANCE, CURIOUS_REACH, LOW_CLEARANCE};
use crate::space::{Anchor, Frame};
use crate::universe::BodyId;

/// Share of the inspect idle spent away, before it heads back to the player.
const INSPECT_AWAY: f32 = 0.62;

/// How far ahead of the player the ground it inspects lies, in meters.
const INSPECT_REACH: f32 = 3.2;

/// Speed in meters per second above which the player was moved rather than moving, and carries nothing along.
const MAX_DRIFT: f32 = 60.0;

/// Decisions and memory of the companion, read by the systems that move and draw it.
#[derive(Component, Debug, Clone)]
pub struct Brain {
    /// The idle behaviours.
    pub mind: Mind,
    /// What it looks at.
    pub attention: Attention,
    /// Where the player's eyes were on the last frame.
    pub stance: Stance,
    /// Velocity of the player in render axes, in meters per second.
    pub drift: Vec3,
    /// Render position it looks at.
    pub look: Vec3,
    /// Render position of what it inspects during the inspect idle.
    pub inspect: Vec3,
    /// How the body is posed on top of following.
    pub pose: Pose,
    /// Frame and position in it of the player's eyes on the last frame.
    seen: Option<(Frame, DVec3)>,
    /// Planets the player has walked on since the companion appeared.
    visited: Vec<BodyId>,
    /// The discovery count already celebrated.
    delight: u32,
}

impl Brain {
    /// Returns a fresh brain whose choices follow `seed`.
    pub fn new(seed: u64) -> Self {
        Self {
            mind: Mind::new(seed),
            attention: Attention::new(seed ^ 0x5EED),
            stance: Stance {
                eye: Vec3::ZERO,
                forward: Vec3::NEG_Z,
                up: Vec3::Y,
            },
            drift: Vec3::ZERO,
            look: Vec3::ZERO,
            inspect: Vec3::ZERO,
            pose: Pose::default(),
            seen: None,
            visited: Vec::new(),
            delight: 0,
        }
    }

    /// Records where the player's eyes are and returns how fast the view turns, in radians per second.
    ///
    /// The player's velocity is measured in the anchor frame, so a rebase of the anchor never
    /// reads as movement. It is zero on the frame the frame itself changes or the player is teleported.
    pub fn observe(&mut self, anchor: &Anchor, stance: Stance, dt: f32) -> f32 {
        let dt = dt.max(1e-4);
        let at = anchor.to_frame(stance.eye);
        let moved = match self.seen {
            Some((frame, was)) if frame == anchor.frame => at - was,
            _ => DVec3::ZERO,
        };
        let drift = (anchor.rotation * moved).as_vec3() / dt;
        self.drift = drift * f32::from(drift.length() < MAX_DRIFT);
        let turn = self.stance.forward.angle_between(stance.forward) / dt;
        self.seen = Some((anchor.frame, at));
        self.stance = stance;
        turn
    }

    /// Returns true once for every discovery: the first steps on a new planet, or a producer's count going up.
    ///
    /// The planet the companion first appears on is not a discovery.
    pub fn discovered(&mut self, sense: &CompanionSense, body: BodyId) -> bool {
        let fresh = !sense.seated && !self.visited.contains(&body);
        if fresh {
            self.visited.push(body);
        }
        let told = sense.delight != self.delight;
        self.delight = sense.delight;
        told || (fresh && self.visited.len() > 1)
    }

    /// Returns the structure close enough to be curious about, if any.
    pub fn nearby(&self, sense: &CompanionSense) -> Option<Vec3> {
        sense
            .structure
            .filter(|at| at.distance(self.stance.eye) < CURIOUS_REACH)
    }

    /// Chooses what the inspect idle looks at: the nearby structure, else the ground ahead of the player.
    pub fn choose_inspect(&mut self, sense: &CompanionSense) {
        let ground = self.stance.eye + self.stance.level() * INSPECT_REACH
            - self.stance.right() * 0.8
            - self.stance.up * 1.4;
        self.inspect = self.nearby(sense).unwrap_or(ground);
    }

    /// Returns true while the inspect idle keeps the companion away from the player.
    fn away(&self) -> bool {
        let far = Idle::Inspect.seconds() * INSPECT_AWAY;
        self.mind.idle == Some(Idle::Inspect) && self.mind.time < far
    }

    /// Returns the spot to float at, before ground clearance, and the clearance to keep there.
    pub fn goal(&self) -> (Vec3, f32) {
        if self.away() {
            let back = (self.stance.eye - self.inspect).normalize_or_zero();
            return (
                self.inspect + back * 0.6 + self.stance.up * 0.3,
                LOW_CLEARANCE,
            );
        }
        (self.stance.rest(), CLEARANCE)
    }

    /// Returns the render position to look at: what the idle calls for, else where the attention is.
    pub fn focus(&self) -> Vec3 {
        let stance = &self.stance;
        let ahead = stance.rest() + stance.level() * 6.0;
        match (self.mind.dozing, self.mind.idle) {
            (true, _) => ahead - stance.up * 3.0,
            (false, Some(Idle::Inspect)) if self.away() => self.inspect,
            (false, Some(Idle::SkyPeek)) => ahead + stance.up * 14.0,
            (false, Some(Idle::LookAround | Idle::Spin | Idle::Dance)) => ahead,
            _ => self.attention.point(stance),
        }
    }
}
