//! Tests for the player colours.

use super::{PALETTE, player_color};

/// Sixty-four consecutive ids reach every colour, and an id always gets the same one.
#[test]
fn ids_spread_over_every_colour() {
    let used: Vec<_> = (0..64).map(player_color).collect();
    assert!(PALETTE.iter().all(|c| used.contains(c)));
    assert_eq!(player_color(7), player_color(7));
}
