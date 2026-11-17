//! What moves the camera besides the look: lag, steps, landings, banking, looking ahead and zoom.

use std::f32::consts::{E, TAU};

use bevy::prelude::*;

use super::basis::Stance;
use super::io::{Kind, Moment};
use super::shot::Shot;
use super::smooth::{Spring, closed};
use super::tune::{
    BANK_LIMIT, BOB, BOB_RATE, BOB_SPEED, DIP, FLY_SWIFT, LAG_TIME, LAND_SPEED, LEAD_LIMIT,
    LEAD_RATE, ROLL_RATE, RUSH_RATE, SHIP_SWIFT, STRIDE, WARP, ZOOM_RATE,
};

/// The slow state of the rig. Vectors are in the axes of the target.
#[derive(Debug, Default, Clone, Copy, PartialEq)]
pub(super) struct Sway {
    /// How far the camera lags behind the moving target, with the dip of a landing.
    lag: Spring,
    /// Steps walked, as a fraction of the current one.
    stride: f32,
    /// Height of the head bob right now.
    bob: f32,
    /// Vertical speed of the target on the last step.
    fall: f32,
    /// Roll of the camera, positive to the right.
    pub roll: f32,
    /// Turn of the camera toward the flight path: yaw and pitch.
    pub lead: Vec2,
    /// Zoom the boom has reached.
    pub zoom: f32,
    /// How much speed widens the view: 0 at rest, 1 at full speed, more during a warp.
    pub rush: f32,
}

/// Returns how much the speed of the target should widen the view.
fn rush(kind: Kind, speed: f32) -> f32 {
    match kind {
        Kind::Ship => closed(1.0 / SHIP_SWIFT, speed) + WARP.1 * closed(1.0 / WARP.0, speed),
        Kind::OnFoot | Kind::Flying => closed(1.0 / FLY_SWIFT, speed),
    }
}

impl Sway {
    /// Returns the state at rest, zoomed to `zoom`.
    pub(super) fn settled(zoom: f32) -> Self {
        Self { zoom, ..default() }
    }

    /// Moves the lag spring: the camera trails the velocity up to the limit of the shot, and a
    /// landing kicks it downward.
    fn trail(&mut self, kind: Kind, stance: &Stance, shot: &Shot) {
        let speed = stance.drift.length();
        let reach = shot.lag * closed(LAG_TIME / shot.lag.max(1.0e-3), speed);
        let goal = -stance.drift.normalize_or_zero() * reach;
        let landed = kind == Kind::OnFoot && self.fall < -LAND_SPEED && stance.drift.y > -1.0;
        if landed {
            let depth = (-self.fall * DIP.0).min(DIP.1);
            self.lag.velocity.y -= depth * shot.stiff * E;
        }
        self.fall = stance.drift.y;
        match stance.fresh {
            true => self.lag.value = goal,
            false => self.lag.step(goal, shot.stiff, stance.dt),
        }
    }

    /// Walks the stride and sizes the head bob: full at walking pace on the ground in first person.
    fn walk(&mut self, moment: &Moment, stance: &Stance, shot: &Shot) {
        let pace = Vec2::new(stance.drift.x, stance.drift.z).length();
        let walking = moment.target.kind == Kind::OnFoot && stance.drift.y.abs() < 0.5 + 0.6 * pace;
        let wanted = moment.prefs.head_bob && walking;
        let size = BOB * (pace / BOB_SPEED).min(1.0) * (1.0 - shot.third);
        let goal = if wanted { size } else { 0.0 };
        self.stride = (self.stride + pace * stance.dt / STRIDE).fract();
        self.bob += (goal - self.bob) * stance.gain(BOB_RATE);
    }

    /// Turns the camera a little toward where a flying body is going, seen from the `look` of the camera.
    fn ahead(&mut self, stance: &Stance, shot: &Shot, look: Quat) {
        let path = look.inverse() * stance.drift.normalize_or_zero();
        let goal = match path.z < 0.0 {
            true => Vec2::new((-path.x).atan2(-path.z), path.y.clamp(-1.0, 1.0).asin()),
            false => Vec2::ZERO,
        };
        let goal = (goal * shot.lead).clamp(Vec2::splat(-LEAD_LIMIT), Vec2::splat(LEAD_LIMIT));
        self.lead += (goal - self.lead) * stance.gain(LEAD_RATE);
    }

    /// Runs the state for one step. `look` is the orientation of the camera in the axes of the target.
    pub(super) fn step(&mut self, moment: &Moment, stance: &Stance, shot: &Shot, look: Quat) {
        let target = &moment.target;
        self.trail(target.kind, stance, shot);
        self.walk(moment, stance, shot);
        self.ahead(stance, shot, look);
        let bank = target.bank.clamp(-BANK_LIMIT, BANK_LIMIT) * shot.bank;
        self.roll += (bank - self.roll) * stance.gain(ROLL_RATE);
        self.zoom += (moment.prefs.zoom - self.zoom) * stance.gain(ZOOM_RATE);
        let goal = rush(target.kind, stance.drift.length());
        self.rush += (goal - self.rush) * stance.gain(RUSH_RATE);
    }

    /// Returns the offset of the camera in the axes of the target: the lag and the head bob.
    pub(super) fn offset(&self) -> Vec3 {
        self.lag.value + Vec3::Y * self.bob * (TAU * self.stride).sin()
    }
}
