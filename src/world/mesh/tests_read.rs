//! Tests of what the colour of a triangle reads: the fixed scale of the cell and not the facet of the mesh,
//! the tone of the finest cells only, and the quarters of turf a triangle holds.

use bevy::math::Vec3;

use super::read::{level, read};
use super::style::{Style, tone};
use crate::universe::PlanetRecord;
use crate::world::lands::{Lands, Reading, Verdure};

/// Returns the lands of the home planet.
fn home() -> Lands {
    Lands::of(&PlanetRecord::TEMPERATE)
}

/// With a reading of its cell a triangle never looks at its facet, whatever the level of detail made it:
/// a coarse mesh paints a mountain as a fine one does.
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
}

/// Without a globe to read, on the legacy plane, a triangle reads its own height and its facet.
#[test]
fn the_legacy_plane_reads_the_facet() {
    let seen = read(7.0, None, || 0.9);
    assert_eq!(
        seen,
        Reading {
            height: 7.0,
            slope: 0.9
        }
    );
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
    let lands = home();
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
