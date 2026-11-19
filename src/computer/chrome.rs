//! Keeping the screen in shape: the square map and the search results follow the window.

use bevy::prelude::*;
use bevy::window::PrimaryWindow;

use super::board::Board;
use super::layout::{SMALLEST, measure};
use super::map::Field;
use super::pointer::MapView;
use super::view::Panel;
use crate::map::{FAVOURITES, Favourites, PLAYERS};
use crate::menus::{MarkerKind, Markers};

/// The window, and the screen when it was just spawned.
type Seen<'w, 's> = (
    Query<'w, 's, &'static Window, With<PrimaryWindow>>,
    Query<'w, 's, (), Added<Panel>>,
);

/// Returns how many rows the favourites and the players lists of the panel show, none for a
/// list that does not exist.
fn listed(favourites: Option<&Favourites>, markers: Option<&Markers>) -> (usize, usize) {
    let saved = favourites.map_or(0, |favourites| favourites.0.len());
    let friends = markers.map_or(0, |markers| {
        let kinds = markers.0.iter().map(|marker| marker.kind);
        kinds.filter(|kind| *kind == MarkerKind::Friend).count()
    });
    (saved.min(FAVOURITES), friends.min(PLAYERS))
}

/// Sizes the square map and the search results for the window and the lists of the panel,
/// when the window changed size, a list changed length or the screen was just spawned.
/// Without a window, as in a test, the smallest supported one is assumed.
pub(super) fn fit(
    (windows, fresh): Seen,
    (favourites, markers): (Option<Res<Favourites>>, Option<Res<Markers>>),
    mut fields: Query<&mut Node, With<Field>>,
    (mut board, mut view): (ResMut<Board>, ResMut<MapView>),
    mut last: Local<Option<(Vec2, (usize, usize))>>,
) {
    let size = |window: &Window| Vec2::new(window.width(), window.height());
    let now = (
        windows.iter().next().map_or(SMALLEST, size),
        listed(favourites.as_deref(), markers.as_deref()),
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
}
