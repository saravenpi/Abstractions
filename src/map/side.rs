//! The pins and the players in the side panel of the map. A pin is a row with its icon, its
//! name and how far it is; the selected one shows what it does and three buttons with their
//! words: Go, Rename and Remove. A player can be looked at.

use bevy::prelude::*;
use bevy::ui::RelativeCursorPosition;

use super::pins_draw::glyph;
use super::side_buttons::{button, go_button};
use crate::computer::{LIST_ROW, PIN_FOOT, heading, heading_row, row_node};
use crate::ui::{Icon, Selected, Tip, Tone, column, frame, label, text, theme};

/// Pins listed at most: every pin a player can keep.
pub const PINS: usize = super::places::MAX;

/// Players listed at most.
pub const PLAYERS: usize = 6;

/// What a button of the selected pin does.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Do {
    /// Make it the active pin and, on another planet, fly there.
    Go,
    /// Type a new name for it.
    Rename,
    /// Forget it.
    Remove,
}

/// A button of the selected pin.
#[derive(Component, Debug, Clone, Copy)]
pub struct PinButton(pub Do);

/// The row of the pin at this index, hidden past the end of the list. A click selects it.
#[derive(Component, Debug, Clone, Copy)]
pub struct PinRow(pub usize);

/// The name of the pin at this index.
#[derive(Component, Debug, Clone, Copy)]
pub struct PinName(pub usize);

/// How far the pin at this index is, or the planet it is on.
#[derive(Component, Debug, Clone, Copy)]
pub struct PinFar(pub usize);

/// The line under the pins: what the selected pin does, or the empty text.
#[derive(Component)]
pub struct PinHint;

/// The name of the key that sends the ship to the selected pin, written at the right of Go.
#[derive(Component)]
pub struct PinGoKey;

/// The row of the three buttons of the selected pin, hidden while none is selected.
#[derive(Component)]
pub struct PinActions;

/// The row of the player at this index of the list of friends, hidden past its end.
#[derive(Component, Debug, Clone, Copy)]
pub struct PlayerRow(pub usize);

/// The name of the player at this index.
#[derive(Component, Debug, Clone, Copy)]
pub struct PlayerName(pub usize);

/// The icon of the player at this index, in the colour of that player.
#[derive(Component, Debug, Clone, Copy)]
pub struct PlayerDot(pub usize);

/// Marks the players with their heading, hidden outside a session.
#[derive(Component)]
pub struct Players;

/// Marks the line that says no one else is in the session yet.
#[derive(Component)]
pub struct NoPlayers;

/// What the line of the players says while no one else is in the session.
pub const ALONE: &str = "No one else is here yet.";

/// The node of a row of a list of the panel, hidden until it is written.
fn strip() -> Node {
    Node {
        display: Display::None,
        ..row_node(LIST_ROW)
    }
}

/// One hidden pin: its icon, its name and how far it is. A click selects it.
fn pin(index: usize) -> impl Bundle {
    let name = Node {
        flex_grow: 1.0,
        min_width: px(0),
        overflow: Overflow::clip(),
        ..default()
    };
    let named = (PinName(index), label("", theme::FONT_SMALL, Tone::Plain));
    (
        (PinRow(index), Selected(false)),
        frame(),
        Tip("Select this pin"),
        strip(),
        children![
            glyph(Icon::MapPin, theme::ICON_SMALL, theme::PIN),
            (name, children![named]),
            (PinFar(index), label("", theme::FONT_SMALL, Tone::Muted)),
        ],
    )
}

/// The foot of the pins: the line that says what the selected pin does, and under it the row of
/// its three buttons, hidden while no pin is selected.
fn foot() -> impl Bundle {
    let node = Node {
        width: percent(100),
        min_height: px(PIN_FOOT),
        flex_direction: FlexDirection::Column,
        row_gap: px(theme::GAP_XS),
        ..default()
    };
    let actions = Node {
        display: Display::None,
        flex_wrap: FlexWrap::Wrap,
        column_gap: px(theme::GAP_S),
        row_gap: px(theme::GAP_XS),
        ..default()
    };
    let buttons = children![
        go_button(),
        button(Do::Rename, ("Rename", "R"), "Type a new name"),
        button(Do::Remove, ("Remove", "Del"), "Forget this pin"),
    ];
    let said = (PinHint, text("", theme::FONT_SMALL, theme::TEXT_2));
    (node, children![said, (PinActions, actions, buttons)])
}

/// The pins: their heading over the pool of rows, and under them their foot.
pub fn pins() -> impl Bundle {
    let rows = Children::spawn(SpawnIter((0..PINS).map(pin)));
    (
        column(percent(100), 0.0),
        children![
            (heading_row(), children![heading(Icon::MapPin, "Pins")]),
            (column(percent(100), 0.0), rows),
            foot(),
        ],
    )
}

/// One hidden player: the icon of a player in their colour and their name.
fn player(index: usize) -> impl Bundle {
    (
        frame(),
        (PlayerRow(index), RelativeCursorPosition::default()),
        Tip("Look at this player"),
        strip(),
        children![
            (
                PlayerDot(index),
                glyph(Icon::User, theme::ICON_SMALL, theme::TEXT)
            ),
            (PlayerName(index), label("", theme::FONT_SMALL, Tone::Plain)),
        ],
    )
}

/// The players: their heading over the pool of rows and the line for when no one else is
/// there, hidden outside a session.
pub fn players() -> impl Bundle {
    let block = Node {
        display: Display::None,
        width: percent(100),
        flex_direction: FlexDirection::Column,
        ..default()
    };
    let alone = Node {
        display: Display::None,
        ..default()
    };
    (
        Players,
        block,
        children![
            (heading_row(), children![heading(Icon::Users, "Players")]),
            (
                NoPlayers,
                alone,
                text(ALONE, theme::FONT_SMALL, theme::TEXT_2)
            ),
            (
                column(percent(100), 0.0),
                Children::spawn(SpawnIter((0..PLAYERS).map(player)))
            )
        ],
    )
}
