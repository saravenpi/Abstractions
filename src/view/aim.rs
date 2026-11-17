//! The look angles: what the player turns, what the rig turns by itself, and the short trail behind them.
//!
//! The player always wins. The rig only turns the camera by itself once the player has left
//! it alone, and any input takes it back at once.

use bevy::prelude::*;

use super::basis::Stance;
use super::io::{Heading, Kind, Moment};
use super::smooth::{closed, trail, wrap};
use super::tune::{
    FOLLOW_DELAY, FOLLOW_RATE, FOLLOW_SPEED, ORBIT_RETURN, PITCH_LIMIT, THIRD_DOWN, THIRD_UP,
    TRAIL_RATE,
};

/// The look angles and how they got there.
#[derive(Debug, Default, Clone, Copy, PartialEq)]
pub(super) struct Aim {
    /// What the player aims at. On foot it is the heading; in the ship, the free orbit around it.
    pub heading: Heading,
    /// How far the camera trails behind the heading, in yaw and in pitch.
    trail: Vec2,
    /// Seconds since the player last turned the camera.
    idle: f32,
}

impl Aim {
    /// Returns an aim at rest on `heading`.
    pub(super) fn settled(heading: Heading) -> Self {
        Self {
            heading,
            ..default()
        }
    }

    /// Returns the turn that springs the free orbit around the ship back behind it.
    fn recentre(&mut self, held: bool, dt: f32) -> Vec2 {
        if held {
            return Vec2::ZERO;
        }
        self.heading.yaw = wrap(self.heading.yaw);
        -Vec2::new(self.heading.yaw, self.heading.pitch) * closed(ORBIT_RETURN, dt)
    }

    /// Returns the yaw that drifts a third person camera behind the direction of travel.
    ///
    /// Nothing drifts before the player has left the camera alone for [`FOLLOW_DELAY`], while
    /// the body stands, or when it comes toward the camera.
    fn follow(&self, moment: &Moment, stance: &Stance, third: f32) -> f32 {
        let pace = Vec2::new(stance.drift.x, stance.drift.z);
        let waits = self.idle < FOLLOW_DELAY || !moment.prefs.auto_follow;
        if waits || pace.length() < FOLLOW_SPEED {
            return 0.0;
        }
        let gap = wrap((-pace.x).atan2(-pace.y) - self.heading.yaw);
        gap * closed(FOLLOW_RATE * gap.cos().max(0.0) * third, stance.dt)
    }

    /// Turns the aim by what the player asks, or by itself when the player asks nothing.
    pub(super) fn step(&mut self, moment: &Moment, stance: &Stance, third: f32) {
        let (look, ship) = (&moment.look, moment.target.kind == Kind::Ship);
        let asked = match ship && !look.orbit_held {
            true => Vec2::ZERO,
            false => Vec2::new(look.yaw_delta, look.pitch_delta),
        };
        self.idle = match asked == Vec2::ZERO {
            true => self.idle + stance.dt,
            false => 0.0,
        };
        let own = match ship {
            true => self.recentre(look.orbit_held, stance.dt),
            false => Vec2::X * self.follow(moment, stance, third),
        };
        let before = Vec2::new(self.heading.yaw, self.heading.pitch);
        self.heading.yaw += asked.x + own.x;
        self.heading.pitch = (before.y + asked.y + own.y).clamp(-PITCH_LIMIT, PITCH_LIMIT);
        let turned = Vec2::new(self.heading.yaw, self.heading.pitch) - before;
        self.trail = trail(self.trail, turned, TRAIL_RATE, stance.dt);
    }

    /// Returns the yaw and pitch the camera shows, `third` of the way to a third person view:
    /// first person shows the aim itself, third person trails it and stops short of the poles.
    pub(super) fn shown(&self, third: f32) -> (f32, f32) {
        let yaw = self.heading.yaw - self.trail.x * third;
        let pitch = self.heading.pitch - self.trail.y * third;
        (yaw, pitch.lerp(pitch.clamp(-THIRD_DOWN, THIRD_UP), third))
    }
}
