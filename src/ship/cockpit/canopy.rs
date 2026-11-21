//! The canopy of the ship: the glass and its frame, hinged across the back of the mouth. It
//! lifts at the front while the player on foot stands at the hatch of the landed ship, which is
//! how a one-seat craft is boarded, and it shuts again once the player sits.

use bevy::prelude::*;

use super::state::Cockpit;
use super::tune::CANOPY_OPEN;

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

/// Swings the canopy of every ship toward what its cockpit asks: open while the owner of the
/// ship stands at its hatch, shut otherwise. The ship the player flies is asked by its hatch,
/// the ship of another player by its record. A canopy at rest writes nothing.
pub fn swing(
    time: Res<Time>,
    cockpits: Query<&Cockpit>,
    mut hinges: Query<(&ChildOf, &mut Canopy, &mut Transform)>,
) {
    for (parent, mut canopy, mut transform) in &mut hinges {
        let open = cockpits
            .get(parent.parent())
            .is_ok_and(|cockpit| cockpit.open);
        let angle = swung(canopy.angle, open, time.delta_secs());
        if angle != canopy.angle {
            canopy.angle = angle;
            transform.rotation = Quat::from_rotation_x(angle);
        }
    }
}
