//! Screen furniture: headings, key hints, pointer actions with a key hint, footers and notices.

use bevy::prelude::*;

use super::icon::Icon;
use super::item::{entry, frame, label};
use super::style::Tone;
use super::theme;
use super::widgets::{caps, text};

/// Section name in spaced capitals, with a gap below it.
pub fn heading(content: &str) -> impl Bundle {
    (
        caps(content),
        Node {
            margin: UiRect::bottom(px(theme::GAP_L)),
            ..default()
        },
    )
}

/// The name of a key, dimmed. It has no box: it is told apart by its brightness.
pub fn key_cap(key: impl Into<String>) -> impl Bundle {
    label(key, theme::FONT_SMALL, Tone::Muted)
}

/// Pointer button with an icon, a label and the key that does the same thing.
///
/// It takes no keyboard focus: the key is its keyboard access.
pub fn action(
    symbol: impl Into<Option<Icon>>,
    content: &str,
    key: &str,
    tone: Tone,
) -> impl Bundle {
    (
        frame(),
        Node {
            align_items: AlignItems::Center,
            column_gap: px(theme::GAP_S),
            padding: UiRect::axes(px(theme::GAP_S), px(theme::GAP_XS)),
            ..default()
        },
        children![
            entry(symbol, content, theme::FONT_SMALL, tone),
            key_cap(key)
        ],
    )
}

/// A horizontal strip of actions or tabs that wraps on a narrow window.
pub fn strip() -> Node {
    Node {
        align_items: AlignItems::Center,
        flex_wrap: FlexWrap::Wrap,
        column_gap: px(theme::GAP_M),
        row_gap: px(theme::GAP_XS),
        margin: UiRect::vertical(px(theme::GAP_M)),
        ..default()
    }
}

/// Dimmed hint pinned to the bottom-left corner.
pub fn footer(content: &str) -> impl Bundle {
    (
        text(content, theme::FONT_SMALL, theme::TEXT_2),
        Node {
            position_type: PositionType::Absolute,
            left: percent(5),
            right: percent(5),
            bottom: px(theme::GAP_M),
            ..default()
        },
    )
}

/// One line for a short message, empty until something writes it. It keeps its height.
pub fn notice() -> impl Bundle {
    (
        text("", theme::FONT_SMALL, theme::TEXT),
        TextLayout::no_wrap(),
        Node {
            min_height: px(theme::GAP_L),
            ..default()
        },
    )
}
