//! Tests of the large shape of the ground where it meets the local ground: the local ground draws every
//! coast, and the large height never steps.

use bevy::math::DVec3;

use super::tests_planet::planet;
use super::tests_record::sphere_dirs;
use super::{Globe, Grain};
use crate::universe::{PlanetClass, Water};
use crate::world::SEA_LEVEL;

/// Returns the direction between `sea` and `land` where the large height of `globe` crosses the sea
/// level, found by halving.
fn coast(globe: &Globe, mut sea: DVec3, mut land: DVec3) -> DVec3 {
    for _ in 0..60 {
        let mid = (sea + land).normalize();
        match globe.large_height(mid).expect("plates") > 0.0 {
            true => land = mid,
            false => sea = mid,
        }
    }
    land
}

/// The local ground draws the coast of a large planet: where the large height is within the fringe of the
/// sea the ground is the local ground alone, with its coast measure pushed, and beyond the fringe it is
/// land on the land side and sea floor on the sea side. A coast found between a sea site and a land site
/// lies in the fringe.
#[test]
fn the_local_ground_draws_the_coast() {
    let globe = planet(PlanetClass::Island, 0.6, Water::Liquid);
    let local = |dir: DVec3, bias: f32| {
        let p = (dir * globe.radius()).to_array();
        globe.shape.measure_on(p, &globe.climate, bias).0
    };
    let (mut sea, mut land) = (None, None);
    for dir in sphere_dirs(20_000) {
        let lift = globe.relief.lift(dir).expect("plates");
        let height = globe.height_seen(dir, Grain::Far);
        match lift.bias {
            b if b >= 1.0 => land = Some(dir),
            b if b <= -1.0 => sea = Some(dir),
            b => assert_eq!(height, local(dir, b), "{dir}"),
        }
        assert!(lift.bias < 1.0 || height > SEA_LEVEL, "sea inland at {dir}");
        assert!(
            lift.bias > -1.0 || height < SEA_LEVEL,
            "land far out at {dir}"
        );
    }
    let shore = coast(&globe, sea.expect("a sea site"), land.expect("a land site"));
    let lift = globe.relief.lift(shore).expect("plates");
    assert!(lift.bias.abs() < 0.01 && lift.height == 0.0, "{lift:?}");
    assert_eq!(
        globe.height_seen(shore, Grain::Far),
        local(shore, lift.bias)
    );
}

/// The large height has no step: 10 m along the ground never change it by more than 10 m, on plate
/// edges, at junctions and on the flanks of volcanoes alike.
#[test]
fn the_large_shape_is_continuous() {
    let globe = planet(PlanetClass::Island, 0.6, Water::Liquid);
    for (k, dir) in sphere_dirs(20_000).enumerate() {
        let (east, north) = dir.any_orthonormal_pair();
        let angle = k as f64 * 0.7;
        let step = (east * angle.cos() + north * angle.sin()) * (10.0 / globe.radius());
        let (a, b) = (
            globe.large_height(dir),
            globe.large_height((dir + step).normalize()),
        );
        let change = (a.expect("plates") - b.expect("plates")).abs();
        assert!(
            change < 10.0,
            "the large height steps by {change} m at {dir}"
        );
    }
}
