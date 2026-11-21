//! The static layout of the pack screen over the blurred game: the tabs and the count line, the
//! list on the left, the card on the right and the key hints under them. `paint` fills them.

use bevy::prelude::*;

use super::Workshop;
use super::bench::Tab;
use super::keys::{DROP_KEY, MAKE_KEYS, SEARCH_KEY, SORT_KEY};
use super::layout::LIST;
use crate::craft::pack::SLOTS;
use crate::craft::wield::KEYS;
use crate::fauna::guide::layout::{BETWEEN, CARD, EDGE, FOOT, HEAD};
use crate::ui::{Blurred, KeyLine, KeySlot, Overlay, bound, key_text, text, theme};

/// What the footer says on the pack tab, with a `{}` for each key the layout names.
const PACK_HINTS: &str =
    "Up/Down choose  {} search  {} sort  Enter hold  {}-{} slot  {} drop  Tab workshop  {} close";

/// What the footer says on the workshop tab, with a `{}` for each key the layout names.
const MAKE_HINTS: &str = "Left/Right intent  Up/Down proposal  {} head  {} shaft  {} treat  \
                          Enter make  Tab pack  {} close";

/// Returns the slots of `keys`, which no setting moves.
fn fixed(keys: &[KeyCode]) -> impl Iterator<Item = KeySlot> + '_ {
    keys.iter().copied().map(KeySlot::from)
}

/// Returns the key hints of `tab`: every key as the layout of the player prints it, and the key
/// that closes the screen as it is bound.
pub fn hints(tab: Tab) -> KeyLine {
    let close = bound(|keys| keys.pack);
    match tab {
        Tab::Pack => {
            let keys = [SEARCH_KEY, SORT_KEY, KEYS[0], KEYS[SLOTS - 1], DROP_KEY];
            KeyLine::new(PACK_HINTS, fixed(&keys).chain([close]))
        }
        Tab::Make => KeyLine::new(MAKE_HINTS, fixed(&MAKE_KEYS).chain([close])),
    }
}

/// Marks the name of a tab in the header.
#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub struct TabName(pub Tab);

/// Marks the count line under the tabs.
#[derive(Component)]
pub struct Tally;

/// Marks the column the rows of the list are drawn in.
#[derive(Component)]
pub struct Rows;

/// Marks the column the card is drawn in.
#[derive(Component)]
pub struct Facts;

/// Marks the key hints.
#[derive(Component)]
pub struct Hints;

/// Returns the header: the two tab names and the count line.
fn header() -> impl Bundle {
    let node = Node {
        height: px(HEAD),
        flex_shrink: 0.0,
        flex_direction: FlexDirection::Column,
        ..default()
    };
    let tabs = Node {
        column_gap: px(theme::GAP_L),
        ..default()
    };
    let tab =
        |tab: Tab, name: &'static str| (TabName(tab), text(name, theme::FONT_HEADING, theme::TEXT));
    (
        node,
        children![
            (
                tabs,
                children![tab(Tab::Pack, "Pack"), tab(Tab::Make, "Workshop")]
            ),
            (
                Tally,
                text("", theme::FONT_SMALL, theme::TEXT_2),
                TextLayout::no_wrap()
            ),
        ],
    )
}

/// Returns the list beside the card.
fn body() -> impl Bundle {
    let node = Node {
        flex_grow: 1.0,
        min_height: px(0),
        column_gap: px(BETWEEN),
        overflow: Overflow::clip(),
        ..default()
    };
    let list = Node {
        width: px(LIST),
        flex_shrink: 0.0,
        flex_direction: FlexDirection::Column,
        ..default()
    };
    let card = Node {
        width: px(CARD),
        max_width: percent(100),
        flex_direction: FlexDirection::Column,
        overflow: Overflow::clip(),
        ..default()
    };
    (node, children![(Rows, list), (Facts, card)])
}

/// Spawns the empty screen.
pub fn spawn(commands: &mut Commands) {
    let root = Node {
        width: percent(100),
        height: percent(100),
        flex_direction: FlexDirection::Column,
        padding: UiRect::all(px(EDGE)),
        ..default()
    };
    let foot = Node {
        height: px(FOOT),
        flex_shrink: 0.0,
        align_items: AlignItems::FlexEnd,
        ..default()
    };
    let hints = (
        Hints,
        key_text(hints(Tab::Pack), theme::FONT_SMALL, theme::TEXT_2),
        TextLayout::no_wrap(),
    );
    let screen = (Overlay, Blurred, root, BackgroundColor(theme::SCRIM));
    commands.spawn((
        DespawnOnExit(Workshop::Open),
        screen,
        children![header(), body(), (foot, children![hints])],
    ));
}
