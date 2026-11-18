//! Keeping the panel in shape: its width, its margins, its map and its list follow the window.

use bevy::prelude::*;
use bevy::window::PrimaryWindow;

use super::board::Board;
use super::layout::{Measures, SMALLEST, measure};
use super::map::Field;
use super::pointer::MapView;
use super::view::Panel;

/// The nodes the window sizes: the panel and the map in it.
type Sized = Or<(With<Panel>, With<Field>)>;

/// The window, and the panel when it was just spawned.
type Seen<'w, 's> = (
    Query<'w, 's, &'static Window, With<PrimaryWindow>>,
    Query<'w, 's, (), Added<Panel>>,
);

/// Returns the node `node` sized as the measures `fit` ask: the map square, the panel as wide
/// as its map and margins with the same margin around and between its sections.
fn sized(node: &Node, map: bool, fit: &Measures) -> Node {
    if map {
        return Node {
            width: px(fit.map),
            height: px(fit.map),
            ..node.clone()
        };
    }
    Node {
        width: px(fit.panel),
        row_gap: px(fit.margin),
        padding: UiRect::all(px(fit.margin)),
        ..node.clone()
    }
}

/// Sizes the panel, the square map and the list for the window, when the window changed size
/// or the panel was just spawned. Without a window, as in a test, the smallest supported one
/// is assumed.
pub(super) fn fit(
    (windows, fresh): Seen,
    mut nodes: Query<(&mut Node, Has<Field>), Sized>,
    mut board: ResMut<Board>,
    mut view: ResMut<MapView>,
    mut last: Local<Option<Measures>>,
) {
    let size = |window: &Window| Vec2::new(window.width(), window.height());
    let fit = measure(windows.iter().next().map_or(SMALLEST, size));
    if *last == Some(fit) && fresh.is_empty() {
        return;
    }
    *last = Some(fit);
    for (mut node, map) in &mut nodes {
        let wanted = sized(&node, map, &fit);
        if *node != wanted {
            *node = wanted;
        }
    }
    view.side = fit.map;
    if board.fit != fit.rows {
        board.fit = fit.rows;
        board.step(0);
    }
}
