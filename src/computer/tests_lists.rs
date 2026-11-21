//! Computer tests: the part of the side panel that scrolls. The pins and the players are all
//! listed whatever the window, since they scroll with the facts of the card, and the head of the
//! card and the keys never give way to them.

use bevy::math::Vec2;

use super::layout::{BODY, FIXED, RESULTS, ROW, SMALLEST, measure};
use crate::settings::InterfaceSize;
use crate::ui::{theme, ui_scale};

/// The tightest layout the game draws, in logical pixels: a window of 960 by 540 pixels at the
/// large interface size, which scales by 1.
pub(super) const TIGHTEST: Vec2 = Vec2::new(960.0, 540.0);

/// Returns the height in pixels the part of the panel that scrolls has in a window of `window`
/// logical pixels: what the fixed parts and the results drawn leave.
pub(super) fn body(window: Vec2) -> f32 {
    window.y - FIXED - measure(window).rows as f32 * ROW
}

/// Every window and interface size the game supports, as the logical size it lays out at.
pub(super) fn layouts() -> Vec<Vec2> {
    let windows = [
        Vec2::new(960.0, 540.0),
        Vec2::new(1280.0, 720.0),
        Vec2::new(1920.0, 1080.0),
        Vec2::new(2560.0, 1440.0),
    ];
    let sized = |window: Vec2| InterfaceSize::ALL.map(|size| window / ui_scale(window, size));
    windows.into_iter().flat_map(sized).collect()
}

/// At every window and interface size what never scrolls fits with the results drawn, and the
/// part that scrolls keeps the height that is left: at least three rows of facts in the tightest
/// layout, at least its nine rows from the reference layout up.
#[test]
fn the_scrolling_part_keeps_its_room_at_every_size() {
    let sizes = layouts();
    assert!(sizes.contains(&SMALLEST) && sizes.contains(&TIGHTEST));
    for window in sizes {
        let (fit, left) = (measure(window), body(window));
        println!("{window}: {fit:?}, {left} pixels scroll");
        assert!((RESULTS.0..=RESULTS.1).contains(&fit.rows), "{window}");
        assert!(left >= 3.0 * theme::LINE_SMALL, "{window}: {fit:?}");
        assert!(window.y < SMALLEST.y || left >= BODY, "{window}: {fit:?}");
    }
}
