//! Computer tests: the favourites and the players of the side panel give way to the search and
//! the top of the card in a short window.

use bevy::math::Vec2;

use super::layout::{FACTS, ROW, SMALLEST, fixed, measure};

/// At the smallest layout, which a 960 by 540 window shares with 1280 by 720, the search, one
/// result, the whole card in its compact form and the rows of the lists drawn always fit: the
/// lists give way to the card, so the smallest layout draws no pins, and a tall window draws
/// them all. Prints the measures.
#[test]
fn the_lists_give_way_to_the_card() {
    for window in [SMALLEST, Vec2::new(1920.0, 1080.0)] {
        for lists in [(0, 0), (2, 1), (5, 0), (3, 3), (1, 6), (5, 6)] {
            let fit = measure(window, lists);
            let used = fixed(fit.lists) - FACTS + fit.rows as f32 * ROW;
            println!("{window} with {lists:?}: {fit:?}");
            assert!(used <= window.y, "{window} {lists:?}: {used}");
            assert!(fit.lists.0 <= lists.0 && fit.lists.1 <= lists.1);
        }
    }
    assert_eq!(measure(SMALLEST, (3, 3)).lists, (0, 0));
    assert_eq!(measure(Vec2::new(1920.0, 1080.0), (5, 3)).lists, (5, 3));
}
