//! Intents: what a player asks of its body, as a message the rules read instead of the keys.
//!
//! The input code turns the keyboard and the mouse of this machine into intents of
//! [`super::PlayerId::LOCAL`]; a remote player's intents are the same messages from another
//! source. New kinds are added at the end, so older senders still read.

use bevy::prelude::*;

/// One thing a player asks for during a step. Held keys are sent on every step they are held.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Intent {
    /// Walk, swim or steer: forward and strafe to the right, each from -1 to 1.
    Move {
        /// Forward, negative backward.
        forward: f32,
        /// To the right, negative to the left.
        strafe: f32,
    },
    /// Where the player looks this step.
    Look {
        /// Heading of the view in radians around the local up.
        yaw: f32,
        /// Pitch of the view in radians, positive upward.
        pitch: f32,
        /// How far the heading was turned this step before the camera rig took it, which the
        /// movement keys follow while they stay held: the mouse, and the drift that brings
        /// the view back behind a body on its wings. What the rig adds afterwards by itself,
        /// such as its drift behind the direction of travel, is not in it.
        turn: f32,
        /// Whether the player sees the body from behind, so that it turns to where it goes.
        behind: bool,
    },
    /// Jump, swim up, or fire the boots when asked again in the air. Sent while held.
    Jump,
    /// Run, or fire the boots with the wingsuit spread. Sent while held.
    Boost,
    /// Spread or fold the wingsuit. Sent on the step the key goes down.
    Wings,
    /// Take the seat of the ship.
    Board,
    /// Leave the seat of the ship.
    Leave,
    /// Use what is in front of the body.
    Interact,
}

/// An intent of one player, as a message.
#[derive(Message, Debug, Clone, Copy, PartialEq)]
pub struct IntentOf {
    /// The player who asks.
    pub player: super::PlayerId,
    /// What the player asks.
    pub intent: Intent,
}
