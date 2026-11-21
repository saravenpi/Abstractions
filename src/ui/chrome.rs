//! Screen furniture: headings, key hints, pointer actions with a key hint, footers and notices.

use bevy::prelude::*;

use super::icon::Icon;
use super::item::{entry, frame, label};
use super::key_line::KeyLine;
use super::keys::key_label;
use super::style::Tone;
use super::theme;
use super::widgets::{caps, text};

/// Section name in spaced capitals, with a gap below it.
pub fn heading(content: &str) -> impl Bundle {
    (
        caps(content),
        Node {
            margin: UiRect::bottom(px(theme::S4)),
            ..default()
        },
    )
}

/// The name of a key, dimmed. It has no box: it is told apart by its brightness. Every action
/// shows its key: pair this with the label of the action.
pub fn key_cap(key: impl Into<String>) -> impl Bundle {
    label(key, theme::FONT_SMALL, Tone::Muted)
}

/// Pointer button with an icon, a label and the key that does the same thing.
///
/// It takes no keyboard focus: the key is its keyboard access.
pub fn action(
    symbol: impl Into<Option<Icon>>,
    content: &str,
    key: impl Into<KeyLine>,
    tone: Tone,
) -> impl Bundle {
    (
        frame(),
        Node {
            align_items: AlignItems::Center,
            column_gap: px(theme::S2),
            padding: UiRect::axes(px(theme::S3), px(theme::S2)),
            ..default()
        },
        children![
            entry(symbol, content, theme::FONT_SMALL, tone),
            key_label(key, theme::FONT_SMALL, Tone::Muted)
        ],
    )
}

/// A horizontal strip of actions or tabs that wraps on a narrow window.
pub fn strip() -> Node {
    Node {
        align_items: AlignItems::Center,
        flex_wrap: FlexWrap::Wrap,
        column_gap: px(theme::S3),
        row_gap: px(theme::S1),
        margin: UiRect::vertical(px(theme::S3)),
        ..default()
    }
}

/// Dimmed hint pinned to the bottom of the screen, in line with the content of a screen: the same
/// side margin, and one edge margin from the bottom.
pub fn footer(content: &str) -> impl Bundle {
    (
        text(content, theme::FONT_SMALL, theme::TEXT_2),
        Node {
            position_type: PositionType::Absolute,
            left: percent(5),
            right: percent(5),
            bottom: px(theme::EDGE),
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
            min_height: px(theme::S4),
            ..default()
        },
    )
}
