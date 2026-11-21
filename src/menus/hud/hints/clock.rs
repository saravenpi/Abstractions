//! The clock of the first-time hints and the hints done: the time spent in each place this
//! session, the time each hint has shown, and the bit of a hint written to the settings once its
//! action is done, by its key, or for the pin hint by a pin dropped in any way.

use bevy::prelude::*;

use super::{HOLD, Hint, Hinted, done, due, finished, with};
use crate::map::Places;
use crate::menus::hud::context::{Element, Where, shown};
use crate::settings::{Persistent, Settings};

/// Time spent in each place this session, and what was given up.
#[derive(Resource, Debug, Default, Clone, Copy, PartialEq)]
pub struct Clock {
    /// Seconds seated in the ship.
    pub seated: f32,
    /// Seconds on the ground, on foot or seated.
    pub afoot: f32,
    /// Seconds each hint has been on the line this session, in all, by the rank of its bit.
    pub shown: [f32; 4],
    /// Bits of the hints given up for this session.
    pub hushed: u8,
}

impl Clock {
    /// Counts `dt` seconds of `hint` on the line, and gives it up for the session once it has
    /// shown [`HOLD`] seconds in all: leaving the place it shows in and coming back, or opening
    /// the map and closing it, does not start its time again.
    pub fn tick(&mut self, hint: Option<Hint>, dt: f32) {
        let Some(hint) = hint else {
            return;
        };
        let spent = &mut self.shown[hint.bit().trailing_zeros() as usize];
        *spent += dt;
        if *spent >= HOLD {
            self.hushed |= hint.bit();
        }
    }
}

/// Writes `hint` as done in the settings, unless it is already: a hint done leaves the settings
/// untouched, so nothing that watches them wakes.
fn mark(settings: &mut ResMut<Persistent<Settings>>, hint: Hint) {
    if done(settings.hints, hint) {
        return;
    }
    let marked = with(settings.hints, hint);
    if let Err(error) = settings.update(|settings| settings.hints = marked) {
        warn!("could not save the hints: {error}");
    }
}

/// Counts the time spent in each place, picks the hint to show, and writes the bit of one the
/// player just did.
pub(in crate::menus::hud) fn update(
    (time, keys, now): (Res<Time>, Res<ButtonInput<KeyCode>>, Res<Where>),
    mut settings: ResMut<Persistent<Settings>>,
    (mut clock, mut hinted): (ResMut<Clock>, ResMut<Hinted>),
) {
    let (dt, context) = (time.delta_secs(), now.0);
    if shown(context).contains(Element::Cardinals) {
        clock.afoot += dt;
    }
    if context.is_ship() {
        clock.seated += dt;
    }
    if let Some(hint) = finished(&settings.controls.keys, |key| keys.just_pressed(key)) {
        mark(&mut settings, hint);
    }
    let wanted = due(settings.hints, &clock, context);
    clock.tick(wanted, dt);
    hinted.set_if_neq(Hinted(wanted));
}

/// Marks the pin hint done once the player has a pin, however it came: the pin key on foot or
/// with the map open, a right click on the map, or a pin kept in the world. The player knows how
/// to drop one, so the hint never shows again, in this session or the next.
pub(in crate::menus::hud) fn retire(
    places: Option<Res<Places>>,
    mut settings: ResMut<Persistent<Settings>>,
) {
    if places.is_some_and(|places| !places.pins.is_empty()) {
        mark(&mut settings, Hint::Pin);
    }
}
