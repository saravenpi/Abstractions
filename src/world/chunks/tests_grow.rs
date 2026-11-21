//! Tests of the path the quadtree is grown along: a focus far from the sea level sphere, in a basin or on a
//! mountain, grows its nodes down to the leaves as a focus on a plain does.

use bevy::math::DVec3;

use super::sight::Height;
use crate::world::globe::quadtree::{View, leaf_at};

/// A focus 12 m over a basin floor 4.4 km under the sea level sphere splits the leaf under it, which it does
/// not when it is measured from the sphere, and a focus on a plain is not moved.
#[test]
fn a_focus_in_a_basin_splits_the_nodes_under_it() {
    let radius = 7.0e6;
    let view = |focus: DVec3| View {
        focus,
        radius,
        deepest: 17,
        reach: 5.0 * radius * std::f64::consts::FRAC_PI_2 / f64::from(1u32 << 17),
        budget: 100,
        split: 2.0,
    };
    let node = leaf_at(DVec3::Y, 16);
    let basin = Height {
        altitude: -4463.0,
        clearance: 12.0,
        radius,
    };
    let at = DVec3::Y * (radius + basin.altitude);
    assert!(!view(at).splits(node));
    assert!(view(at - DVec3::Y * basin.drop()).splits(node));
    let plain = Height {
        altitude: 12.0,
        clearance: 12.0,
        radius,
    };
    assert_eq!(plain.drop(), 0.0);
}
