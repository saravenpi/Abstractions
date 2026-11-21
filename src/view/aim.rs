//! The look angles: what the player turns, what the rig turns by itself, and the short trail behind them.
//!
//! The player always wins. The rig only turns the camera by itself once the player has left
//! it alone, and any input takes it back at once. With auto-follow off it never turns it by
//! itself: the look the player left stays, relative to the ship, until the recentre key.

use std::f32::consts::{FRAC_PI_2, PI};

use bevy::prelude::*;

use super::basis::Stance;
use super::curve;
use super::io::{Heading, Kind, Moment};
use super::smooth::{closed, trail, wrap};
use super::tune::{
    FOLLOW_DELAY, FOLLOW_RATE, FOLLOW_SPEED, ORBIT_HOLD, ORBIT_RETURN, PITCH_LIMIT, THIRD_DOWN,
    THIRD_UP, TRAIL_RATE,
};

/// The look angles and how they got there.
#[derive(Debug, Default, Clone, Copy, PartialEq)]
pub(super) struct Aim {
    /// What the player aims at. On foot it is the heading; in the ship, the free look around it.
    /// It is kept whole while the pilot seat holds the view inside the reach of the head, so a
    /// chase angle comes back when the view leaves the seat again.
    pub heading: Heading,
    /// The heading as the view takes it: inside the reach of the head in the pilot seat.
    pub view: Heading,
    /// How far the camera trails behind the heading, in yaw and in pitch.
    trail: Vec2,
    /// Seconds since the player last turned the camera.
    idle: f32,
    /// How far the view is that of a flying body, from 0 to 1. Such a view may look all the
    /// way round, so the third person camera goes over the poles with it.
    pub flight: f32,
}

/// Returns the look angles `yaw` and `pitch` kept where the view may go. A flying body may
/// look past the vertical and all the way round, which is how it loops. Anything else looks
/// up or down as far as [`PITCH_LIMIT`]; a view left on its back by a flight is put back on
/// its feet, facing the way it was looking.
fn bound(flying: bool, turned: Vec2) -> Heading {
    let (mut yaw, mut pitch) = (turned.x, turned.y);
    if flying && pitch.abs() > PI {
        pitch = wrap(pitch);
    } else if !flying && pitch.abs() > FRAC_PI_2 + 0.05 {
        (yaw, pitch) = (yaw + PI, pitch.signum() * PI - pitch);
    }
    let limit = if flying { PI } else { PITCH_LIMIT };
    Heading {
        yaw,
        pitch: pitch.clamp(-limit, limit),
    }
}

impl Aim {
    /// Returns an aim at rest on `heading`.
    pub(super) fn settled(heading: Heading) -> Self {
        Self {
            heading,
            view: heading,
            ..default()
        }
    }

    /// Returns the turn that brings the free look around the ship back to its nose: all of it
    /// on the frame the recentre key is pressed, and with auto-follow on a spring once the
    /// player has left the mouse alone for [`ORBIT_HOLD`]. With auto-follow off and no key the
    /// look stays where the player left it.
    fn recentre(&mut self, moment: &Moment, dt: f32) -> Vec2 {
        let asked = moment.look.recentre;
        let sprung = moment.prefs.auto_follow && self.idle >= ORBIT_HOLD;
        if !asked && !sprung {
            return Vec2::ZERO;
        }
        self.heading.yaw = wrap(self.heading.yaw);
        let share = match asked {
            true => 1.0,
            false => closed(ORBIT_RETURN, dt),
        };
        -Vec2::new(self.heading.yaw, self.heading.pitch) * share
    }

    /// Returns the yaw that drifts a third person camera behind the direction of travel.
    ///
    /// Nothing drifts before the player has left the camera alone for [`FOLLOW_DELAY`], while
    /// the body stands, or when it comes toward the camera. Nothing drifts behind a flying
    /// body either: its view is free to look around, and the player module brings it back
    /// behind the wings in its own time.
    fn follow(&self, moment: &Moment, stance: &Stance, third: f32) -> f32 {
        let pace = Vec2::new(stance.drift.x, stance.drift.z);
        let free = moment.target.kind == Kind::Flying || !moment.prefs.auto_follow;
        if free || self.idle < FOLLOW_DELAY || pace.length() < FOLLOW_SPEED {
            return 0.0;
        }
        let gap = wrap((-pace.x).atan2(-pace.y) - self.heading.yaw);
        gap * closed(FOLLOW_RATE * gap.cos().max(0.0) * third, stance.dt)
    }

    /// Turns the aim by what the player asks, or by itself when the player asks nothing, and
    /// keeps the view inside the reach of the head in the pilot seat. The mouse turns the view
    /// from where it shows, so it never moves the heading unseen past that reach.
    pub(super) fn step(&mut self, moment: &Moment, stance: &Stance, third: f32) {
        let (look, kind) = (&moment.look, moment.target.kind);
        let asked = Vec2::new(look.yaw_delta, look.pitch_delta);
        self.idle = match asked == Vec2::ZERO {
            true => self.idle + stance.dt,
            false => 0.0,
        };
        if asked != Vec2::ZERO {
            self.heading = curve::head(kind, self.heading, third);
        }
        let own = match kind == Kind::Ship {
            true => self.recentre(moment, stance.dt),
            false => Vec2::X * self.follow(moment, stance, third),
        };
        let before = Vec2::new(self.heading.yaw, self.heading.pitch);
        self.heading = bound(kind == Kind::Flying, before + asked + own);
        let turned = Vec2::new(self.heading.yaw, self.heading.pitch) - before;
        let turned = Vec2::new(turned.x, wrap(turned.y));
        self.trail = trail(self.trail, turned, TRAIL_RATE, stance.dt);
        self.view = curve::head(kind, self.heading, third);
    }

    /// Returns the yaw and pitch the camera shows, `third` of the way to a third person view:
    /// first person shows the aim itself, third person trails it and stops short of the poles.
    /// Behind a flying body it does not stop: a camera held at a pole while the view goes over
    /// the top of a loop would jump to the other pole when the view comes round.
    pub(super) fn shown(&self, third: f32) -> (f32, f32) {
        let yaw = self.view.yaw - self.trail.x * third;
        let pitch = self.view.pitch - self.trail.y * third;
        let short = pitch.clamp(-THIRD_DOWN, THIRD_UP);
        (yaw, pitch.lerp(short, third * (1.0 - self.flight)))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::view::{Look, Target, ViewMode, ViewPrefs};

    /// Returns the yaw of a third person view left alone for `seconds` behind a body of `kind`
    /// that moves ahead and to its right at 20 metres per second.
    fn left_alone(kind: Kind, seconds: f32) -> f32 {
        let target = Target {
            velocity: Vec3::new(1.0, 0.0, -1.0).normalize() * 20.0,
            kind,
            ..default()
        };
        let moment = Moment {
            target,
            look: Look::default(),
            prefs: ViewPrefs {
                on_foot: ViewMode::Third,
                ..default()
            },
            fov: 1.3,
            dt: 1.0 / 60.0,
        };
        let stance = Stance::of(&target, moment.dt, false);
        let mut aim = Aim::default();
        for _ in 0..(seconds * 60.0) as usize {
            aim.step(&moment, &stance, 1.0);
        }
        aim.heading.yaw
    }

    /// The rig drifts behind the direction of travel of a body on foot once the camera has
    /// been left alone, and never behind a flying body, whose free look is the player module's.
    #[test]
    fn the_rig_does_not_follow_a_flying_body() {
        assert_eq!(left_alone(Kind::OnFoot, 1.0), 0.0);
        assert!(left_alone(Kind::OnFoot, 2.4).abs() > 0.2);
        for seconds in [1.0, 2.4, 10.0] {
            assert_eq!(left_alone(Kind::Flying, seconds), 0.0, "{seconds}");
        }
    }
}
