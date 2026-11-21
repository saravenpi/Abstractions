//! The pieces every section of the side panel is built from: its heading in spaced capitals, the
//! row of the heading, and the node of a row of its list.
//!
//! A row reaches a few pixels past the text column on both sides, so its words line up with the
//! headings while the tint of a lit or selected row frames them.

use bevy::prelude::*;
use bevy::text::LetterSpacing;

use super::keys::HOME_KEY;
use super::layout::LEGEND;
use super::typing::SEARCH_KEY;
use crate::ui::{KeyLine, bound, key_text, text, theme};

/// The heading of a section of the side panel: its name in white spaced capitals. The section
/// keeps [`theme::GAP_S`] under it.
pub fn heading(name: &str) -> impl Bundle {
    (
        text(name.to_uppercase(), theme::FONT_SMALL, theme::TEXT),
        TextLayout::no_wrap(),
        LetterSpacing::Px(theme::CAPS_SPACING),
    )
}

/// The row of the heading of a section, with what it holds at its right end and the room under
/// it.
pub fn heading_row() -> Node {
    Node {
        width: percent(100),
        justify_content: JustifyContent::SpaceBetween,
        align_items: AlignItems::Center,
        column_gap: px(theme::GAP_S),
        margin: UiRect::bottom(px(theme::GAP_S)),
        flex_shrink: 0.0,
        ..default()
    }
}

/// The node of one row of a list of the side panel, `height` pixels tall, whose content starts
/// on the text column.
pub fn row_node(height: f32) -> Node {
    Node {
        height: px(height),
        flex_shrink: 0.0,
        align_items: AlignItems::Center,
        column_gap: px(theme::GAP_XS),
        margin: UiRect::horizontal(px(-theme::GAP_XS)),
        padding: UiRect::horizontal(px(theme::GAP_XS)),
        overflow: Overflow::clip(),
        ..default()
    }
}

/// The keys of the map, in small text that wraps to the two lines the foot of the panel keeps.
/// Each key is named as the layout of the player prints it, and Go and the pin follow their
/// bindings.
pub fn legend() -> impl Bundle {
    let node = Node {
        margin: UiRect::top(auto()),
        ..default()
    };
    let keys = [
        bound(|keys| keys.engage),
        bound(|keys| keys.waypoint),
        HOME_KEY.into(),
        SEARCH_KEY.into(),
    ];
    let line = KeyLine::new(LEGEND, keys);
    (node, key_text(line, theme::FONT_SMALL, theme::TEXT))
}
