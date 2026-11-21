//! Bars that show a share: a thin meter, and a stat row that names a number, draws its share and
//! prints its value. A number the player compares is read faster as a bar than as a float.
//!
//! ```text
//! Reach    [######----]  0.82 m
//! label    meter         value
//! ```

use bevy::prelude::*;

use super::theme;
use super::widgets::text;

/// Height in pixels of a meter.
pub const METER: f32 = theme::S1;

/// Width in pixels of the label column of a stat row: six glyphs of small text.
pub const STAT_LABEL: f32 = 64.0;

/// Width in pixels of the meter of a stat row.
pub const STAT_METER: f32 = 64.0;

/// Marks a stat row.
#[derive(Component, Debug, Clone, Copy, Default)]
pub struct Stat;

/// A meter `width` pixels wide filled to `share`, 0 to 1: the filled part in `TEXT` over the rest
/// in `HAIRLINE`, the track of every bar.
pub fn meter(width: f32, share: f32) -> impl Bundle {
    let node = Node {
        width: px(width),
        height: px(METER),
        flex_shrink: 0.0,
        ..default()
    };
    let fill = Node {
        width: percent(share.clamp(0.0, 1.0) * 100.0),
        height: percent(100),
        ..default()
    };
    (
        node,
        BackgroundColor(theme::HAIRLINE),
        children![(fill, BackgroundColor(theme::TEXT))],
    )
}

/// A stat row: `label` in secondary text, a meter filled to `share` and `value` in primary text,
/// on one small line.
pub fn stat(label: &str, share: f32, value: impl Into<String>) -> impl Bundle {
    let node = Node {
        align_items: AlignItems::Center,
        column_gap: px(theme::S2),
        min_height: px(theme::LINE_SMALL),
        flex_shrink: 0.0,
        ..default()
    };
    let name = Node {
        width: px(STAT_LABEL),
        flex_shrink: 0.0,
        ..default()
    };
    (
        Stat,
        node,
        children![
            (
                text(label, theme::FONT_SMALL, theme::TEXT_2),
                TextLayout::no_wrap(),
                name
            ),
            meter(STAT_METER, share),
            (
                text(value, theme::FONT_SMALL, theme::TEXT),
                TextLayout::no_wrap()
            ),
        ],
    )
}
