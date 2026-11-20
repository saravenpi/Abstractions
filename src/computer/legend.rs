//! The legend of the charts, written over the map with its other words: the letter for north
//! at the right end of the top line and the scale at the right end of the bottom line.

use bevy::prelude::*;

use super::map::ChartLayer;
use crate::ui::{text, theme};

/// Marks the bar whose length stands for the distance written above it.
#[derive(Component)]
pub(super) struct ScaleBar;

/// Marks the text that says which distance the scale bar stands for.
#[derive(Component)]
pub(super) struct ScaleText;

/// The letter for north, which is up on a chart, shown with the charts only.
pub(super) fn north() -> impl Bundle {
    (ChartLayer, text("N", theme::FONT_SMALL, theme::TEXT_2))
}

/// The scale of a chart, shown with the charts only: a distance over a bar of that length,
/// both at the right end.
pub(super) fn scale() -> impl Bundle {
    let bar = Node {
        height: px(2),
        ..default()
    };
    let node = Node {
        flex_direction: FlexDirection::Column,
        align_items: AlignItems::FlexEnd,
        flex_shrink: 0.0,
        ..default()
    };
    (
        ChartLayer,
        node,
        children![
            (ScaleText, text("", theme::FONT_SMALL, theme::TEXT_2)),
            (ScaleBar, bar, BackgroundColor(theme::TEXT)),
        ],
    )
}
