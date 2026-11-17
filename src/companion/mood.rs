//! Mood: which face the surroundings call for, and which changes of face are worth a chirp.

use bevy::prelude::*;

use super::face::Expression;
use super::sense::CompanionSense;
use super::tune::{SLEEPY_NIGHT, TAKEOFF_CLIMB, WORRY_DEPTH};
pub use crate::audio::ChirpKind;

/// Sent when the companion's face changes in a way worth a sound.
#[derive(Message, Debug, Clone, Copy, PartialEq, Eq)]
pub struct CompanionChirp(pub ChirpKind);

/// Returns the face the surroundings call for when no reaction is running.
///
/// From the most to the least urgent: worried under water and in a steep dive, wide-eyed on
/// takeoff and in the clouds, delighted while flying with the player, the face of the idle in
/// progress, sleepy at night, curious near a structure, else neutral.
pub fn ambient(sense: &CompanionSense, idle: Option<Expression>, curious: bool) -> Expression {
    if sense.depth > WORRY_DEPTH || sense.diving {
        Expression::Worried
    } else if sense.in_clouds || sense.climb > TAKEOFF_CLIMB {
        Expression::Surprised
    } else if sense.flying {
        Expression::Happy
    } else if let Some(face) = idle {
        face
    } else if sense.night > SLEEPY_NIGHT {
        Expression::Sleepy
    } else if curious {
        Expression::Curious
    } else {
        Expression::Neutral
    }
}

/// Returns the chirp for a face that just appeared, if it is worth one.
pub fn chirp(expr: Expression) -> Option<ChirpKind> {
    match expr {
        Expression::Neutral => None,
        Expression::Happy => Some(ChirpKind::Happy),
        Expression::Curious => Some(ChirpKind::Curious),
        Expression::Surprised => Some(ChirpKind::Surprised),
        Expression::Sleepy => Some(ChirpKind::Sleepy),
        Expression::Worried => Some(ChirpKind::Worried),
        Expression::Love => Some(ChirpKind::Delight),
    }
}
