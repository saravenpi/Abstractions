//! The skeleton every full-screen page shares: one screen edge, a heading zone, a content zone
//! and a footer zone. A page built here puts its heading, its content and its key legend at the
//! same place as every other page.
//!
//! ```text
//! EDGE
//!   heading zone   LINE_HEADING high: the title on the left, a status on the right
//!   S5
//!   content zone   takes the room left, clips what does not fit
//!   S4
//!   footer zone    ROW_HEIGHT high: the back action first, then the key legend
//! EDGE
//! ```

use bevy::prelude::*;

use super::backdrop::Blurred;
use super::theme;
use super::widgets::{Overlay, text};

/// Height of a page above its content zone: the screen edge, the heading and the gap under it.
pub const PAGE_HEAD: f32 = theme::EDGE + theme::LINE_HEADING + theme::S5;

/// Height of a page below its content zone: the gap, a footer of one line and the screen edge.
pub const PAGE_FOOT: f32 = theme::S4 + theme::ROW_HEIGHT + theme::EDGE;

/// Marks the root of a full-screen page.
#[derive(Component, Debug, Clone, Copy, Default)]
pub struct Page;

/// Which of the three zones of a page a node is.
#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub enum Zone {
    /// The heading zone at the top.
    Head,
    /// The content zone between the two others.
    Body,
    /// The footer zone at the bottom.
    Foot,
}

/// The root of a page on `fill`: the full screen, padded by the screen edge on every side, its
/// zones stacked from the top. Escape pressed while it exists is a cancel.
///
/// It is sized in window units, not in shares: Bevy puts every root in a grid row of auto
/// height, where a share of the height grows with the content, so a long page pushed its
/// footer and its last rows below the window.
pub fn page_frame(fill: Color) -> impl Bundle {
    let node = Node {
        width: vw(100),
        height: vh(100),
        flex_direction: FlexDirection::Column,
        padding: UiRect::all(px(theme::EDGE)),
        ..default()
    };
    (Page, Overlay, node, BackgroundColor(fill))
}

/// The title of a page: heading text on the shared text line.
pub fn page_title(title: &str) -> impl Bundle {
    (
        text(title, theme::FONT_HEADING, theme::TEXT),
        TextLayout::no_wrap(),
    )
}

/// The status of a page: one small secondary line at the right end of the heading zone, such
/// as a count or "Saved". Whoever marks it writes it.
pub fn page_status(content: impl Into<String>) -> impl Bundle {
    (
        text(content, theme::FONT_SMALL, theme::TEXT_2),
        TextLayout::no_wrap(),
    )
}

/// The heading zone of a page. `content` is a `children!` list: a [`page_title`], then a
/// [`page_status`] if the page has one.
pub fn page_head(content: impl Bundle) -> impl Bundle {
    let node = Node {
        height: px(theme::LINE_HEADING),
        flex_shrink: 0.0,
        align_items: AlignItems::Center,
        justify_content: JustifyContent::SpaceBetween,
        column_gap: px(theme::GUTTER),
        padding: UiRect::horizontal(px(theme::INSET)),
        margin: UiRect::bottom(px(theme::S5)),
        ..default()
    };
    (Zone::Head, node, content)
}

/// The content zone of a page: it takes the room between the heading and the footer and lays
/// its children out along `direction`, a gutter apart when side by side.
pub fn page_body(direction: FlexDirection) -> impl Bundle {
    let node = Node {
        flex_direction: direction,
        flex_grow: 1.0,
        flex_basis: px(0),
        min_height: px(0),
        column_gap: px(theme::GUTTER),
        overflow: Overflow::clip_y(),
        ..default()
    };
    (Zone::Body, node)
}

/// The footer zone of a page. `content` is a `children!` list: the back action first, then the
/// legend of the keys of the page.
pub fn page_foot(content: impl Bundle) -> impl Bundle {
    let node = Node {
        min_height: px(theme::ROW_HEIGHT),
        flex_shrink: 0.0,
        align_items: AlignItems::Center,
        column_gap: px(theme::GUTTER),
        margin: UiRect::top(px(theme::S4)),
        ..default()
    };
    (Zone::Foot, node, content)
}

/// A page over the blurred game: `head`, `body` and `foot` are its three zones, built with
/// [`page_head`], [`page_body`] and [`page_foot`].
pub fn page(head: impl Bundle, body: impl Bundle, foot: impl Bundle) -> impl Bundle {
    (
        page_frame(theme::SCRIM),
        Blurred,
        children![head, body, foot],
    )
}
