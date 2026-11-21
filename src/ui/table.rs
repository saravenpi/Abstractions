//! Rows: settings rows with a name and a value, and table rows with fixed columns.

use bevy::input_focus::tab_navigation::TabIndex;
use bevy::prelude::*;

use super::item::frame;
use super::theme;
use super::widgets::caps;

/// The node of a full-width row of `height` pixels.
fn strip(height: f32, justify: JustifyContent) -> Node {
    Node {
        width: percent(100),
        height: px(height),
        flex_shrink: 0.0,
        align_items: AlignItems::Center,
        justify_content: justify,
        column_gap: px(theme::GAP_M),
        padding: UiRect::horizontal(px(theme::GAP_S)),
        overflow: Overflow::clip(),
        ..default()
    }
}

/// Settings row with a name on the left and a value on the right.
pub fn row_with(group: i32, name: impl Bundle, value: impl Bundle) -> impl Bundle {
    (
        frame(),
        TabIndex(group),
        strip(theme::ROW_HEIGHT, JustifyContent::SpaceBetween),
        children![name, value],
    )
}

/// The value column of a settings row: a fixed width, sized for the widest value any setting
/// shows, that never shrinks. The value inside must not wrap.
pub fn value_cell(justify: JustifyContent, content: impl Bundle) -> impl Bundle {
    (
        Node {
            width: px(theme::VALUE_WIDTH),
            min_width: px(theme::VALUE_WIDTH),
            flex_shrink: 0.0,
            justify_content: justify,
            overflow: Overflow::clip(),
            ..default()
        },
        children![content],
    )
}

/// Focusable table row of `height` pixels. `cells` is a `children!` list of [`cell`]s.
pub fn table_row(group: i32, height: f32, cells: impl Bundle) -> impl Bundle {
    (
        frame(),
        TabIndex(group),
        strip(height, JustifyContent::FlexStart),
        cells,
    )
}

/// The line of column names above a table. `cells` is a `children!` list of [`head`]s and buttons.
pub fn table_head(cells: impl Bundle) -> impl Bundle {
    (strip(theme::ROW_COMPACT, JustifyContent::FlexStart), cells)
}

/// One column of a row: `width` pixels wide whatever it holds. Content that does not fit is cut.
pub fn cell(width: f32, content: impl Bundle) -> impl Bundle {
    (
        Node {
            width: px(width),
            flex_shrink: 0.0,
            flex_direction: FlexDirection::Column,
            overflow: Overflow::clip(),
            ..default()
        },
        children![content],
    )
}

/// The name of one column, `width` pixels wide.
pub fn head(width: f32, name: &str) -> impl Bundle {
    cell(width, caps(name))
}
