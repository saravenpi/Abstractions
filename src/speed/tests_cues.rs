//! Tests of the speed cues: buffeting, condensation and the vapour cone.

use super::{buffet, condensation, cone, dynamic_pressure, moisture, vortex};

/// There is no buffeting without density, at any speed, and it grows with the dynamic pressure.
#[test]
fn buffeting_is_zero_without_air_and_grows_with_pressure() {
    assert_eq!(buffet(dynamic_pressure(0.0, 8000.0)), 0.0);
    assert_eq!(buffet(0.0), 0.0);
    let steps = [100.0, 500.0, 1500.0, 4000.0, 9000.0];
    let levels = steps.map(buffet);
    assert!(levels.windows(2).all(|pair| pair[0] <= pair[1]));
    assert!(levels[1] > 0.0 && levels[4] == 1.0);
}

/// Thin air at the same speed buffets less than thick air.
#[test]
fn thin_air_buffets_less() {
    let thick = buffet(dynamic_pressure(1.2, 60.0));
    let thin = buffet(dynamic_pressure(0.3, 60.0));
    assert!(thin < thick);
}

/// A wingtip vortex condenses in wet air and not in dry air.
#[test]
fn condensation_is_off_in_dry_air() {
    assert_eq!(condensation(0.0), 0.0);
    assert_eq!(vortex(0.0), 1.0);
    assert_eq!(condensation(1.0), 1.0);
    assert!(vortex(1.0) > vortex(0.4) && vortex(0.4) > 1.0);
}

/// The vapour cone shows around Mach 1 in humid air, and not slower, much faster, dry or in thin air.
#[test]
fn the_cone_shows_near_mach_one_in_wet_air_only() {
    assert!(cone(1.0, 1.0, 1.0) > 0.99);
    assert_eq!(cone(0.5, 1.0, 1.0), 0.0);
    assert_eq!(cone(2.0, 1.0, 1.0), 0.0);
    assert_eq!(cone(1.0, 1.0, 0.0), 0.0);
    assert_eq!(cone(1.0, 0.0, 1.0), 0.0);
}

/// The air is wet inside a cloud that covers the sky or in rain, and dry in a clear sky.
#[test]
fn wetness_comes_from_cloud_and_rain() {
    assert_eq!(moisture(1.0, 0.0, 0.0), 0.0);
    assert_eq!(moisture(0.0, 1.0, 0.0), 0.0);
    assert_eq!(moisture(0.5, 1.0, 0.0), 0.5);
    assert_eq!(moisture(0.0, 0.0, 0.8), 0.8);
    assert_eq!(moisture(1.0, 1.0, 2.0), 1.0);
}
