//! The hook of the wingsuit: one message each time a body asks its wings open or folds them.

use bevy::prelude::*;

/// A body asked its wingsuit open or folded it, on the frame it did.
///
/// The controller sends it as [`super::Motion::wings`] changes: on the press that opens them,
/// the frame they start to spread over `gear::tune::WING_TIME`, and on the press, the landing or
/// the water that folds them. Whatever the hands hold must be put away before the wings take
/// them: read it after [`super::PlayerSet::Move`] in the same frame.
#[derive(Message, Debug, Clone, Copy, PartialEq, Eq)]
pub struct Spread {
    /// The player body whose wings changed.
    pub body: Entity,
    /// True when the wings are asked open, false when they fold.
    pub open: bool,
}

/// Sends a [`Spread`] for `body` into `messages` when its wings went from `was` to `now`.
/// Without the message registered, as in a bare test world, nothing is sent.
pub(super) fn tell(
    messages: Option<&mut Messages<Spread>>,
    body: Entity,
    (was, now): (bool, bool),
) {
    if let Some(messages) = messages.filter(|_| was != now) {
        messages.write(Spread { body, open: now });
    }
}
