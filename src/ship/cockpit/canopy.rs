//! The canopy of the ship: the glass and its frame, hinged across the back of the mouth. It
//! lifts at the front while the player on foot stands at the hatch of the landed ship, which is
//! how a one-seat craft is boarded, and it shuts again once the player sits.

use bevy::prelude::*;

use super::tune::CANOPY_OPEN;
use crate::ship::board::Hatch;
use crate::ship::body::Ship;

/// The hinge of the canopy of one ship, a child of the ship. The glass and the frame hang
/// from it, so turning it opens both.
#[derive(Component, Debug, Default, Clone, Copy, PartialEq)]
pub struct Canopy {
    /// How far the canopy is open, from 0 (shut) to [`CANOPY_OPEN`] radians.
    pub angle: f32,
}

/// Returns the angle of a canopy that is at `now` and is asked to be `open`, `dt` seconds later.
pub fn swung(now: f32, open: bool, dt: f32) -> f32 {
    let wanted = if open { CANOPY_OPEN.0 } else { 0.0 };
    let step = CANOPY_OPEN.1 * dt;
    now + (wanted - now).clamp(-step, step)
}

/// Swings the canopy of the ship the player flies open while the player stands at its hatch,
/// and shuts it otherwise. The canopy of any other ship stays shut. A canopy at rest writes
/// nothing.
pub fn swing(
    time: Res<Time>,
    hatch: Option<Res<Hatch>>,
    ships: Query<(), With<Ship>>,
    mut hinges: Query<(&ChildOf, &mut Canopy, &mut Transform)>,
) {
    let near = hatch.is_some_and(|hatch| hatch.near);
    for (parent, mut canopy, mut transform) in &mut hinges {
        let open = near && ships.contains(parent.parent());
        let angle = swung(canopy.angle, open, time.delta_secs());
        if angle != canopy.angle {
            canopy.angle = angle;
            transform.rotation = Quat::from_rotation_x(angle);
        }
    }
}
