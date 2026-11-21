//! Tests that the guide fits the smallest and the usual window: the header, the list rows and
//! the footer stack inside the height, the list and the card sit side by side inside the width,
//! and the tallest card of real planets fits beside the list.

use bevy::math::Vec2;

use super::book::book;
use super::card::card;
use super::layout::{BETWEEN, EDGE, FOOT, HEAD, LIST, ROW, SMALL, chars, height, logical, measure};
use super::rig::lives;
use super::seen::Seen;
use super::view::HINTS;
use crate::ui::{HINT_GAP, KEY_GAP, theme};

/// The windows the guide must fit, in pixels: both lay out as 1280 by 720.
const WINDOWS: [Vec2; 2] = [Vec2::new(960.0, 540.0), Vec2::new(1280.0, 720.0)];

/// The list rows, the header and the footer fit the height, and the card keeps room for its
/// facts beside the list.
#[test]
fn the_guide_fits_both_windows() {
    for window in WINDOWS.map(|window| logical(window, crate::ui::ui_scale_for(window.y))) {
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

/// Every window and interface size the game supports, as the logical size it lays out at.
fn layouts() -> Vec<Vec2> {
    let windows = [
        Vec2::new(960.0, 540.0),
        Vec2::new(1280.0, 720.0),
        Vec2::new(1920.0, 1080.0),
        Vec2::new(2560.0, 1440.0),
    ];
    let sizes = crate::settings::InterfaceSize::ALL;
    let sized = |window: Vec2| sizes.map(|size| window / crate::ui::ui_scale(window, size));
    windows.into_iter().flat_map(sized).collect()
}

/// At every window and interface size the header, the rows of the list and the footer stack
/// inside the height, the list and the card sit side by side, the key hints fit one line, and
/// the name and the subtitle of the card, which never scroll, leave its facts at least ten rows.
#[test]
fn the_guide_fits_every_interface_size() {
    let hint = |(words, key): (&str, &str)| {
        let chars = (words.chars().count() + key.chars().count()) as f32;
        chars * theme::FONT_SMALL * theme::GLYPH_ASPECT + KEY_GAP
    };
    let legend = HINTS.iter().copied().map(hint).sum::<f32>() + 3.0 * HINT_GAP;
    let sizes = layouts();
    assert!(sizes.contains(&Vec2::new(960.0, 540.0)));
    for window in sizes {
        let fit = measure(window);
        assert!(
            2.0 * EDGE + HEAD + fit.rows as f32 * ROW + FOOT <= window.y,
            "{window}"
        );
        assert!(
            2.0 * EDGE + LIST + BETWEEN + fit.card <= window.x,
            "{window}"
        );
        assert!(legend <= window.x - 2.0 * EDGE, "{window}: {legend}");
        assert!(fit.rows >= 12 && chars(fit.card) >= 50, "{window}: {fit:?}");
        let head = theme::FONT_BODY * 1.2 + SMALL + theme::GAP_S;
        assert!(fit.room - head >= 10.0 * SMALL, "{window}: {fit:?}");
    }
}
