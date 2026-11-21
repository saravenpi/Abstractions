//! The pieces every section of the side panel is built from: its heading with an icon and spaced
//! capitals, the row of the heading, the node of a row of its list, the icon of each kind of
//! place, the part of the panel that scrolls and the legend of the keys.
//!
//! A row reaches a few pixels past the text column on both sides, so its words line up with the
//! headings while the tint of a lit or selected row frames them.

use bevy::prelude::*;
use bevy::text::LetterSpacing;

use super::layout::{LEGEND, SECTION_GAP};
use super::route::Target;
use crate::ui::{Icon, Tone, hints, icon, scroll_area, text, theme};

/// The heading of a section of the side panel: its icon and its name in white spaced capitals.
/// The section keeps [`theme::GAP_S`] under it.
pub fn heading(symbol: Icon, name: &str) -> impl Bundle {
    let node = Node {
        align_items: AlignItems::Center,
        column_gap: px(theme::S2),
        flex_shrink: 0.0,
        ..default()
    };
    let named = (
        text(name.to_uppercase(), theme::FONT_SMALL, theme::TEXT),
        TextLayout::no_wrap(),
        LetterSpacing::Px(theme::CAPS_SPACING),
    );
    (node, children![icon(symbol, Tone::Plain), named])
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

/// The keys of the map at the foot of the panel, each after what it does, in a row that wraps
/// to the lines the panel keeps for it.
pub fn legend() -> impl Bundle {
    let node = Node {
        width: percent(100),
        flex_shrink: 0.0,
        ..default()
    };
    (node, children![crate::ui::legend(hints(&LEGEND))])
}

/// Returns the icon of what a place is: a star, a planet, a moon or a galaxy.
pub fn kind(target: Target) -> Icon {
    match target {
        Target::Star(_) => Icon::Star,
        Target::Body(body) if body.is_moon() => Icon::Moon,
        Target::Body(_) => Icon::Planet,
        Target::Galaxy(_) => Icon::Galaxy,
    }
}

/// The part of the panel that scrolls when the window is short: the facts of the card, the pins
/// and the players, sections apart.
pub(super) fn body() -> impl Bundle {
    let node = Node {
        width: percent(100),
        flex_direction: FlexDirection::Column,
        flex_shrink: 0.0,
        row_gap: px(SECTION_GAP),
        ..default()
    };
    let parts = children![
        super::readout::facts(),
        crate::map::pins(),
        crate::map::players()
    ];
    scroll_area(children![(node, parts)])
}
