//! Computer tests: the favourites and the players of the side panel give way to the search and
//! the top of the card in a short window.

use bevy::math::Vec2;

use super::layout::{FACTS, ROW, SMALLEST, fixed, measure};

/// At the smallest window and at 1280 by 720 the search, one result, the top of the card with
/// its buttons and the rows of the lists drawn always fit: the facts of the card are cut first,
/// then the lists lose their last rows. At 540 two favourites and a player all show, and at 720
/// five favourites and three players. Prints the measures.
#[test]
fn the_lists_give_way_after_the_facts() {
    for window in [SMALLEST, Vec2::new(1280.0, 720.0)] {
        for lists in [(0, 0), (2, 1), (5, 0), (3, 3), (1, 6), (5, 6)] {
            let fit = measure(window, lists);
            let used = fixed(fit.lists) - FACTS + fit.rows as f32 * ROW;
            println!("{window} with {lists:?}: {fit:?}");
            assert!(used <= window.y, "{window} {lists:?}: {used}");
            assert!(fit.lists.0 <= lists.0 && fit.lists.1 <= lists.1);
        }
    }
    assert_eq!(measure(SMALLEST, (2, 1)).lists, (2, 1));
    assert_eq!(measure(Vec2::new(1280.0, 720.0), (5, 3)).lists, (5, 3));
}
