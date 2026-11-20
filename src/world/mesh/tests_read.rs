//! Tests of what the colour of a triangle reads: the fixed scale of the cell and not the facet of the mesh,
//! the tone of the finest cells only, and the quarters of turf a triangle holds.

use bevy::math::Vec3;

use super::read::{level, read, rock_share, scale};
use super::style::{Style, tone};
use super::tune::{COARSE, FINE};
use crate::universe::PlanetRecord;
use crate::world::lands::{Lands, Reading, Verdure};

/// With a reading of its cell, which only cells of [`COARSE`] metres or more have, a triangle never looks at
/// its facet: a coarse mesh paints a mountain as the map does.
#[test]
fn a_triangle_with_a_reading_never_reads_its_facet() {
    let cell = Reading {
        height: 20.0,
        slope: 0.3,
    };
    let seen = read(18.0, Some(cell), || panic!("the facet was read"));
    assert_eq!(seen, cell);
}

/// Under the sea the drawn height stands, so a coast follows the mesh, and over it the land is never read
/// at or under the sea level.
#[test]
fn the_coast_follows_the_drawn_height() {
    let cell = Reading {
        height: 6.0,
        slope: 0.3,
    };
    assert_eq!(read(-3.0, Some(cell), || 0.0).height, -3.0);
    let low = Reading {
        height: -2.0,
        ..cell
    };
    assert!(read(1.5, Some(low), || 0.0).height > 0.0);
    let plane = read(7.0, None, || 0.9);
    assert_eq!((plane.height, plane.slope), (7.0, 0.9));
}

/// A triangle holds the quarters of turf of the mean thickness of its corners, rounded.
#[test]
fn a_triangle_holds_quarters_of_turf() {
    let thick = |t: f32| level(&[[0.0, 0.0, t]; 3]);
    assert_eq!(
        (thick(0.0), thick(0.1), thick(0.5), thick(0.9)),
        (0.0, 0.0, 2.0, 4.0)
    );
    assert_eq!((thick(-1.0), thick(7.0)), (0.0, 4.0));
}

/// The tone of a triangle stays between 0.92 and 1.08 and averages 1, so toning the finest cells and not
/// the coarse ones leaves the mean colour of a place the same at every level.
#[test]
fn the_tone_has_zero_mean() {
    let tones: Vec<f32> = (0..64)
        .flat_map(|i| (0..64).flat_map(move |j| [tone(i, j, 0), tone(i, j, 1)]))
        .collect();
    assert!(tones.iter().all(|t| (0.92..=1.08).contains(t)));
    let mean = tones.iter().sum::<f32>() / tones.len() as f32;
    assert!((mean - 1.0).abs() < 0.005, "{mean}");
}

/// Only cells no wider than the reach of the tone are toned, and the ground cover is carried along.
#[test]
fn only_the_finest_cells_are_toned() {
    let lands = Lands::of(&PlanetRecord::TEMPERATE);
    assert!(Style::detail(lands).toned && Style::coarse(4.0, lands).toned);
    assert!(!Style::coarse(8.0, lands).toned);
    let wide = Style::detail(lands).grown(8.0, None);
    assert!(!wide.toned);
    let verdure = Verdure {
        mean: Vec3::splat(0.2),
        cover: 0.5,
    };
    let carpet = Style::detail(lands).grown(2.0, Some(verdure));
    assert!(carpet.toned && carpet.verdure == Some(verdure));
}

/// A 4 m cell on a 60 degree step inside a gentle hill reads its own facet and is rock, while the 125 m
/// slope of the same hill is gentle and no rock.
#[test]
fn a_small_cliff_is_rock_up_close() {
    let step = 60.0_f32.to_radians().tan();
    let close = read(10.0, None, || step);
    assert!(close.slope > 1.0);
    assert_eq!(rock_share(close.slope), 1.0);
    let hill = 0.2;
    assert_eq!(rock_share(hill), 0.0);
}

/// A 150 m cell over a hillside whose 125 m slope is at the rock angle is the blend of rock and ground, and
/// the share rises smoothly with the slope.
#[test]
fn a_coarse_hillside_is_a_blend() {
    assert!((rock_share(1.0) - 0.5).abs() < 1.0e-6);
    let shares: Vec<f32> = (0..=20).map(|k| rock_share(k as f32 * 0.1)).collect();
    assert!(shares.windows(2).all(|w| w[1] >= w[0]));
    assert!(shares[0] == 0.0 && shares[20] == 1.0);
}

/// Cells of 125 m and 1 km at one direction read the same: the scale is capped, so they give one reading
/// and one colour.
#[test]
fn wide_cells_at_one_place_share_one_reading() {
    let reading = Reading {
        height: 20.0,
        slope: 0.9,
    };
    assert_eq!((scale(1.0), scale(4.0), scale(125.0)), (FINE, 4.0, COARSE));
    assert_eq!(scale(125.0), scale(1000.0));
    let (a, b) = (
        read(18.0, Some(reading), || 0.0),
        read(25.0, Some(reading), || 0.0),
    );
    assert_eq!(a, b);
    assert_eq!(rock_share(a.slope), rock_share(b.slope));
}
