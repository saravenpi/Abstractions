//! Tests of how fast a leaf sinks.

use super::*;

/// A leaf sinks at about a metre per second at home; thin air and strong gravity make it sink faster, and
/// weak gravity slower.
#[test]
fn a_leaf_sinks_with_gravity_and_air() {
    let home_sink = leaf(0.1, 9.8, 1.0);
    assert!((0.7..1.1).contains(&home_sink), "{home_sink}");
    assert!(leaf(0.1, 9.8, 0.3) > home_sink && leaf(0.1, 20.0, 1.0) > home_sink);
    assert!(leaf(0.1, 3.0, 1.0) < home_sink);
}
