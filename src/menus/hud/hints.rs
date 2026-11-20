//! First-time hints: at most four in the whole game, shown one at a time on the prompt line,
//! only while nothing with a higher priority shows. A hint never comes back once its action
//! is done, and stays at most [`HOLD`] seconds in a session. They are kept in the `hints` bitset
//! of the settings.

use bevy::prelude::*;

use super::context::{Context, Element, Where, shown};
use crate::settings::{Keymap, Persistent, Settings};

/// Seconds a hint stays on the line before it is given up for the session.
pub const HOLD: f32 = 10.0;

/// Seconds seated in the ship before the map hint shows.
const MAP_AFTER: f32 = 5.0;

/// Seconds seated in the ship before the stabiliser hint shows.
const ASSIST_AFTER: f32 = 15.0;

/// Seconds on the ground before the minimap range hint shows.
const MINIMAP_AFTER: f32 = 10.0;

/// Seconds on the ground before the pin hint shows.
const PIN_AFTER: f32 = 60.0;

/// One first-time hint.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Hint {
    /// Open the map.
    Map,
    /// Drop a pin.
    Pin,
    /// Step the minimap range.
    Minimap,
    /// The stabiliser key of the ship.
    Assist,
}

impl Hint {
    /// Every hint, in the order they are offered.
    pub const ALL: [Hint; 4] = [Hint::Map, Hint::Assist, Hint::Minimap, Hint::Pin];

    /// Returns the bit of the hint in the settings.
    pub const fn bit(self) -> u8 {
        match self {
            Hint::Map => 1,
            Hint::Pin => 2,
            Hint::Minimap => 4,
            Hint::Assist => 8,
        }
    }

    /// Returns the words of the hint, after its key.
    pub const fn words(self) -> &'static str {
        match self {
            Hint::Map => "Open the map",
            Hint::Pin => "Drop a pin",
            Hint::Minimap => "Minimap range",
            Hint::Assist => "Stabiliser",
        }
    }

    /// Returns the key that does what the hint says.
    pub fn key(self, keys: &Keymap) -> KeyCode {
        match self {
            Hint::Map => keys.map,
            Hint::Pin => keys.waypoint,
            Hint::Minimap => keys.minimap,
            Hint::Assist => keys.ship.assist,
        }
    }
}

/// The hint the line shows now, if any.
#[derive(Resource, Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct Hinted(pub Option<Hint>);

/// Time spent in each place this session, and what was given up.
#[derive(Resource, Debug, Default, Clone, Copy, PartialEq)]
pub struct Clock {
    /// Seconds seated in the ship.
    pub seated: f32,
    /// Seconds on the ground, on foot or seated.
    pub afoot: f32,
    /// Seconds the current hint has been on the line.
    pub shown: f32,
    /// Bits of the hints given up for this session.
    pub hushed: u8,
}

/// Returns whether the hint's action was done once.
pub const fn done(bits: u8, hint: Hint) -> bool {
    bits & hint.bit() != 0
}

/// Returns `bits` with the hint's action marked done.
pub const fn with(bits: u8, hint: Hint) -> u8 {
    bits | hint.bit()
}

/// Returns the first hint worth showing in `context`: not done, not given up, and its moment
/// has come.
pub fn due(bits: u8, clock: &Clock, context: Context) -> Option<Hint> {
    let seated = matches!(
        context,
        Context::ShipLanded | Context::ShipAir | Context::ShipOrbit | Context::ShipSpace
    );
    let landed = matches!(context, Context::ShipLanded | Context::ShipAir);
    let minimap = shown(context).contains(Element::Minimap);
    Hint::ALL.into_iter().find(|hint| {
        let open = !done(bits, *hint) && !done(clock.hushed, *hint);
        open && match hint {
            Hint::Map => seated && clock.seated >= MAP_AFTER,
            Hint::Assist => landed && clock.seated >= ASSIST_AFTER,
            Hint::Minimap => minimap && clock.afoot >= MINIMAP_AFTER,
            Hint::Pin => {
                context == Context::OnFoot && done(bits, Hint::Map) && clock.afoot >= PIN_AFTER
            }
        }
    })
}

/// Returns the hint a pressed key finishes: the map hint also ends with the computer key.
pub fn finished(keys: &Keymap, pressed: impl Fn(KeyCode) -> bool) -> Option<Hint> {
    Hint::ALL
        .into_iter()
        .find(|hint| pressed(hint.key(keys)) || (*hint == Hint::Map && pressed(keys.computer)))
}

/// Counts the time spent in each place, picks the hint to show, and writes the bit of one the
/// player just did.
pub(super) fn update(
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
    let bits = settings.hints;
    if let Some(hint) = finished(&settings.controls.keys, |key| keys.just_pressed(key))
        && !done(bits, hint)
    {
        let marked = with(bits, hint);
        if let Err(error) = settings.update(|settings| settings.hints = marked) {
            warn!("could not save the hints: {error}");
        }
    }
    let wanted = due(settings.hints, &clock, context);
    if wanted == hinted.0 {
        clock.shown += dt * f32::from(u8::from(wanted.is_some()));
        if let Some(hint) = wanted.filter(|_| clock.shown >= HOLD) {
            clock.hushed |= hint.bit();
        }
        return;
    }
    clock.shown = 0.0;
    hinted.0 = wanted;
}
