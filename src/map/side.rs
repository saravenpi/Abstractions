//! The favourites and the players in the side panel of the map. A favourite can be looked at,
//! set as the course, renamed and deleted; a player can be looked at. The button over the
//! favourites saves the place picked on the flat map, or where the player is.

use bevy::prelude::*;

use crate::ui::{Icon, Tone, column, frame, icon_slot, label, step_button, text, theme};

/// Favourites listed at most.
pub const FAVOURITES: usize = 5;

/// Players listed at most.
pub const PLAYERS: usize = 6;

/// What a button of a favourite does.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Do {
    /// Look at it on the flat map.
    Go,
    /// Set the course to its planet.
    Course,
    /// Type a new name for it.
    Rename,
    /// Forget it.
    Delete,
}

/// A button of the favourite at this index of the list.
#[derive(Component, Debug, Clone, Copy)]
pub struct FavButton(pub usize, pub Do);

/// The row of the favourite at this index, hidden past the end of the list.
#[derive(Component, Debug, Clone, Copy)]
pub struct FavRow(pub usize);

/// The name of the favourite at this index.
#[derive(Component, Debug, Clone, Copy)]
pub struct FavName(pub usize);

/// The button that saves a favourite.
#[derive(Component)]
pub struct SaveFavourite;

/// The words of the button that saves a favourite, which say what it saves on the level shown.
#[derive(Component)]
pub struct SaveWords;

/// What the button that saves a favourite says on a planet and on a chart, where it saves
/// where the player is.
pub const SAVE: [&str; 2] = ["Save as favourite", "Save my spot"];

/// The row of the player at this index of the list of friends, hidden past its end.
#[derive(Component, Debug, Clone, Copy)]
pub struct PlayerRow(pub usize);

/// The name of the player at this index.
#[derive(Component, Debug, Clone, Copy)]
pub struct PlayerName(pub usize);

/// The colour square of the player at this index.
#[derive(Component, Debug, Clone, Copy)]
pub struct PlayerDot(pub usize);

/// The node of a row of a list of the panel.
fn strip() -> Node {
    Node {
        display: Display::None,
        width: percent(100),
        align_items: AlignItems::Center,
        column_gap: px(theme::GAP_XS),
        ..default()
    }
}

/// One hidden favourite: its name, which looks at it, and its buttons.
fn favourite(index: usize) -> impl Bundle {
    let name = Node {
        flex_grow: 1.0,
        min_width: px(0),
        overflow: Overflow::clip(),
        ..default()
    };
    let named = (FavName(index), label("", theme::FONT_SMALL, Tone::Plain));
    let button = |symbol: Icon, act: Do| (step_button(symbol), FavButton(index, act));
    (
        FavRow(index),
        strip(),
        children![
            (frame(), name, FavButton(index, Do::Go), children![named]),
            button(Icon::Send, Do::Course),
            button(Icon::Pencil, Do::Rename),
            button(Icon::Trash, Do::Delete),
        ],
    )
}

/// The favourites: their heading, the button that saves one and the pool of rows.
pub fn favourites() -> impl Bundle {
    let head = (
        text("Favourites", theme::FONT_SMALL, theme::MUTED),
        TextLayout::no_wrap(),
    );
    let row = Node {
        align_items: AlignItems::Center,
        column_gap: px(theme::GAP_S),
        ..default()
    };
    let words = (SaveWords, label(SAVE[0], theme::FONT_SMALL, Tone::Plain));
    let save = (
        frame(),
        SaveFavourite,
        row,
        children![icon_slot(Some(Icon::Star), Tone::Plain), words],
    );
    (
        column(percent(100), theme::GAP_XS),
        children![
            head,
            save,
            (
                column(percent(100), 0.0),
                Children::spawn(SpawnIter((0..FAVOURITES).map(favourite)))
            )
        ],
    )
}

/// One hidden player: a square of their colour and their name.
fn player(index: usize) -> impl Bundle {
    let dot = Node {
        width: px(theme::GAP_S),
        height: px(theme::GAP_S),
        ..default()
    };
    (
        frame(),
        PlayerRow(index),
        strip(),
        children![
            (PlayerDot(index), dot, BackgroundColor(Color::NONE)),
            (PlayerName(index), label("", theme::FONT_SMALL, Tone::Plain)),
        ],
    )
}

/// Marks the players with their heading, hidden while there are none.
#[derive(Component)]
pub struct Players;

/// The players: their heading over the pool of rows, hidden while there are none.
pub fn players() -> impl Bundle {
    let head = (
        text("Players", theme::FONT_SMALL, theme::MUTED),
        TextLayout::no_wrap(),
    );
    let block = Node {
        display: Display::None,
        width: percent(100),
        flex_direction: FlexDirection::Column,
        row_gap: px(theme::GAP_XS),
        ..default()
    };
    (
        Players,
        block,
        children![
            head,
            (
                column(percent(100), 0.0),
                Children::spawn(SpawnIter((0..PLAYERS).map(player)))
            )
        ],
    )
}
