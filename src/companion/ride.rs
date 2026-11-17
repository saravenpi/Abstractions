//! Boarding and leaving the ship with the player: the companion becomes a child of the player
//! body, which sits in the pilot seat, and back.
//!
//! It rides with the seat, not with the camera, so it sits in the same place in the cockpit
//! whether the view is from the seat or from behind the ship.

use bevy::prelude::*;

use super::Companion;
use super::motion::Float;
use super::sense::CompanionSense;
use super::tune::BOARD_REACH;
use crate::player::Player;
use crate::space::Rebased;

/// Returns the pose in the space of `eye` of a body at the render pose `at`, pulled within reach of the seat.
pub fn boarding(eye: &Transform, at: &Transform) -> Transform {
    let turn = eye.rotation.inverse();
    Transform {
        translation: (turn * (at.translation - eye.translation)).clamp_length_max(BOARD_REACH),
        rotation: turn * at.rotation,
        scale: at.scale,
    }
}

/// Returns the render pose of a body at the pose `at` in the space of `eye`.
pub fn leaving(eye: &Transform, at: &Transform) -> Transform {
    Transform {
        translation: eye.translation + eye.rotation * at.translation,
        rotation: eye.rotation * at.rotation,
        scale: at.scale,
    }
}

/// Filter of the player body, which carries the seat, and is not the companion.
type Seat = (With<Player>, Without<Companion>);

/// Attaches the companion to the player body when the player sits in the ship, and frees it
/// when the player leaves.
///
/// Seated, it rides in the space of the seat and the anchor no longer moves it. On leaving it
/// keeps the render pose it had in the cockpit, so it pops out of the ship and follows the
/// player again.
pub fn switch(
    mut commands: Commands,
    sense: Res<CompanionSense>,
    seat: Single<(Entity, &Transform), Seat>,
    bot: Single<(Entity, &mut Transform, &mut Float), With<Companion>>,
) {
    let (entity, mut at, mut float) = bot.into_inner();
    if sense.seated == float.seated {
        return;
    }
    float.seated = sense.seated;
    let (seat, eye) = (seat.0, *seat.1);
    if sense.seated {
        *at = boarding(&eye, &at);
        float.vel = eye.rotation.inverse() * float.vel;
        commands
            .entity(entity)
            .insert(ChildOf(seat))
            .remove::<Rebased>();
    } else {
        *at = leaving(&eye, &at);
        float.vel = eye.rotation * float.vel;
        commands.entity(entity).remove::<ChildOf>().insert(Rebased);
    }
}
