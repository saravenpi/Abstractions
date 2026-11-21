//! Keeping the screen in shape: the square map and the search results follow the window.

use bevy::prelude::*;
use bevy::window::PrimaryWindow;

use super::board::Board;
use super::layout::{SMALLEST, measure};
use super::map::Field;
use super::pointer::MapView;
use super::view::Panel;

/// The window, and the screen when it was just spawned.
type Seen<'w, 's> = (
    Query<'w, 's, &'static Window, With<PrimaryWindow>>,
    Query<'w, 's, (), Added<Panel>>,
);

/// Sizes the square map and the search results for the window, when the window or the interface
/// scale changed or the screen was just spawned. The window is measured in logical pixels of the
/// interface: its size over the interface scale. Without a window, as in a test, the reference
/// one is assumed.
pub(super) fn fit(
    (windows, fresh): Seen,
    scale: Option<Res<UiScale>>,
    mut fields: Query<&mut Node, With<Field>>,
    (mut board, mut view): (ResMut<Board>, ResMut<MapView>),
    mut last: Local<Option<Vec2>>,
) {
    let scale = scale.map_or(1.0, |scale| scale.0).max(f32::EPSILON);
    let size = |window: &Window| Vec2::new(window.width(), window.height()) / scale;
    let now = windows.iter().next().map_or(SMALLEST, size);
    if *last == Some(now) && fresh.is_empty() {
        return;
    }
    *last = Some(now);
    let fit = measure(now);
    for mut node in &mut fields {
        if node.width != px(fit.map) {
            (node.width, node.height) = (px(fit.map), px(fit.map));
        }
    }
    view.side = fit.map;
    if board.fit != fit.rows {
        board.fit = fit.rows;
        board.step(0);
    }
}
