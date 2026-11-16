//! Text buttons: menu items, links and small step buttons.

use bevy::input_focus::tab_navigation::TabIndex;
use bevy::picking::hover::Hovered;
use bevy::prelude::*;
use bevy::ui_widgets::Button;

use super::style::{Look, Shift, Tone};
use super::theme;
use super::widgets::hairline;

/// Text that follows the look of the button around it.
pub fn label(content: impl Into<String>, size: f32, tone: Tone) -> impl Bundle {
    (Text::new(content), TextFont::from_font_size(size), tone)
}

/// Marker glyph that appears left of an active item.
fn marker() -> impl Bundle {
    (
        label(">", theme::FONT_BODY, Tone::Marker),
        Node {
            position_type: PositionType::Absolute,
            left: px(-theme::SPACE * 2.5),
            ..default()
        },
    )
}

/// Full-width menu row that wraps `content`. `group` orders it for keyboard navigation.
pub fn item_with(group: i32, content: impl Bundle) -> impl Bundle {
    (
        Button,
        Hovered::default(),
        TabIndex(group),
        Look::default(),
        Shift,
        Node {
            width: percent(100),
            align_items: AlignItems::Center,
            padding: UiRect::vertical(px(theme::SPACE)),
            ..default()
        },
        children![marker(), content],
    )
}

/// Full-width menu row with body text.
pub fn item(content: &str, tone: Tone) -> impl Bundle {
    item_with(0, label(content, theme::FONT_BODY, tone))
}

/// Small text action that sizes to its text.
pub fn link(group: i32, content: &str, tone: Tone) -> impl Bundle {
    (
        Button,
        Hovered::default(),
        TabIndex(group),
        Look::default(),
        Node {
            padding: UiRect::axes(px(theme::SPACE), px(theme::SPACE / 2.0)),
            ..default()
        },
        children![label(content, theme::FONT_SMALL, tone)],
    )
}

/// Small square button for stepping a value, reachable by pointer only.
pub fn step_button(content: &str) -> impl Bundle {
    (
        Button,
        Hovered::default(),
        Look::default(),
        Node {
            width: px(theme::SPACE * 3.0),
            height: px(theme::SPACE * 3.0),
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            ..default()
        },
        children![label(content, theme::FONT_BODY, Tone::Muted)],
    )
}

/// Settings row with a name on the left and a value on the right, closed by a hairline.
pub fn row_with(group: i32, name: impl Bundle, value: impl Bundle) -> impl Bundle {
    (
        Button,
        Hovered::default(),
        TabIndex(group),
        Look::default(),
        Node {
            width: percent(100),
            height: px(theme::ROW_HEIGHT),
            align_items: AlignItems::Center,
            justify_content: JustifyContent::SpaceBetween,
            border: UiRect::bottom(px(1)),
            ..default()
        },
        hairline(),
        children![marker(), name, value],
    )
}
