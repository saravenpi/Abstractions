//! The controls reference: every action on foot and in the ship with its current key.

use bevy::prelude::*;

use super::Tab;
use super::field::{Bind, Field};
use super::pages::page;
use super::rows::value;
use crate::ui::{caps, cell, column, text, theme};

/// Width of the key column of the reference, in pixels.
const KEY_WIDTH: f32 = 120.0;

/// Most characters an action of the reference may have to stay on one line.
#[cfg(test)]
pub const ACTION_CHARS: usize = 19;

/// Width of one half of the reference, in pixels.
const HALF_WIDTH: f32 = 288.0;

/// One line: a key and what it does.
fn line(key: impl Bundle, action: &str) -> impl Bundle {
    (
        Node {
            column_gap: px(theme::GAP_S),
            ..default()
        },
        children![
            cell(KEY_WIDTH, key),
            text(action, theme::FONT_SMALL, theme::MUTED)
        ],
    )
}

/// One line for a rebindable action, whose key follows the settings.
fn bound(bind: Bind) -> impl Bundle {
    line(value(Field::Key(bind), theme::FONT_SMALL), bind.label())
}

/// One line for something that cannot be rebound.
fn fixed(key: &str, action: &str) -> impl Bundle {
    line(text(key, theme::FONT_SMALL, theme::TEXT), action)
}

/// One half of the reference: a title, the fixed lines and the rebindable ones.
fn half(title: &str, mouse: &str, binds: &'static [Bind]) -> impl Bundle {
    (
        column(px(HALF_WIDTH), 0.0),
        Children::spawn((
            Spawn(caps(title)),
            Spawn(fixed("Mouse", mouse)),
            SpawnIter(binds.iter().copied().map(bound)),
            Spawn(fixed("Esc", "Pause or go back")),
        )),
    )
}

/// Help tab: the controls on foot and in the ship, side by side.
pub fn help(active: Tab) -> impl Bundle {
    (
        page(Tab::Help, active),
        children![(
            Node {
                column_gap: px(theme::GAP_L),
                flex_wrap: FlexWrap::Wrap,
                row_gap: px(theme::GAP_M),
                ..default()
            },
            children![
                half("On foot", "Look around", &Bind::FOOT),
                half("In the ship", "Pitch and roll", &Bind::SHIP),
            ],
        )],
    )
}
