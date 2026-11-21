//! Tests of the spread of tools put down at one spot: a second tool goes beside the first, around
//! the feet, never onto it.

use bevy::math::DVec3;

use super::lay::clear_of;
use crate::universe::BodyId;

/// Returns the spots ahead of a robot at the origin, one metre out, at each of `turns` radians.
fn around(turns: &[f64]) -> Vec<(BodyId, DVec3)> {
    let spot = |turn: &f64| (BodyId::default(), DVec3::new(turn.sin(), 0.0, turn.cos()));
    turns.iter().map(spot).collect()
}

/// With the spot ahead free the tool lies ahead; with a tool there the next tool lies at the first
/// turn its length away from it; with every spot taken it lies ahead after all.
#[test]
fn a_second_tool_lies_beside_the_first() {
    let spots = around(&[0.0, 0.8, -0.8, 1.6]);
    let ahead = spots[0];
    assert_eq!(clear_of(spots.clone().into_iter(), 1.0, &[]), Some(ahead));
    let one = [ahead];
    assert_eq!(
        clear_of(spots.clone().into_iter(), 0.5, &one),
        Some(spots[1])
    );
    assert_eq!(
        clear_of(spots.clone().into_iter(), 1.2, &one),
        Some(spots[3])
    );
    let other = [(
        BodyId {
            index: 3,
            ..BodyId::default()
        },
        ahead.1,
    )];
    assert_eq!(
        clear_of(spots.clone().into_iter(), 1.0, &other),
        Some(ahead)
    );
    assert_eq!(
        clear_of(spots.clone().into_iter(), 1.0, &spots),
        Some(ahead)
    );
}
