//! The fog from the air: shut on foot, open to beyond the horizon a few hundred metres up.

use super::atmosphere::air;
use super::fog::{Height, cull_distance, fog_sight};

/// Returns where the fog sees a camera `clearance` metres above the ground from, on a planet of
/// 3 km whose ground lies 200 m above the sea and whose air ends `top` metres up.
fn above(clearance: f32, top: f32) -> Height {
    let altitude = 200.0 + clearance;
    Height {
        altitude,
        clearance,
        radius: 3000.0,
        air: air(altitude, top),
    }
}

/// On foot the fog hides the edge of the detailed ground, even on a hill; from a few hundred
/// metres up it lets the horizon show; it opens smoothly and never closes while climbing, and
/// the view culls no closer than the fog ends.
#[test]
fn fog_opens_with_height_over_the_ground() {
    let ground = 230.0;
    assert!(fog_sight(ground, &above(2.0, 4000.0)) < ground * 1.2);
    let up = above(300.0, 4000.0);
    let horizon = (up.altitude * (2.0 * up.radius + up.altitude)).sqrt();
    assert!(fog_sight(ground, &up) > horizon * 2.0);
    let sights: Vec<f32> = (0..=400)
        .map(|i| fog_sight(ground, &above(i as f32 * 2.5, 4000.0)))
        .collect();
    for pair in sights.windows(2) {
        assert!(pair[1] >= pair[0] && pair[1] < pair[0] * 1.06, "{pair:?}");
    }
    assert_eq!(cull_distance(ground), 1000.0);
    assert!(cull_distance(sights[400]) >= sights[400]);
}
