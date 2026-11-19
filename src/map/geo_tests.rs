//! Tests of the places on a planet: the round trip of a direction through the flat map, and
//! the words of its latitude and longitude.

use std::f64::consts::{FRAC_PI_2, PI};

use bevy::math::{DVec3, Vec2};

use super::geo::{coords, from_chart, lat_lon, to_chart};
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
    assert!((to_chart(direction(PI, 0.0)) - Vec2::ZERO).length() < 1e-6);
    assert!(to_chart(direction(PI + 0.1, 0.0)).x > 0.0);
    let dir = direction(2.0, -0.7);
    let back = from_chart(to_chart(dir)).unwrap_or(DVec3::ZERO);
    assert!(back.distance(dir) < 1e-5);
    assert_eq!(from_chart(Vec2::new(0.0, 0.6)), None);
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
