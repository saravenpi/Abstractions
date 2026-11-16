//! Layout and static text building blocks.

use bevy::prelude::*;
use bevy::text::LetterSpacing;

use super::theme;

/// Full-screen column with generous margins on the given background, filled from `justify`.
pub fn root_on(background: Color, justify: JustifyContent) -> impl Bundle {
    (
        Node {
            width: percent(100),
            height: percent(100),
            flex_direction: FlexDirection::Column,
            align_items: AlignItems::FlexStart,
            justify_content: justify,
            padding: UiRect::axes(percent(8), percent(6)),
            ..default()
        },
        BackgroundColor(background),
    )
}

/// Full-screen column over the dimmed world.
pub fn root() -> impl Bundle {
    root_on(theme::OVERLAY, JustifyContent::FlexStart)
}

/// Vertical stack with the given gap.
pub fn column(width: Val, gap: f32) -> impl Bundle {
    Node {
        width,
        flex_direction: FlexDirection::Column,
        row_gap: px(gap),
        ..default()
    }
}

/// Static text of a given size and color.
pub fn text(content: impl Into<String>, size: f32, color: Color) -> impl Bundle {
    (
        Text::new(content),
        TextFont::from_font_size(size),
        TextColor(color),
    )
}

/// Small muted uppercase label with letter spacing, used for section names.
pub fn caps(content: &str) -> impl Bundle {
    (
        text(content.to_uppercase(), theme::FONT_SMALL, theme::MUTED),
        LetterSpacing::Px(theme::CAPS_SPACING),
    )
}

/// Short accent rule that sits under a title.
pub fn rule() -> impl Bundle {
    (
        Node {
            width: px(48),
            height: px(2),
            margin: UiRect::new(px(0), px(0), px(theme::SPACE * 2.0), px(theme::SPACE * 5.0)),
            ..default()
        },
        BackgroundColor(theme::ACCENT),
    )
}

/// Border color of a hairline rule.
pub fn hairline() -> BorderColor {
    BorderColor::all(theme::HAIRLINE)
}

/// Muted hint pinned to the bottom-left corner.
pub fn footer(content: &str) -> impl Bundle {
    (
        text(content, theme::FONT_SMALL, theme::MUTED),
        Node {
            position_type: PositionType::Absolute,
            left: percent(8),
            bottom: px(theme::EDGE),
            ..default()
        },
    )
}
