//! The lens in motion: easing toward where it was asked to look, moving the sheet of the
//! chart with it, and keeping the scale bar true.

use bevy::prelude::*;

use super::labels::scale_bar;
use super::legend::{ScaleBar, ScaleText};
use super::map::{SIZE, Sheet};
use super::pointer::MapView;
use crate::ui::units::distance;

/// How fast the lens closes the gap to its goal, per second. It covers most of it in a
/// fifth of a second.
pub(super) const EASE: f32 = 12.0;

/// What the scale of the map says on a chart whose distances are not drawn true.
pub(super) const NOT_TO_SCALE: &str = "orbits not to scale";

/// Moves the lens part of the way to its goal, so a zoom or a jump glides instead of cutting.
pub(super) fn ease(time: Res<Time>, mut view: ResMut<MapView>) {
    if view.lens == view.goal {
        return;
    }
    let share = 1.0 - (-EASE * time.delta_secs()).exp();
    view.lens = view.lens.toward(view.goal, share);
}

/// Moves and sizes the sheet the chart is drawn on, so the map shows what the lens looks at.
pub(super) fn slide(view: Res<MapView>, mut sheets: Query<&mut Node, With<Sheet>>) {
    let corner = view.lens.to_map(Vec2::new(-1.0, 1.0)) / SIZE * 100.0;
    let side = percent(view.lens.zoom * 100.0);
    for mut node in &mut sheets {
        let wanted = (percent(corner.x), percent(corner.y), side, side);
        if (node.left, node.top, node.width, node.height) != wanted {
            (node.left, node.top, node.width, node.height) = wanted;
        }
    }
}

/// Writes the scale of the map when the zoom, the size of the map or the chart changed: a
/// round distance and a bar of that length.
pub(super) fn rule(
    view: Res<MapView>,
    fresh: Query<(), Added<ScaleText>>,
    mut bars: Query<&mut Node, With<ScaleBar>>,
    mut texts: Query<&mut Text, With<ScaleText>>,
    mut last: Local<(f32, Option<f64>)>,
) {
    let now = (view.lens.zoom * view.side, view.span);
    if *last == now && fresh.is_empty() {
        return;
    }
    *last = now;
    let true_scale = view.span.filter(|_| now.0 > 0.0);
    let bar = true_scale.map(|span| scale_bar(span * 2.0 / f64::from(now.0)));
    let (width, words) = match bar {
        Some((pixels, metres)) => (pixels, distance(metres)),
        None => (0.0, NOT_TO_SCALE.to_string()),
    };
    for mut node in &mut bars {
        node.width = px(width);
    }
    for mut text in &mut texts {
        text.set_if_neq(Text(words.clone()));
    }
}
