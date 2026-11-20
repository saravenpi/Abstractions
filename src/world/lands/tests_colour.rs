//! Tests of the one colour of the ground: the surface follows the reading and the biome as the rules of
//! the lands say, a channel bed takes the place of the surface above the sea only, and a carpet of ground
//! cover moves grass to the mean colour of its turf.

use bevy::color::{LinearRgba, Srgba};
use bevy::math::Vec3;

use super::colour::{Reading, Verdure};
use super::tests::{GROUND, lands};
use crate::universe::PlanetClass::*;
use crate::world::biome::{Biome, ground_rgb};
use crate::world::surface::{Surface, classify};

/// On a world of seas and islands the colour is the palette of home for the surface the rules give, at
/// every height, slope and biome.
#[test]
fn the_colour_is_the_palette_of_the_surface() {
    let home = lands(Island, 0.6);
    let cases = Biome::ALL.into_iter().flat_map(|b| GROUND.map(|g| (b, g)));
    for (biome, (height, slope)) in cases {
        let reading = Reading { height, slope };
        let (rgb, surface) = home.colour(reading, [biome, biome], None);
        assert_eq!(surface, classify(height, slope, biome));
        assert_eq!(rgb, ground_rgb(surface, biome));
    }
}

/// A channel bed paints the land above the sea and never the floor under it.
#[test]
fn a_bed_paints_only_above_the_sea() {
    let home = lands(Island, 0.6);
    let dry = Reading {
        height: 5.0,
        slope: 0.1,
    };
    let wet = Reading {
        height: -5.0,
        slope: 0.1,
    };
    let bed = Some(Surface::River);
    assert_eq!(home.colour(dry, [Biome::Meadow; 2], bed).1, Surface::River);
    assert_ne!(home.colour(wet, [Biome::Meadow; 2], bed).1, Surface::River);
}

/// The same reading has the same colour whoever asks: the colour never depends on the mesh.
#[test]
fn a_reading_has_one_colour() {
    let home = lands(Island, 0.6);
    let reading = Reading {
        height: 45.0,
        slope: 0.4,
    };
    let first = home.colour(reading, [Biome::Forest, Biome::Meadow], None);
    assert_eq!(
        first,
        home.colour(reading, [Biome::Forest, Biome::Meadow], None)
    );
}

/// A carpet leaves bare grass alone, moves it to the mean colour of the turf where it covers all the
/// ground, and in between the grass lies between the two.
#[test]
fn a_carpet_moves_grass_to_the_mean_of_its_turf() {
    let mean = Vec3::new(0.05, 0.3, 0.02);
    let verdure = Verdure { mean, cover: 0.5 };
    let grass = (0.3, 0.6, 0.2);
    let bare = verdure.carpeted(grass, 0.0);
    assert!((Vec3::from(bare) - Vec3::from(grass)).abs().max_element() < 1.0e-4);
    let full = verdure.carpeted(grass, 1.0);
    let shown = Srgba::from(LinearRgba::rgb(mean.x, mean.y, mean.z));
    let want = Vec3::new(shown.red, shown.green, shown.blue);
    assert!((Vec3::from(full) - want).abs().max_element() < 1.0e-4);
    let half = Vec3::from(verdure.carpeted(grass, 0.5));
    assert!(half.y < grass.1 && half.y > want.y);
}
