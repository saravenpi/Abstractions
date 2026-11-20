//! The colours of the ground: a painted patch takes its texels from the palette of the full
//! map, with its relief, on planets of every class.

use bevy::math::DVec3;

use crate::map::raster::colour;
use crate::menus::hud::minimap::paint::{BYTES, paint};
use crate::menus::hud::minimap::patch::{Patch, TEXELS};
use crate::universe::{PlanetClass, PlanetRecord};
use crate::world::Globe;

/// Returns a planet of `class` with `ocean` of its surface under its liquid, the patch 4.2 km
/// wide on it that the tests paint, and the image of that patch.
fn painted(class: PlanetClass, ocean: f64) -> (Globe, Patch, Vec<u8>) {
    let record = PlanetRecord {
        class,
        ocean,
        ..PlanetRecord::TEMPERATE
    };
    let globe = Globe::new(7, 6000.0, 22.0).with_record(record);
    let patch = Patch::around(DVec3::new(0.6, 0.3, -0.74).normalize(), 6000.0, 1050.0);
    let pixels = paint(&globe, &patch);
    assert_eq!(pixels.len(), TEXELS * TEXELS * BYTES);
    (globe, patch, pixels)
}

/// Returns how far the strongest and the weakest channel of `texel` lie apart.
fn spread(texel: &[u8]) -> u8 {
    let rgb = &texel[..3];
    rgb.iter().max().unwrap() - rgb.iter().min().unwrap()
}

/// Every texel of the image is the colour of the full map at that place, fully opaque.
#[test]
fn a_texel_has_the_colour_of_the_full_map() {
    let (globe, patch, pixels) = painted(PlanetClass::Island, PlanetRecord::BALANCED_OCEAN);
    for (x, y) in [
        (0, 0),
        (10, 70),
        (48, 48),
        (TEXELS - 1, 33),
        (90, TEXELS - 1),
    ] {
        let first = (y * TEXELS + x) * BYTES;
        let wanted = colour(&globe, patch.dir(x, y));
        assert_eq!(pixels[first..first + BYTES], wanted, "({x}, {y})");
        assert_eq!(pixels[first + 3], 255);
    }
}

/// A world with seas and land shows both, in different hues: the picture is not grey.
#[test]
fn a_world_of_seas_is_coloured() {
    let (_, _, pixels) = painted(PlanetClass::Island, PlanetRecord::BALANCED_OCEAN);
    let texels = pixels.as_chunks::<BYTES>().0;
    let tinted = texels.iter().filter(|texel| spread(*texel) > 20).count();
    assert!(tinted > texels.len() / 2, "{tinted} tinted texels");
    let blue = texels
        .iter()
        .filter(|t| t[2] > t[0].saturating_add(15) && t[2] > t[1])
        .count();
    let green = texels
        .iter()
        .filter(|t| t[1] > t[2].saturating_add(15) && t[1] > t[0])
        .count();
    assert!(blue > 500 && green > 500, "{blue} blue, {green} green");
}

/// Every class paints its own colours from its own planet, with no special case for one
/// world: a dry world and an ice world are tinted and differ from the island.
#[test]
fn every_class_is_painted_from_its_own_ground() {
    let mean = |pixels: &[u8]| {
        let texels = pixels.as_chunks::<BYTES>().0;
        let sum = |channel: usize| {
            texels
                .iter()
                .map(|t| usize::from(t[channel]))
                .sum::<usize>()
        };
        [sum(0), sum(1), sum(2)].map(|total| total / texels.len())
    };
    let (_, _, island) = painted(PlanetClass::Island, PlanetRecord::BALANCED_OCEAN);
    for class in [PlanetClass::Desert, PlanetClass::Ice, PlanetClass::Lava] {
        let (_, _, pixels) = painted(class, 0.0);
        assert_ne!(mean(&pixels), mean(&island), "{class:?}");
        assert!(
            pixels.as_chunks::<BYTES>().0.iter().any(|t| spread(t) > 10),
            "{class:?}"
        );
    }
}
