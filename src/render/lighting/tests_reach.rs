//! Tests of the reach of lamps and of the cascades of the shadow settings.

use bevy::light::CascadeShadowConfig;

use super::quality::cascades;
use super::reach::reach;
use super::tune::{FIRST_BOUND, LOCAL_FLOOR_LUX, MOON_SHADOW_LUX};
use crate::settings::{ShadowDetail, ShadowReach};

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

/// Every reach ends its last cascade where it says, starts with the 12 m cascade of the robot on
/// High with bounds rising by at most 4.6, and drops one cascade on Low detail.
#[test]
fn the_cascades_end_at_the_reach() {
    let counts = [
        (ShadowReach::Near, 2, 1),
        (ShadowReach::Normal, 3, 2),
        (ShadowReach::Far, 4, 3),
    ];
    for (reach, high, low) in counts {
        let CascadeShadowConfig { bounds, .. } = cascades(reach, ShadowDetail::High);
        assert_eq!((bounds.len(), reach.cascades()), (high, high));
        assert!((bounds[bounds.len() - 1] - reach.metres()).abs() < 1.0e-3);
        assert!((bounds[0] - FIRST_BOUND).abs() < 1.0e-3);
        assert!(
            bounds
                .windows(2)
                .all(|pair| pair[1] > pair[0] && pair[1] < 4.6 * pair[0])
        );
        let low_bounds = cascades(reach, ShadowDetail::Low).bounds;
        assert_eq!(
            (low_bounds.len(), reach.cascades_for(ShadowDetail::Low)),
            (low, low)
        );
        assert!((low_bounds[low_bounds.len() - 1] - reach.metres()).abs() < 1.0e-3);
    }
    let normal = cascades(ShadowReach::Normal, ShadowDetail::High).bounds;
    assert!((normal[1] - 36.3).abs() < 0.1, "{normal:?}");
}

/// Returns the size in metres of one texel of the first cascade on a map of `side` texels, for a
/// camera with the game's 75 degree vertical field of view on a 16 by 9 picture: the cascade is as
/// wide as the far face of the first slice of the view, as Bevy sizes it.
fn first_texel(side: f32) -> f32 {
    let half_high = FIRST_BOUND * (75.0f32.to_radians() / 2.0).tan();
    let half_wide = half_high * 16.0 / 9.0;
    2.0 * half_high.hypot(half_wide) / side
}

/// The first cascade gives the robot and the companion texels under 2 cm on a High map and under
/// 4 cm on a Low one, so the 5 by 5 filter of the shadow spans less than a limb on High.
#[test]
fn the_first_cascade_draws_limbs() {
    let (high, low) = (
        first_texel(ShadowDetail::High.map_size() as f32),
        first_texel(ShadowDetail::Low.map_size() as f32),
    );
    assert!(high < 0.02 && low < 0.04, "{high} {low}");
    assert!(5.0 * high < 0.1);
}
