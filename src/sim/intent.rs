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
    /// Jump or swim up. Asked again in the air it spreads or folds the wingsuit. Sent while held.
    Jump,
    /// Run. In the air it fires the boots: a brake with the wingsuit folded, the boost with it
    /// spread. Sent while held.
    Boost,
    /// Crouch: lower the body and walk slowly, on the ground only. Sent while held.
    Crouch,
    /// Spread or fold the wingsuit, as a jump asked again in the air does. Sent on the step the
    /// key goes down.
    Wings,
    /// Take the seat of the ship.
    Board,
    /// Leave the seat of the ship.
    Leave,
    /// Use what is in front of the body.
    Interact,
    /// Turn flat on the wingsuit, from -1 (left) to 1 (right). Sent while held.
    Yaw(f32),
    /// Let the view steer the wingsuit. Sent while the steer button is held.
    Steer,
}

/// An intent of one player, as a message.
#[derive(Message, Debug, Clone, Copy, PartialEq)]
pub struct IntentOf {
    /// The player who asks.
    pub player: super::PlayerId,
    /// What the player asks.
    pub intent: Intent,
}
