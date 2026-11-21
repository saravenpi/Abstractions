//! Tests of the spacing of the marks of the compass: icons at the same bearing are spread by the
//! least that clears them, around where they were, and stay on the bar.

use super::space::{BASE, inside, spread};
use crate::menus::hud::needle::WIDTH;

/// Five marks on one spot end a mark apart, centred on the spot, in the order they came.
#[test]
fn marks_on_one_spot_spread_evenly_around_it() {
    let mut at = [200.0; 5];
    spread(&mut at);
    assert!(
        at.windows(2)
            .all(|pair| (pair[1] - pair[0] - BASE).abs() < 0.01)
    );
    let mean = at.iter().sum::<f32>() / 5.0;
    assert!((mean - 200.0).abs() < 0.01);
}

/// Marks that already clear each other do not move, and a far one does not drag a near pair.
#[test]
fn marks_that_clear_each_other_stay() {
    let mut at = [40.0, 40.0 + BASE, 300.0];
    let before = at;
    spread(&mut at);
    assert_eq!(at, before);
}

/// A crowd at the end of the bar is pulled inside it, keeping its spacing.
#[test]
fn a_crowd_at_the_end_stays_on_the_bar() {
    let mut at = [WIDTH; 4];
    spread(&mut at);
    inside(&mut at);
    assert!(at[0] >= BASE * 0.5 - 0.01 && at[3] <= WIDTH - BASE * 0.5 + 0.01);
    assert!(at.windows(2).all(|pair| pair[1] - pair[0] >= BASE - 0.01));
}
