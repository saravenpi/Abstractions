//! The movement state the controller owns for each body.

use bevy::prelude::*;

use super::steer::{Steer, Turn};

/// Movement state owned by the controller.
#[derive(Component, Debug, Default, Clone, Copy)]
pub struct Motion {
    /// Current velocity in the upright frame of the body: +Y is the local up.
    pub velocity: Vec3,
    /// Heading of the body in radians around the local up. The body orientation follows it. In
    /// first person it is the heading of the view; in third person the body turns to where it goes.
    pub yaw: f32,
    /// Whether walkable ground was touched on the last step.
    pub grounded: bool,
    /// Remaining seconds in which a jump is still allowed.
    pub coyote: f32,
    /// Whether the body is swimming instead of walking.
    pub swimming: bool,
    /// How far the body is crouched, from 0 (standing) to 1 (fully down).
    pub crouch: f32,
    /// Whether the wingsuit is asked open. Only the air keeps it: landing and water fold it.
    pub wings: bool,
    /// Whether the boots are fired: the run key is held in the air, pressed there, on the wings
    /// or in a fast fall. A run that only carries on through a jump never fires them.
    pub boots: bool,
    /// Whether the held jump key already worked the gear in the air, and so starts no jump
    /// until let go.
    pub spent: bool,
    /// The heading the movement keys are read against.
    pub steer: Steer,
    /// How the body turns to its heading.
    pub turn: Turn,
    /// How far the eyes trail the body after a step-up, in the upright frame. It fades away.
    pub lag: Vec3,
    /// Frames in a row the flight gear has carried the body through the air with walkable
    /// ground under its soles. Past `flight::WATCH` the body is on the ground, whatever the
    /// gap: no body stays falling on the ground.
    pub aground: u8,
    /// Seconds left of the sling that puts the tool on the back before the asked wings spread,
    /// see `asks::SLING`. The gear is told to spread them once it is over.
    pub sling: f32,
}
