//! The switch of the headlight in the nose: one key turns it on and off while the pilot sits in
//! the ship. The lamp itself is one of the lights of the hull, see `exhaust::lights`: it burns
//! from what the status display of the cockpit reads, so the ship of another player lights the
//! ground the same way from its record.

use bevy::prelude::*;

use super::ShipEvent;
use super::body::Ship;
use crate::computer::Computer;
use crate::settings::{Persistent, Settings};
use crate::space::{Location, Travel};

/// Returns the headlight after a frame: switched over when its key was `pressed` while the pilot
/// is `seated` and the keyboard is not taken by the board computer, else as it was.
pub fn switched(on: bool, pressed: bool, (seated, taken): (bool, bool)) -> bool {
    on ^ (pressed && seated && !taken)
}

/// Switches the headlight of the ship the player flies on its key, and says so. The ship does
/// not switch it by itself: by day and at night the pilot decides.
pub fn switch(
    (keys, settings): (Res<ButtonInput<KeyCode>>, Res<Persistent<Settings>>),
    location: Res<Location>,
    computer: Option<Res<State<Computer>>>,
    mut ship: Single<&mut Ship>,
    mut events: MessageWriter<ShipEvent>,
) {
    let pressed = keys.just_pressed(settings.controls.keys.ship.headlight);
    let taken = computer.is_some_and(|state| *state.get() == Computer::Open);
    let seated = location.travel == Travel::InShip;
    let next = switched(ship.headlight, pressed, (seated, taken));
    if next != ship.headlight {
        ship.headlight = next;
        events.write(ShipEvent::Headlight(next));
    }
}
