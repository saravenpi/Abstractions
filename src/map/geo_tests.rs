//! Tests of the places on a planet: the round trip of a direction through the flat map, and
//! the words of its latitude and longitude, and the map wrapping east and west.

use std::f64::consts::{FRAC_PI_2, PI};

use bevy::math::{DVec2, DVec3};

use super::geo::{coords, copy_near, from_chart, lat_lon, to_chart, wrapped};
use crate::computer::direction;

#[test]
fn latitude_and_longitude_undo_the_direction() {
    for (lon, lat) in [(0.0, 0.0), (1.0, 0.5), (4.0, -1.2), (6.0, 1.5)] {
        let (back_lat, back_lon) = lat_lon(direction(lon, lat));
        assert!((back_lat - lat).abs() < 1e-9 && (back_lon - lon).abs() < 1e-9);
    }
}

#[test]
fn the_flat_map_puts_north_up_and_east_to_the_right() {
    assert!(to_chart(direction(0.0, FRAC_PI_2 - 1e-6)).y > 0.49);
    assert!((to_chart(direction(PI, 0.0)) - DVec2::ZERO).length() < 1e-6);
    assert!(to_chart(direction(PI + 0.1, 0.0)).x > 0.0);
    let dir = direction(2.0, -0.7);
    let back = from_chart(to_chart(dir)).unwrap_or(DVec3::ZERO);
    assert!(back.distance(dir) < 1e-5);
    assert_eq!(from_chart(DVec2::new(0.0, 0.6)), None);
}

#[test]
fn coordinates_read_north_south_east_and_west() {
    let degree = PI / 180.0;
    assert_eq!(
        coords(direction(103.2 * degree, 12.4 * degree)),
        "12.4° N  103.2° E"
    );
    assert_eq!(
        coords(direction(-45.0 * degree, -30.0 * degree)),
        "30.0° S  45.0° W"
    );
}

/// A point just east of longitude 0 is drawn just east of a view on longitude 0 from the far
/// side, and a point of a wrapped view comes back onto the flat map.
#[test]
fn the_flat_map_wraps_at_the_date_line() {
    let east = DVec2::new(-0.95, 0.2);
    assert!(copy_near(east, 0.98).distance(DVec2::new(1.05, 0.2)) < 1e-6);
    assert_eq!(copy_near(east, -0.9), east);
    assert!(copy_near(east, 5.0).distance(DVec2::new(5.05, 0.2)) < 1e-5);
    assert!(wrapped(DVec2::new(1.05, 0.2)).distance(east) < 1e-6);
    assert!(wrapped(DVec2::new(-3.0, 0.0)).distance(DVec2::new(-1.0, 0.0)) < 1e-6);
}
