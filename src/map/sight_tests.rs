//! Tests of the sight of the flat map: the closest zoom shows about 500 m of an Earth, and a mark
//! far from the middle of the chart is drawn on its own pixel there, as is the place under the
//! pointer.

use std::f64::consts::{PI, TAU};

use bevy::math::{DVec2, Vec2};

use super::Sight;
use crate::computer::{SIZE, direction};
use crate::map::geo::{copy_near, to_chart};
use crate::map::level::{CLOSEST, REFERENCE_MAP};
use crate::universe::EARTH_RADIUS;

/// At the closest zoom the map of a 1280 by 720 window spans about 500 m of a planet the size of
/// Earth, under a metre a pixel.
#[test]
fn the_closest_zoom_shows_about_five_hundred_metres() {
    let across = TAU * EARTH_RADIUS / f64::from(CLOSEST);
    assert!((400.0..600.0).contains(&across), "{across} m");
    let pixel = across / f64::from(REFERENCE_MAP);
    assert!(pixel < 1.0, "{pixel} m a pixel");
}

/// Far from the middle of the chart, by the turn of longitude and at a high latitude, a mark ten
/// metres east of the centre of the view is drawn where ten metres fall, within a twentieth of a
/// pixel of a 720 pixel map, and the pixel of a mark gives the mark back.
#[test]
fn a_mark_far_from_the_chart_origin_sits_on_its_pixel() {
    let scale = REFERENCE_MAP / SIZE;
    for (lon, lat) in [(0.000_01, 0.31), (TAU - 0.000_02, -0.62), (PI * 0.73, 1.2)] {
        let centre = to_chart(direction(lon, lat));
        let lens = Sight {
            zoom: CLOSEST,
            centre,
        };
        let east = 10.0 / (EARTH_RADIUS * lat.cos());
        let mark = copy_near(to_chart(direction(lon + east, lat)), centre.x);
        let drawn = (lens.to_map(mark) - Vec2::splat(SIZE * 0.5)) * scale;
        let wanted = east / PI * f64::from(CLOSEST) * 0.5 * f64::from(REFERENCE_MAP);
        assert!(
            (f64::from(drawn.x) - wanted).abs() < 0.05,
            "{lon} {lat}: {drawn} for {wanted}"
        );
        assert!(drawn.y.abs() < 0.05, "{lon} {lat}: {drawn}");
        let back = lens.to_chart(lens.to_map(mark));
        let gap = (back - mark) * f64::from(CLOSEST) * 0.5 * f64::from(REFERENCE_MAP);
        assert!(gap.length() < 0.01, "{lon} {lat}: {gap}");
    }
}

/// A sight eases to its goal and snaps onto it once close, and a drag at the closest zoom moves
/// the view by what the pointer moved, not by a step of an f32 chart point.
#[test]
fn a_sight_eases_and_pans_by_the_pixel() {
    let goal = Sight {
        zoom: CLOSEST,
        centre: DVec2::new(0.999_9, 0.2),
    };
    let start = Sight {
        zoom: 8.0,
        centre: DVec2::ZERO,
    };
    let eased = (0..200).fold(start, |sight, _| sight.toward(goal, 0.2));
    assert_eq!(eased, goal);
    let moved = crate::map::roam::slid(goal, Vec2::new(0.1, 0.0));
    let step = (goal.centre.x - moved.centre.x) * f64::from(CLOSEST) * 0.5 * f64::from(SIZE);
    assert!((step - 0.1).abs() < 1e-6, "{step}");
}
