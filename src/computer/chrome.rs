//! Keeping the screen in shape: the square map and the search results follow the window.

use bevy::prelude::*;
use bevy::window::PrimaryWindow;

use super::board::Board;
use super::layout::{SMALLEST, measure};
use super::map::Field;
use super::pointer::MapView;
use super::view::Panel;
use crate::map::{PINS, PLAYERS, Places};
use crate::menus::{MarkerKind, Markers};

/// The window, and the screen when it was just spawned.
type Seen<'w, 's> = (
    Query<'w, 's, &'static Window, With<PrimaryWindow>>,
    Query<'w, 's, (), Added<Panel>>,
);

/// Returns how many rows the pins and the players lists of the panel show, none for a list
/// that does not exist.
fn listed(places: Option<&Places>, markers: Option<&Markers>) -> (usize, usize) {
    let saved = places.map_or(0, |places| places.pins.len());
    let friends = markers.map_or(0, |markers| {
        let kinds = markers.0.iter().map(|marker| marker.kind);
        kinds.filter(|kind| *kind == MarkerKind::Friend).count()
    });
    (saved.min(PINS), friends.min(PLAYERS))
}

/// What the size of the lists and of the interface is read from.
type Lists<'w> = (
    Option<Res<'w, Places>>,
    Option<Res<'w, Markers>>,
    Option<Res<'w, UiScale>>,
);

/// Sizes the square map, the search results and the rows of the lists for the window and the
/// lists of the panel, when the window changed size, a list changed length or the screen was
/// just spawned. The window is measured in logical pixels of the interface: its size over the
/// interface scale. Without a window, as in a test, the smallest supported one is assumed.
pub(super) fn fit(
    (windows, fresh): Seen,
    (places, markers, scale): Lists,
    mut fields: Query<&mut Node, With<Field>>,
    (mut board, mut view): (ResMut<Board>, ResMut<MapView>),
    mut last: Local<Option<(Vec2, (usize, usize))>>,
) {
    let scale = scale.map_or(1.0, |scale| scale.0).max(f32::EPSILON);
    let size = |window: &Window| Vec2::new(window.width(), window.height()) / scale;
    let now = (
        windows.iter().next().map_or(SMALLEST, size),
        listed(places.as_deref(), markers.as_deref()),
    );
    if *last == Some(now) && fresh.is_empty() {
        return;
    }
    *last = Some(now);
    let fit = measure(now.0, now.1);
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
    if board.lists != Some(fit.lists) {
        board.lists = Some(fit.lists);
    }
}
