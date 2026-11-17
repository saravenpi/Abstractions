//! Which way the movement keys point and which way the body faces.
//!
//! The keys are read against a heading of their own. It is the heading of the camera when the
//! held keys last changed, turned since by the player's own hand only: what the camera does by
//! itself behind a walking body does not bend the walk. In first person nothing but the hand
//! turns the camera, so there the keys simply follow the view.
//!
//! The body turns to its heading with a critically damped, rate limited turn, so it never
//! snaps and never swings past.

use std::f32::consts::{PI, TAU};

use bevy::prelude::*;

/// Natural frequency of the turn of the body, in radians per second.
pub(super) const TURN_RATE: f32 = 14.0;

/// Fastest the body turns, in radians per second.
pub(super) const MAX_TURN: f32 = 10.0;

/// Angle under which a body that follows its heading rigidly counts as on it.
const SETTLED: f32 = 1.0e-4;

/// Returns `angle` wrapped to the half turn around zero.
pub(crate) fn wrap(angle: f32) -> f32 {
    (angle + PI).rem_euclid(TAU) - PI
}

/// Returns the heading of the upright direction `flat`, or nothing when it has none.
pub(super) fn heading_of(flat: Vec3) -> Option<f32> {
    (flat.x * flat.x + flat.z * flat.z > 1.0e-6).then(|| (-flat.x).atan2(-flat.z))
}

/// The heading the movement keys are read against.
#[derive(Debug, Default, Clone, Copy, PartialEq)]
pub struct Steer {
    /// Heading in radians around the local up.
    pub yaw: f32,
    /// The forward and right keys held on the last frame.
    held: Vec2,
}

impl Steer {
    /// Returns the heading for this frame: the `view` heading while no key is held and when the
    /// held keys `axes` change, else the last one turned by `turned`, the player's own turn.
    pub(super) fn step(&mut self, view: f32, turned: f32, axes: Vec2) -> f32 {
        self.yaw = match axes == Vec2::ZERO || axes != self.held {
            true => view,
            false => self.yaw + turned,
        };
        self.held = axes;
        self.yaw
    }
}

/// What the body turns to.
#[derive(Debug, Default, Clone, Copy, PartialEq)]
pub enum Lead {
    /// Nothing: the body keeps its heading.
    #[default]
    Hold,
    /// The view, which the body follows rigidly: first person.
    View(f32),
    /// The nose of the flight gear, which the body follows rigidly.
    Nose(f32),
    /// The direction of travel, which the body turns to.
    Travel(f32),
}

impl Lead {
    /// Returns the heading to turn to and whether the body follows it rigidly once on it.
    fn target(self) -> Option<(f32, bool)> {
        match self {
            Lead::Hold => None,
            Lead::View(yaw) | Lead::Nose(yaw) => Some((yaw, true)),
            Lead::Travel(yaw) => Some((yaw, false)),
        }
    }

    /// Returns true when `self` and `other` lead the same way, whatever their headings.
    fn same(self, other: Lead) -> bool {
        std::mem::discriminant(&self) == std::mem::discriminant(&other)
    }
}

/// How the body turns to its heading.
#[derive(Debug, Default, Clone, Copy, PartialEq)]
pub struct Turn {
    /// What it turned to on the last frame.
    lead: Lead,
    /// How far the body is from the heading it turns to, in radians.
    gap: f32,
    /// Rate of that gap in radians per second.
    spin: f32,
}

impl Turn {
    /// Returns the heading of a body at `yaw` after `dt` seconds of turning to `lead`.
    ///
    /// A rigid lead carries its gap from frame to frame, so the body stays exactly on a moving
    /// view or nose once it is there. A direction of travel is measured again every frame.
    pub(super) fn step(&mut self, yaw: f32, lead: Lead, dt: f32) -> f32 {
        let Some((target, rigid)) = lead.target() else {
            *self = Self::default();
            return yaw;
        };
        let kept = rigid && self.lead.same(lead);
        let (gap, spin) = match kept {
            true => (self.gap, self.spin),
            false => (
                wrap(yaw - target),
                self.spin * f32::from(self.lead.same(lead)),
            ),
        };
        let decay = (-TURN_RATE * dt).exp();
        let pull = spin + gap * TURN_RATE;
        let step = ((gap + pull * dt) * decay - gap).clamp(-MAX_TURN * dt, MAX_TURN * dt);
        let on = rigid && (gap + step).abs() < SETTLED;
        self.gap = if on { 0.0 } else { gap + step };
        self.spin = match on {
            true => 0.0,
            false => ((spin - pull * TURN_RATE * dt) * decay).clamp(-MAX_TURN, MAX_TURN),
        };
        self.lead = lead;
        target + self.gap
    }
}
