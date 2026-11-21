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
            height: px(theme::ROW_HEIGHT),
            flex_shrink: 0.0,
            column_gap: px(KEY_GAP),
            padding: UiRect::horizontal(px(theme::INSET)),
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
        margin: UiRect::top(px(theme::S4)),
        ..default()
    }
}

/// The back action of a page: the first thing of its footer zone, on every page. `words` says
/// what Escape does there, such as "Back", "Resume" or "Cancel".
pub fn back(words: &str) -> impl Bundle {
    action(Icon::Back, words, "Esc", Tone::Plain)
}

/// One line for a short message, empty until something writes it. It keeps its height.
pub fn notice() -> impl Bundle {
    (
        text("", theme::FONT_SMALL, theme::TEXT),
        TextLayout::no_wrap(),
        Node {
            min_height: px(theme::LINE_SMALL),
            margin: UiRect::left(px(theme::INSET)),
            flex_shrink: 0.0,
            ..default()
        },
    )
}
