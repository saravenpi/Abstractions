//! Where the player looks, and what the body turns to in each kind of movement.

use bevy::prelude::*;

use super::controller::Motion;
use super::intent::Intent;
use super::steer::{Lead, heading_of};
use crate::gear::Gear;
use crate::view::tune::PITCH_LIMIT;
use crate::view::{Heading, Look, ViewMode};

/// Where the player looks this frame, in the upright frame of the body.
#[derive(Debug, Clone, Copy, PartialEq)]
pub(super) struct View {
    /// Heading of the view in radians around the local up.
    pub(super) yaw: f32,
    /// Pitch of the view in radians, positive upward.
    pub(super) pitch: f32,
    /// Whether the body is seen from behind: it then turns to where it goes, not to the view.
    pub(super) third: bool,
}

impl View {
    /// Returns the view the camera rig will show this frame: the angles it holds, turned by
    /// what the mouse asks, which the rig applies after the controller ran. A view that is
    /// already past the vertical, as in a loop on the wings, is left where it is.
    pub(super) fn ahead(heading: &Heading, look: &Look, mode: ViewMode) -> Self {
        let pitch = heading.pitch + look.pitch_delta;
        let bound = PITCH_LIMIT.max(heading.pitch.abs());
        Self {
            yaw: heading.yaw + look.yaw_delta,
            pitch: pitch.clamp(-bound, bound),
            third: mode == ViewMode::Third,
        }
    }

    /// Returns the unit direction the view looks along.
    pub(super) fn direction(&self) -> Vec3 {
        Quat::from_rotation_y(self.yaw) * Quat::from_rotation_x(self.pitch) * Vec3::NEG_Z
    }
}

/// Returns what the body turns to this frame.
///
/// In the air under the gear it is the nose of the flight. Sliding and rolling it is the way
/// the body goes. On foot and in water it is the view in first person; in third person it is
/// the way the keys push, and nothing while they are let go, so the camera orbits a body that
/// stands.
pub(super) fn lead(motion: &Motion, gear: &Gear, intent: &Intent, view: &View) -> Lead {
    let state = &gear.state;
    if state.mode.airborne() {
        return Lead::Nose(state.servo.yaw);
    }
    if state.mode.drives() {
        return heading_of(motion.velocity).map_or(Lead::Hold, Lead::Travel);
    }
    if !view.third {
        return Lead::View(view.yaw);
    }
    heading_of(intent.wish).map_or(Lead::Hold, Lead::Travel)
}
