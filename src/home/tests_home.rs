//! Tests of the home point: the one place the compass, the minimap, the map and the marker over
//! the world call home is on the container, not on the open ground at the base centre.

use bevy::math::DVec3;

use super::container::{self, HOME};
use super::tests::base;

/// Returns the horizontal distance in metres between two points near the base.
fn level(up: DVec3, a: DVec3, b: DVec3) -> f64 {
    let d = a - b;
    (d - up * d.dot(up)).length()
}

/// For seeds 0 to 9 the home point is the point of the container pose, to the millimetre, so
/// the icon sits on the building whatever the ground under its legs.
#[test]
fn the_home_point_is_on_the_container_for_ten_seeds() {
    for seed in 0..10 {
        let (base, globe) = base(seed);
        let lot = base.lot(&globe);
        let roof = lot.planet(container::pose(&lot).transform_point3(HOME));
        let off = base.home().distance(roof);
        assert!(off < 1e-3, "seed {seed}: home is {off} m off the roof");
    }
}

/// For seeds 0 to 9 the home point is within the box of the hatch, well away from the base
/// centre and from the pad, which keeps the marker of the ship.
#[test]
fn the_home_point_is_at_the_hatch_not_at_the_centre() {
    for seed in 0..10 {
        let (base, _) = base(seed);
        let home = base.home();
        let hatch = level(base.up, home, base.door);
        assert!(hatch < 2.0, "seed {seed}: {hatch} m from the hatch");
        let rise = (home - base.door).dot(base.up);
        assert!((2.9..3.1).contains(&rise), "seed {seed}: {rise} m up");
        let centre = level(base.up, home, base.centre);
        assert!(centre > 12.0, "seed {seed}: {centre} m from the centre");
        let pad = level(base.up, home, base.pad);
        assert!(
            (30.0..65.0).contains(&pad),
            "seed {seed}: {pad} m from the pad"
        );
    }
}
