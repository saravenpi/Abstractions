//! The gaze: which thing the companion looks at, for how long, and what makes it look away.
//!
//! Each thing has a weight and a dwell time. When the dwell is over the next one is drawn by
//! weight. A stop, a jump or a landing of the player draws a quick glance at it, and a still
//! player who keeps the camera on the companion gets a short look back.

use bevy::prelude::*;

use super::attention::Focus;
use super::rng::Rng;
use super::scene::Scene;
use super::tune::{STILL_SPEED, WATCHED};

/// Seconds the eyes stay on anything before something that happened may draw them away.
const REACT: f32 = 0.6;

/// Climb rate in meters per second above which the player counts as jumping.
const JUMP: f32 = 2.5;

/// Fall rate in meters per second above which coming to rest counts as a landing.
const LAND: f32 = 3.0;

/// Speed in meters per second above which coming to rest counts as stopping.
const STOP: f32 = 1.5;

/// Cosine of the angle off the view within which the camera counts as watching the companion.
const WATCH_CONE: f32 = 0.98;

/// Widest turn from the way away from the player, and lowest and highest rise, of a look
/// around, in radians.
const WANDER: (f32, f32, f32) = (2.6, -0.3, 0.25);

/// Where the attention is and for how long.
#[derive(Debug, Clone, Copy)]
pub struct Attention {
    /// What it looks at now.
    pub focus: Focus,
    /// Seconds it has looked at it.
    held: f32,
    /// Seconds left before it looks at something else.
    left: f32,
    /// True when something the player did calls for a glance at it.
    urge: bool,
    /// Seconds a still player has kept the camera on the companion.
    watched: f32,
    /// Seconds before it may look back at the camera again.
    wait: f32,
    /// Speed along the ground and climb rate of the player on the last frame.
    was: Vec2,
    /// Turn and rise of the look around in progress.
    wander: Vec2,
    /// Random numbers for its choices.
    rng: Rng,
}

impl Attention {
    /// Returns an attention on the player whose choices follow `seed`.
    pub fn new(seed: u64) -> Self {
        Self {
            focus: Focus::Player,
            held: 0.0,
            left: 2.0,
            urge: false,
            watched: 0.0,
            wait: 0.0,
            was: Vec2::ZERO,
            wander: Vec2::ZERO,
            rng: Rng::new(seed),
        }
    }

    /// Notes a stop, a jump or a landing of the player, which call for a glance at it.
    fn notice(&mut self, scene: &Scene) {
        let climb = scene.drift.dot(scene.up);
        let speed = (scene.drift - scene.up * climb).length();
        let jumped = climb > JUMP && self.was.y <= JUMP;
        let landed = self.was.y < -LAND && climb > -LAND * 0.3;
        let stopped = self.was.x > STOP && speed < STILL_SPEED;
        self.urge |= jumped || landed || stopped;
        self.was = Vec2::new(speed, climb);
    }

    /// Returns true when a still player has watched the companion long enough for a look back.
    fn watched(&mut self, scene: &Scene, dt: f32) -> bool {
        let toward = (scene.here - scene.camera).normalize_or_zero();
        let still = self.was.x < STILL_SPEED && Focus::Camera.open(scene);
        let seen = still && scene.view.dot(toward) > WATCH_CONE;
        self.watched = if seen { self.watched + dt } else { 0.0 };
        self.watched > WATCHED.0 && self.wait <= 0.0
    }

    /// Looks at `focus` from now on, for a dwell time of its own.
    fn look(&mut self, focus: Focus) {
        let (low, high) = focus.dwell();
        (self.focus, self.held, self.left) = (focus, 0.0, self.rng.range(low, high));
        let turn = self.rng.range(-WANDER.0, WANDER.0);
        self.wander = Vec2::new(turn, self.rng.range(WANDER.1, WANDER.2));
    }

    /// Draws what to look at next by weight among what `scene` offers.
    fn pick(&mut self, scene: &Scene) -> Focus {
        let weight = |focus: &Focus| focus.weight(scene, self.focus);
        let total: f32 = Focus::CHOICES.iter().map(weight).sum();
        let mut roll = self.rng.next() * total;
        let mut chosen = Focus::Player;
        for focus in Focus::CHOICES.iter().filter(|focus| weight(focus) > 0.0) {
            chosen = *focus;
            if roll < weight(focus) {
                break;
            }
            roll -= weight(focus);
        }
        chosen
    }

    /// Advances by `dt` seconds in `scene` and moves the eyes on when their time is up, when
    /// what they look at is gone, or when the player does something worth a glance.
    pub fn advance(&mut self, scene: &Scene, dt: f32) {
        self.notice(scene);
        let back = self.watched(scene, dt);
        (self.held, self.left, self.wait) = (self.held + dt, self.left - dt, self.wait - dt);
        let ready = self.held >= REACT;
        if back && self.focus != Focus::Camera {
            self.look(Focus::Camera);
            (self.wait, self.watched) = (WATCHED.1, 0.0);
        } else if ready && self.urge && self.focus != Focus::Player {
            self.look(Focus::Player);
        } else if self.left <= 0.0 || (ready && !self.focus.open(scene)) {
            let next = self.pick(scene);
            self.look(next);
        }
        self.urge &= self.focus != Focus::Player;
    }

    /// Returns the render position the companion looks at in `scene`.
    pub fn point(&self, scene: &Scene) -> Vec3 {
        self.focus.point(scene, self.wander)
    }
}
