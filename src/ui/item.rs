//! Text buttons: menu items, links and small step buttons.

use bevy::input_focus::tab_navigation::TabIndex;
use bevy::picking::hover::Hovered;
use bevy::prelude::*;
use bevy::ui_widgets::Button;

use super::icon::{Icon, icon, icon_slot};
use super::style::{Look, Tone};
use super::theme;

/// What every button carries: the widget, its hover state, its look and its background.
pub fn frame() -> impl Bundle {
    (
        Button,
        Hovered::default(),
        Look::default(),
        BackgroundColor(Color::NONE),
    )
}

/// Text that follows the look of the button around it. It never wraps.
pub fn label(content: impl Into<String>, size: f32, tone: Tone) -> impl Bundle {
    (
        Text::new(content),
        TextFont::from_font_size(size),
        TextLayout::no_wrap(),
        tone,
    )
}

/// The room of an icon followed by a label. Without an icon the room stays empty.
pub fn entry(symbol: impl Into<Option<Icon>>, content: &str, size: f32, tone: Tone) -> impl Bundle {
    (
        Node {
            align_items: AlignItems::Center,
            column_gap: px(theme::GAP_S),
            ..default()
        },
        children![icon_slot(symbol.into(), tone), label(content, size, tone)],
    )
}

/// Full-width menu row that wraps `content`. `group` orders it for keyboard navigation.
///
/// A chevron shows at its right end while it is the current row.
pub fn item_with(group: i32, content: impl Bundle) -> impl Bundle {
    (
        frame(),
        TabIndex(group),
        Node {
            width: percent(100),
            align_items: AlignItems::Center,
            justify_content: JustifyContent::SpaceBetween,
            column_gap: px(theme::GAP_S),
            padding: UiRect::axes(px(theme::GAP_S), px(theme::GAP_XS)),
            overflow: Overflow::clip(),
            ..default()
        },
        children![content, icon(Icon::ChevronRight, Tone::Marker)],
    )
}

/// Full-width menu row with an icon and body text.
pub fn item(symbol: impl Into<Option<Icon>>, content: &str, tone: Tone) -> impl Bundle {
    item_with(0, entry(symbol, content, theme::FONT_BODY, tone))
}

/// Small action that sizes to its content and takes keyboard focus.
pub fn link(group: i32, symbol: impl Into<Option<Icon>>, content: &str, tone: Tone) -> impl Bundle {
    (
        frame(),
        TabIndex(group),
        Node {
            padding: UiRect::axes(px(theme::GAP_S), px(theme::GAP_XS)),
            ..default()
        },
        children![entry(symbol, content, theme::FONT_SMALL, tone)],
    )
}

/// Small square icon button for stepping a value, reachable by pointer only.
pub fn step_button(symbol: Icon) -> impl Bundle {
    (
        frame(),
        Node {
            width: px(theme::ICON),
            height: px(theme::ICON),
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            ..default()
        },
        children![icon(symbol, Tone::Muted)],
    )
}
