//! Empty states and notices: the places where the interface says why there is nothing, or what
//! just went wrong. Both say it in words, with an icon in the icon column of the rows around.

use bevy::prelude::*;

use super::icon::{Icon, icon_slot};
use super::style::Tone;
use super::theme;
use super::widgets::text;

/// Marks an empty state: the block shown in place of a list that has nothing to show.
#[derive(Component, Debug, Clone, Copy, Default)]
pub struct EmptyState;

/// Marks a notice: one line with an icon for a short message.
#[derive(Component, Debug, Clone, Copy, Default)]
pub struct NoticeRow;

/// An icon and one line of words beside it, on the icon column and the label line of a row.
fn line(symbol: Option<Icon>, words: impl Bundle) -> impl Bundle {
    let node = Node {
        align_items: AlignItems::Center,
        column_gap: px(theme::ICON_GAP),
        padding: UiRect::horizontal(px(theme::INSET)),
        min_height: px(theme::ROW_HEIGHT),
        flex_shrink: 0.0,
        ..default()
    };
    (node, children![icon_slot(symbol, Tone::Plain), words])
}

/// An empty state: what is missing, in body text beside its icon, and under it why and what to
/// do about it, in small secondary text that wraps. It is [`theme::span`] of six columns wide.
/// The action that fills the list follows it as the next sibling.
pub fn empty_state(symbol: Icon, title: &str, why: &str) -> impl Bundle {
    let node = Node {
        width: px(theme::span(6)),
        max_width: percent(100),
        flex_direction: FlexDirection::Column,
        flex_shrink: 0.0,
        ..default()
    };
    let title = text(title, theme::FONT_BODY, theme::TEXT);
    let reason = text(why, theme::FONT_SMALL, theme::TEXT_2);
    (
        EmptyState,
        node,
        children![line(Some(symbol), title), line(None, reason)],
    )
}

/// A notice: `symbol` and `words`, a text that whoever marks it writes. It keeps its height
/// while its text is empty, so nothing under it moves when a message comes.
pub fn notice_row(symbol: impl Into<Option<Icon>>, words: impl Bundle) -> impl Bundle {
    (NoticeRow, line(symbol.into(), words))
}
