//! Fitting the page of the settings to the window: the column of rows is as tall as the window
//! has room for at the current interface scale, and scrolls beyond that.

use bevy::prelude::*;
use bevy::window::PrimaryWindow;

use super::view::VISIBLE_ROWS;
use crate::ui::theme;

/// Fewest rows of a page visible at once, whatever the size of the interface.
pub const MIN_ROWS: f32 = 6.0;

/// Height in logical pixels of everything in the settings screen but the rows of the page: the
/// margins, the panel padding, the heading, the line under the rows and the key hints.
pub const CHROME: f32 = 240.0;

/// Marks the column that scrolls the pages of the settings.
#[derive(Component)]
pub(super) struct Pages;

/// Returns how many rows of a page show at once in a window `height` logical pixels high: as
/// many as fit, from [`MIN_ROWS`] to [`VISIBLE_ROWS`], so the page scrolls instead of running
/// off the screen when the interface is large.
pub fn rows_for(height: f32) -> f32 {
    ((height - CHROME) / theme::ROW_HEIGHT)
        .floor()
        .clamp(MIN_ROWS, VISIBLE_ROWS)
}

/// Sets how tall the page column is for the room the window has at the current interface scale.
pub(super) fn fit_page(
    window: Single<&Window, With<PrimaryWindow>>,
    scale: Res<UiScale>,
    mut pages: Query<&mut Node, With<Pages>>,
) {
    let wanted = px(theme::ROW_HEIGHT * rows_for(window.height() / scale.0.max(f32::EPSILON)));
    for mut node in &mut pages {
        if node.max_height != wanted {
            node.max_height = wanted;
        }
    }
}
