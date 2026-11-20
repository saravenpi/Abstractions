//! Tests of the reach of lamps and of the cascades of the shadow settings.

use bevy::light::CascadeShadowConfig;

use super::quality::cascades;
use super::reach::reach;
use super::tune::{LOCAL_FLOOR_LUX, MOON_SHADOW_LUX};
use crate::settings::ShadowReach;

/// The reach follows the inverse square law: four times the power doubles it, and at the reach the
/// lamp gives the floor in lux. The lamp of the robot, 850 lm, keeps its 3.5 m.
#[test]
fn the_reach_follows_the_inverse_square_law() {
    assert_eq!(reach(0.0), 0.0);
    assert_eq!(reach(-5.0), 0.0);
    assert!((reach(4.0 * 850.0) / reach(850.0) - 2.0).abs() < 1.0e-5);
    let lux = 850.0 / (4.0 * std::f32::consts::PI) / reach(850.0).powi(2);
    assert!((lux - LOCAL_FLOOR_LUX).abs() < 1.0e-3);
    assert!((reach(850.0) - 3.5).abs() < 0.05, "{}", reach(850.0));
    assert!(reach(60_000.0) > 25.0 && reach(60_000.0) < 35.0);
    const { assert!(LOCAL_FLOOR_LUX < MOON_SHADOW_LUX) };
}

/// Every reach ends its last cascade where it says, the far one has a third cascade, and the
/// bounds rise.
#[test]
fn the_cascades_end_at_the_reach() {
    for reach in [ShadowReach::Near, ShadowReach::Normal, ShadowReach::Far] {
        let CascadeShadowConfig { bounds, .. } = cascades(reach);
        assert_eq!(bounds.len(), reach.cascades());
        assert!((bounds[bounds.len() - 1] - reach.metres()).abs() < 1.0e-3);
        assert!(bounds.windows(2).all(|pair| pair[1] > pair[0]));
    }
    assert!((cascades(ShadowReach::Normal).bounds[0] - 28.0).abs() < 1.0e-3);
}
