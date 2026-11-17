//! Attention: whether the companion looks at the player, ahead, or at what the player stares at.

use bevy::prelude::*;

use super::rng::Rng;
use super::spot::Stance;
use super::tune::{PLAYER_SHARE, STARE_AFTER, STARE_REACH, STARE_TURN};

/// Angle in radians from straight ahead beyond which the eyes turn toward a target.
const GLANCE: f32 = 0.3;

/// Distance in meters of the point it looks at when it looks where the player is heading.
const HORIZON: f32 = 9.0;

/// What the companion looks at.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum Focus {
    /// The player's eyes.
    #[default]
    Player,
    /// The way the player is heading.
    Ahead,
    /// What the player stares at.
    Stare,
}

/// Where the attention is and for how long.
#[derive(Debug, Clone, Copy)]
pub struct Attention {
    /// What it looks at now.
    pub focus: Focus,
    /// Seconds left before it looks at something else.
    left: f32,
    /// Seconds the player's view has been steady.
    steady: f32,
    /// Random numbers for the glances.
    rng: Rng,
}

impl Attention {
    /// Returns an attention on the player whose glances follow `seed`.
    pub fn new(seed: u64) -> Self {
        Self {
            focus: Focus::Player,
            left: 2.0,
            steady: 0.0,
            rng: Rng::new(seed),
        }
    }

    /// Returns true while the player has been staring at one thing long enough.
    pub fn staring(&self) -> bool {
        self.steady > STARE_AFTER
    }

    /// Advances by `dt` seconds. `turn` is the turn rate of the view in radians per second.
    ///
    /// While the player stares, the companion looks at the same thing with short glances back at
    /// the player. Otherwise it looks at the player more often than ahead.
    pub fn advance(&mut self, turn: f32, still: bool, dt: f32) {
        let was = self.staring();
        self.steady = if still && turn < STARE_TURN {
            self.steady + dt
        } else {
            0.0
        };
        self.left -= dt;
        if self.staring() == was && self.left > 0.0 {
            return;
        }
        let glance = self.rng.next() < PLAYER_SHARE;
        (self.focus, self.left) = match (self.staring(), self.focus, glance) {
            (true, Focus::Stare, _) => (Focus::Player, self.rng.range(1.0, 2.0)),
            (true, ..) => (Focus::Stare, self.rng.range(3.5, 6.0)),
            (false, _, true) => (Focus::Player, self.rng.range(2.0, 4.5)),
            (false, _, false) => (Focus::Ahead, self.rng.range(2.0, 5.0)),
        };
    }

    /// Returns the render position the companion looks at.
    pub fn point(&self, stance: &Stance) -> Vec3 {
        match self.focus {
            Focus::Player => stance.eye,
            Focus::Ahead => stance.eye + stance.level() * HORIZON,
            Focus::Stare => stance.eye + stance.forward * STARE_REACH,
        }
    }
}

/// Returns where the eyes turn for a target at `local`, a direction in the body's own axes.
///
/// The screen faces -Z. The result is in picture axes: x toward the right of the picture, which
/// is the body's -X, and y up. Each axis is -1, 0 or 1.
pub fn eye_shift(local: Vec3) -> IVec2 {
    let side = local.x.atan2(-local.z);
    let rise = local.y.atan2(Vec2::new(local.x, local.z).length());
    let step = |angle: f32| i32::from(angle > GLANCE) - i32::from(angle < -GLANCE);
    IVec2::new(-step(side), step(rise))
}
