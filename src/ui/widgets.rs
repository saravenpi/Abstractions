//! Small bundle functions for the menu building blocks.

use bevy::picking::hover::Hovered;
use bevy::prelude::*;
use bevy::ui_widgets::Button;

use super::theme;

/// Full-screen dimmed container that centers its children.
pub fn root() -> impl Bundle {
    (
        Node {
            width: percent(100),
            height: percent(100),
            flex_direction: FlexDirection::Column,
            align_items: AlignItems::Center,
            justify_content: JustifyContent::Center,
            ..default()
        },
        BackgroundColor(theme::OVERLAY),
    )
}

/// Dark translucent column that groups a menu's widgets.
pub fn panel() -> impl Bundle {
    (
        Node {
            flex_direction: FlexDirection::Column,
            align_items: AlignItems::Center,
            row_gap: px(theme::GAP),
            padding: UiRect::all(px(theme::PAD)),
            ..default()
        },
        BackgroundColor(theme::PANEL),
    )
}

/// Horizontal group of widgets, such as one settings row.
pub fn row() -> impl Bundle {
    Node {
        align_items: AlignItems::Center,
        column_gap: px(theme::GAP),
        ..default()
    }
}

/// Body text.
pub fn label(text: impl Into<String>) -> impl Bundle {
    (
        Text::new(text),
        TextFont::from_font_size(theme::FONT_BODY),
        TextColor(theme::TEXT),
    )
}

/// Large heading text.
pub fn heading(text: impl Into<String>, size: f32) -> impl Bundle {
    (
        Text::new(text),
        TextFont::from_font_size(size),
        TextColor(theme::TEXT),
    )
}

/// Plain text button of the given width. Add an action component next to it.
pub fn button(text: impl Into<String>, width: f32) -> impl Bundle {
    button_with(width, label(text))
}

/// Button of the given width that wraps any content bundle.
pub fn button_with(width: f32, content: impl Bundle) -> impl Bundle {
    (
        Button,
        Hovered::default(),
        Node {
            width: px(width),
            padding: UiRect::axes(px(12), px(8)),
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            ..default()
        },
        BackgroundColor(theme::BUTTON),
        children![content],
    )
}
