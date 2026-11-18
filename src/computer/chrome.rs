//! Keeping the panel in shape: its size follows the window, and its footer offers the actions
//! that make sense for the row under the cursor.

use bevy::prelude::*;
use bevy::window::PrimaryWindow;

use super::board::Board;
use super::course::offered;
use super::layout::{SMALLEST, map_side, panel_width, rows_that_fit};
use super::map::Field;
use super::pointer::MapView;
use super::route::Destination;
use super::view::{Act, Panel};

/// The nodes the window sizes: the panel and the map in it.
type Sized = Or<(With<Panel>, With<Field>)>;

/// Sizes the panel, the square map and the list for the window. Without a window, as in a
/// test, the smallest supported one is assumed.
pub(super) fn fit(
    windows: Query<&Window, With<PrimaryWindow>>,
    mut nodes: Query<(&mut Node, Has<Field>), Sized>,
    mut board: ResMut<Board>,
    mut view: ResMut<MapView>,
) {
    let size = |window: &Window| Vec2::new(window.width(), window.height());
    let window = windows.iter().next().map_or(SMALLEST, size);
    let (wide, side) = (px(panel_width(window.x)), px(map_side(window)));
    for (mut node, map) in &mut nodes {
        let wanted = if map {
            (side, side)
        } else {
            (wide, node.height)
        };
        if (node.width, node.height) != wanted {
            (node.width, node.height) = wanted;
        }
    }
    if view.side != map_side(window) {
        view.side = map_side(window);
    }
    let rows = rows_that_fit(window);
    if board.fit != rows {
        board.fit = rows;
        board.step(0);
    }
}

/// Shows the actions of the footer that apply and hides the others.
pub(super) fn offer(
    board: Res<Board>,
    course: Res<Destination>,
    mut actions: Query<(&Act, &mut Node)>,
) {
    let cursor = board.lines.get(board.cursor).map(|line| line.target);
    for (act, mut node) in &mut actions {
        let display = match offered(*act, cursor, course.0) {
            true => Display::Flex,
            false => Display::None,
        };
        if node.display != display {
            node.display = display;
        }
    }
}
