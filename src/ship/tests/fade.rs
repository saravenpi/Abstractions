//! Tests of the top of the air: its density eases on to zero over the last scale heights, so the
//! drag and the heat of a ship that climbs out of it fade and never jump.

use super::planet;
use crate::ship::drag::{TOP_FADE, density};

/// Under the fade the density is the exponential one, at the top it is zero, and from two scale
/// heights under the top to a little above it no ten metres change it by more than
/// three thousandths of the density where the fade starts.
#[test]
fn the_air_fades_out_at_its_top() {
    let planet = planet(6.371e6, 1.0e5, 86_400.0);
    let (top, scale) = (planet.atmosphere_height, planet.scale_height());
    let start = top - TOP_FADE * scale;
    let plain = 1.2 * (-(start - 100.0) / scale).exp();
    let seen = density(&planet, start - 100.0) / plain;
    assert!(
        (seen - density(&planet, 0.0) / 1.2).abs() < 1.0e-9,
        "{seen}"
    );
    assert_eq!(density(&planet, top), 0.0);
    let mut last = density(&planet, start - 10.0);
    let mut height = start;
    while height < top + 100.0 {
        let now = density(&planet, height);
        assert!(
            (now - last).abs() < 3.0e-3 * density(&planet, start),
            "at {height} m"
        );
        assert!(now <= last);
        (last, height) = (now, height + 10.0);
    }
}
