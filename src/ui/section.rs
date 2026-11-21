//! Sections: a title and the content it heads. One rule everywhere for the space around a title:
//! more above it than below it, so a title belongs to what it heads and never to the block
//! before it.
//!
//! ```text
//!   ...last row of the section before
//!   SECTION_ABOVE (S5) on a page, GROUP_ABOVE (S4) inside a card
//!   [icon] TITLE IN SPACED CAPITALS
//!   TITLE_BELOW (S2)
//!   first row of this section, its label under the title, its icon under the icon
//! ```

use bevy::prelude::*;

use super::icon::{Icon, icon_slot};
use super::style::Tone;
use super::theme;
use super::widgets::caps;

/// Marks a section: a column of a title and its content.
#[derive(Component, Debug, Clone, Copy, Default)]
pub struct Section;

/// Marks the title of a section.
#[derive(Component, Debug, Clone, Copy, Default)]
pub struct SectionTitle;

/// Marks a column of sections, which sets the space above each title.
#[derive(Component, Debug, Clone, Copy, Default)]
pub struct Sections;

/// The node of a column of sections `gap` pixels apart.
fn stack(gap: f32) -> impl Bundle {
    let node = Node {
        width: percent(100),
        flex_direction: FlexDirection::Column,
        flex_shrink: 0.0,
        row_gap: px(gap),
        ..default()
    };
    (Sections, node)
}

/// The column that holds the sections of a page, [`theme::SECTION_ABOVE`] apart.
pub fn sections() -> impl Bundle {
    stack(theme::SECTION_ABOVE)
}

/// The column that holds the groups of a card or a panel, [`theme::GROUP_ABOVE`] apart.
pub fn groups() -> impl Bundle {
    stack(theme::GROUP_ABOVE)
}

/// The title of a section: the icon of what it heads, in the icon column of the rows under it,
/// then its name in spaced capitals on their label line.
pub fn section_title(symbol: impl Into<Option<Icon>>, title: &str) -> impl Bundle {
    let node = Node {
        align_items: AlignItems::Center,
        column_gap: px(theme::ICON_GAP),
        padding: UiRect::horizontal(px(theme::INSET)),
        ..default()
    };
    (
        SectionTitle,
        node,
        children![icon_slot(symbol.into(), Tone::Muted), caps(title)],
    )
}

/// A section: `title` with its icon, then `body`, one node that holds the content. Put it in a
/// [`sections`] or a [`groups`] column, which sets the space above it.
pub fn section(symbol: impl Into<Option<Icon>>, title: &str, body: impl Bundle) -> impl Bundle {
    let node = Node {
        width: percent(100),
        flex_direction: FlexDirection::Column,
        flex_shrink: 0.0,
        row_gap: px(theme::TITLE_BELOW),
        ..default()
    };
    (Section, node, children![section_title(symbol, title), body])
}

/// The column of the rows of a section, with no space between rows: a row carries its own
/// height.
pub fn rows() -> impl Bundle {
    Node {
        width: percent(100),
        flex_direction: FlexDirection::Column,
        flex_shrink: 0.0,
        ..default()
    }
}
