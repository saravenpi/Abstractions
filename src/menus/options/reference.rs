//! The controls reference: every action on foot and in the ship with its current key. Its lines
//! are laid out as the rows of the other tabs are: the action on the label line, the key at the
//! left of the value zone.

use bevy::prelude::*;

use super::Tab;
use super::field::{Bind, Field};
use super::pages::page;
use super::rows::value;
use crate::ui::{Icon, Tone, cell, entry, rows, section, sign, text, theme};

/// Most characters an action of the reference may have to stay on one line.
#[cfg(test)]
pub const ACTION_CHARS: usize = 15;

/// One line: what is done, and the key that does it.
fn line(key: impl Bundle, action: &str) -> impl Bundle {
    let node = Node {
        width: percent(100),
        min_height: px(theme::LINE_SMALL),
        flex_shrink: 0.0,
        align_items: AlignItems::Center,
        justify_content: JustifyContent::SpaceBetween,
        column_gap: px(theme::S3),
        padding: UiRect::horizontal(px(theme::INSET)),
        ..default()
    };
    let said = entry(None::<Icon>, action, theme::FONT_SMALL, Tone::Muted);
    (node, children![said, cell(theme::VALUE_CLUSTER, key)])
}

/// One line for a rebindable action, whose key follows the settings.
fn bound(bind: Bind) -> impl Bundle {
    line(value(Field::Key(bind), theme::FONT_SMALL), bind.label())
}

/// One line for something that cannot be rebound.
fn fixed(key: &str, action: &str) -> impl Bundle {
    line(text(key, theme::FONT_SMALL, theme::TEXT), action)
}

/// What the mouse and held keys do on foot, which no row can rebind.
const FOOT_FIXED: [(&str, &str); 5] = [
    ("Mouse", "Look around"),
    ("Wheel", "Zoom camera"),
    ("Jump in air", "Wings out or in"),
    ("Sprint, fall", "Boots brake"),
    ("Sprint, wings", "Boost"),
];

/// What the mouse does in the ship, and the home key: home through the orbital cruise near the
/// home planet, a course to it from afar.
const SHIP_FIXED: [(&str, &str); 3] = [
    ("Mouse", "Look around"),
    ("Wheel", "Zoom camera"),
    ("H", "Fly home"),
];

/// The lines of one half of the reference: the fixed lines, the rebindable ones, then the two
/// keys that work everywhere.
fn lines(
    fixed_lines: &'static [(&'static str, &'static str)],
    binds: &'static [Bind],
) -> impl Bundle {
    Children::spawn((
        SpawnIter(fixed_lines.iter().map(|(key, action)| fixed(key, action))),
        SpawnIter(binds.iter().copied().map(bound)),
        Spawn(fixed("F1", "Hide interface")),
        Spawn(fixed("Esc", "Pause, go back")),
    ))
}

/// Help tab: the controls on foot, then those of the ship.
pub fn help(active: Tab) -> impl Bundle {
    let foot = (rows(), lines(&FOOT_FIXED, &Bind::FOOT));
    let ship = (rows(), lines(&SHIP_FIXED, &Bind::SHIP));
    let content = children![
        section(sign::ON_FOOT, "On foot", foot),
        section(Icon::ShipParked, "In the ship", ship),
    ];
    page(Tab::Help, active, content)
}
