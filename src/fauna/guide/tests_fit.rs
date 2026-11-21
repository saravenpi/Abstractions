//! Tests that the guide fits the smallest and the usual window: the header, the list rows and
//! the footer stack inside the height, the list and the card sit side by side inside the width,
//! and the tallest card of real planets fits beside the list.

use bevy::math::Vec2;

use super::book::book;
use super::card::card;
use super::layout::{BETWEEN, EDGE, FOOT, HEAD, LIST, ROW, chars, height, logical, measure};
use super::rig::lives;
use super::seen::Seen;

/// The windows the guide must fit, in pixels: both lay out as 1280 by 720.
const WINDOWS: [Vec2; 2] = [Vec2::new(960.0, 540.0), Vec2::new(1280.0, 720.0)];

/// The list rows, the header and the footer fit the height, and the card keeps room for its
/// facts beside the list.
#[test]
fn the_guide_fits_both_windows() {
    for window in WINDOWS.map(logical) {
        let fit = measure(window);
        let tall = 2.0 * EDGE + HEAD + fit.rows as f32 * ROW + FOOT;
        assert!(tall <= window.y, "{window}: {fit:?}");
        assert!(fit.rows >= 12, "{window}: {fit:?}");
        assert!(
            2.0 * EDGE + LIST + BETWEEN + fit.card <= window.x,
            "{window}: {fit:?}"
        );
        assert!(chars(fit.card) >= 60, "{window}: {fit:?}");
    }
}

/// Every card of the living home planets of 40 universes, every species seen, fits the room the
/// smallest window leaves.
#[test]
fn every_card_fits_the_smallest_window() {
    let room = measure(logical(WINDOWS[0], crate::ui::ui_scale_for(WINDOWS[0].y))).room;
    let mut tallest = 0.0f32;
    for life in lives(40) {
        let mut seen = Seen::default();
        life.species.iter().for_each(|s| {
            seen.mark(s.id);
        });
        for entry in book(Some(&life), &seen).entries {
            tallest = tallest.max(height(&card(Some(&life), Some(entry.index), &seen)));
        }
    }
    assert!(
        tallest > 0.0 && tallest <= room,
        "tallest {tallest} room {room}"
    );
}
