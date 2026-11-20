//! A blow that lands on the robot, sent by whatever hit it.

use bevy::prelude::*;

/// One hit on the robot: where it landed, how much energy it carried and whether it cut.
#[derive(Message, Clone, Copy, Debug)]
pub struct Blow {
    /// The contact point in world space.
    pub at: Vec3,
    /// The energy delivered, in joules.
    pub joules: f32,
    /// True when an edge or point carried the hit, false when it was blunt.
    pub cut: bool,
}
