//! The key of an instrument of the ship readout: the name of the key that switches it, after its
//! value. Only the headlight has one today. A key of the HUD is in `TEXT`, like the rest of
//! the HUD: with no panel behind it the secondary tone is too faint over a bright sky.

use bevy::prelude::*;

use super::flight::{Flight, Gauge};
use super::panel::Source;
use crate::ui::{KeyName, Tone, label, theme};

/// Marks the name of the key of an instrument, hidden for an instrument no key switches.
#[derive(Component, Debug, Clone, Copy)]
pub(super) struct KeyOf(Gauge);

/// Returns the name of the key that switches an instrument, or nothing for one no key does.
pub(super) fn key(gauge: Gauge, flight: &Flight) -> &str {
    match (gauge, &flight.headlight) {
        (Gauge::Headlight, Some((_, key))) => key.as_str(),
        _ => "",
    }
}

/// The name of a key on the HUD, empty until it is written: in `TEXT`, set apart from the words
/// before it by the gap alone.
pub(super) fn cap() -> impl Bundle {
    (KeyName, label("", theme::FONT_SMALL, Tone::Plain))
}

/// The hidden room of the key of the instrument `gauge`, at the end of its row.
pub(super) fn slot(gauge: Gauge) -> impl Bundle {
    let apart = Node {
        display: Display::None,
        margin: UiRect::left(px(theme::S2)),
        ..default()
    };
    (KeyOf(gauge), apart, cap())
}

/// Writes the name of the key of every instrument that has one and hides the room of the others.
pub(super) fn write(source: Source, mut keys: Query<(&KeyOf, &mut Text, &mut Node)>) {
    let flight = source.flight();
    for (KeyOf(gauge), mut text, mut node) in &mut keys {
        let name = key(*gauge, &flight);
        let display = [Display::Flex, Display::None][usize::from(name.is_empty())];
        if text.0 != name {
            text.0 = name.to_string();
        }
        if node.display != display {
            node.display = display;
        }
    }
}
