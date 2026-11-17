//! Layout and static text building blocks.

use bevy::prelude::*;
use bevy::text::LetterSpacing;

use super::backdrop::Blurred;
use super::theme;

/// Marks the full-screen root of a menu or screen. Escape pressed while one exists is a cancel.
#[derive(Component, Debug, Clone, Copy, Default)]
pub struct Overlay;

/// Full-screen column with margins on the given background, filled from `justify`.
pub fn root_on(background: Color, justify: JustifyContent) -> impl Bundle {
    (
        Overlay,
        Node {
            width: percent(100),
            height: percent(100),
            flex_direction: FlexDirection::Column,
            align_items: AlignItems::FlexStart,
            justify_content: justify,
            padding: UiRect::axes(percent(5), percent(4)),
            ..default()
        },
        BackgroundColor(background),
    )
}

/// Full-screen column over the game: the game shows through, blurred and darkened, and the
/// content is white. Without a game behind it, it is a plain dark veil.
pub fn root() -> impl Bundle {
    (root_on(theme::SCRIM, JustifyContent::FlexStart), Blurred)
}

/// The content of a screen: a plain column that never grows past the screen. It has no
/// background and no border.
pub fn panel() -> impl Bundle {
    Node {
        flex_direction: FlexDirection::Column,
        max_width: percent(100),
        max_height: percent(100),
        overflow: Overflow::clip(),
        ..default()
    }
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

/// Static text of a given size and color. It wraps inside its box.
pub fn text(content: impl Into<String>, size: f32, color: Color) -> impl Bundle {
    (
        Text::new(content),
        TextFont::from_font_size(size),
        TextColor(color),
    )
}

/// Small dimmed uppercase label with letter spacing, used for section and column names.
pub fn caps(content: &str) -> impl Bundle {
    (
        text(content.to_uppercase(), theme::FONT_SMALL, theme::MUTED),
        TextLayout::no_wrap(),
        LetterSpacing::Px(theme::CAPS_SPACING),
    )
}

/// Short white rule that sits under a title.
pub fn rule() -> impl Bundle {
    (
        Node {
            width: px(48),
            height: px(2),
            margin: UiRect::new(px(0), px(0), px(theme::GAP_S), px(theme::GAP_L)),
            ..default()
        },
        BackgroundColor(theme::TEXT),
    )
}

/// Border color of a dividing line.
pub fn hairline() -> BorderColor {
    BorderColor::all(theme::HAIRLINE)
}
