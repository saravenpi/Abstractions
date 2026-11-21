//! Screen furniture: headings, pointer actions with a key hint, strips and notices.

use bevy::prelude::*;

use super::icon::Icon;
use super::item::{entry, frame};
use super::keys::{KEY_GAP, key_cap};
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
            column_gap: px(KEY_GAP),
            padding: UiRect::axes(px(theme::S3), px(theme::S2)),
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
        column_gap: px(theme::S3),
        row_gap: px(theme::S1),
        margin: UiRect::vertical(px(theme::S3)),
        ..default()
    }
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
