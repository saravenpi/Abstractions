//! The grey of the ground: what covers a texel, how each class of planet reads, and whole
//! patches painted on hand-built planets of the six classes.

use bevy::math::DVec3;

use crate::menus::hud::minimap::paint::{BYTES, paint};
use crate::menus::hud::minimap::patch::{Patch, TEXELS};
use crate::menus::hud::minimap::tone::{BANDS, Cover, cover, tone};
use crate::universe::{PlanetClass, PlanetRecord};
use crate::world::Globe;

/// Every cover a texel can have.
const COVERS: [Cover; 8] = [
    Cover::Deep,
    Cover::Shallow,
    Cover::Basin,
    Cover::Land(0),
    Cover::Land(1),
    Cover::Land(2),
    Cover::Land(3),
    Cover::Land(4),
];

/// Returns the greys of `cover` on a planet of `class` over a block of texels, lowest first
/// and each once: one grey for a flat cover, several for a patterned one.
fn greys(class: PlanetClass, cover: Cover) -> Vec<u8> {
    let cells = (0..10).flat_map(|y| (0..10).map(move |x| (x, y)));
    let mut seen: Vec<u8> = cells.map(|cell| tone(class, cover, cell)).collect();
    seen.sort_unstable();
    seen.dedup();
    seen
}

/// Ground under the sea level is liquid only where the planet has a sea, deep under a few
/// metres, and land rises through five bands.
#[test]
fn covers_follow_the_height_and_the_sea() {
    assert_eq!(
        (cover(-12.0, true), cover(-2.0, true)),
        (Cover::Deep, Cover::Shallow)
    );
    assert_eq!(
        (cover(-12.0, false), cover(0.0, false)),
        (Cover::Basin, Cover::Basin)
    );
    assert_eq!(
        (cover(0.4, true), cover(0.4, false)),
        (Cover::Land(0), Cover::Land(0))
    );
    let bands: Vec<Cover> = BANDS.iter().map(|edge| cover(edge + 0.1, true)).collect();
    let wanted = [
        Cover::Land(1),
        Cover::Land(2),
        Cover::Land(3),
        Cover::Land(4),
    ];
    assert_eq!(bands, wanted);
    assert_eq!(cover(120.0, false), Cover::Land(4));
}

/// On every class the land brightens with height, and a basin is dotted and darker than the
/// shore. Water is flat and darker than anything dry.
#[test]
fn land_rises_in_bands_over_dark_water() {
    for class in PlanetClass::ALL {
        let band = |band: u8| greys(class, Cover::Land(band))[0];
        let land: Vec<u8> = (0..5).map(band).collect();
        let rising = land.windows(2).all(|pair| pair[0] < pair[1]);
        assert!(rising, "{class:?} {land:?}");
        let basin = greys(class, Cover::Basin);
        let shore = greys(class, Cover::Land(0));
        let dotted = basin.len() >= 2 && basin[0] < shore[shore.len() - 1];
        assert!(dotted, "{class:?} {basin:?}");
    }
    let island = PlanetClass::Island;
    let (deep, shallow) = (greys(island, Cover::Deep), greys(island, Cover::Shallow));
    assert_eq!((deep.len(), shallow.len()), (1, 1));
    assert!(deep[0] < shallow[0] && shallow[0] < greys(island, Cover::Basin)[0]);
}

/// Ice is brighter than ordinary land and flaked, and molten rock is a checkerboard brighter
/// than everything on its dark world, cooler toward its shore.
#[test]
fn ice_and_molten_rock_read_by_brightness_and_pattern() {
    let (ice, lava, island) = (PlanetClass::Ice, PlanetClass::Lava, PlanetClass::Island);
    for band in 0..5 {
        let frozen = greys(ice, Cover::Land(band));
        let plain = greys(island, Cover::Land(band));
        assert_eq!((frozen.len(), plain.len()), (2, 1), "band {band}");
        assert!(frozen[1] > plain[0] + 20, "band {band}");
    }
    let (molten, edge) = (greys(lava, Cover::Deep), greys(lava, Cover::Shallow));
    let crust = greys(lava, Cover::Land(4));
    assert_eq!((molten.len(), edge.len(), crust.len()), (2, 2, 1));
    assert!(molten[0] > crust[0] + 100, "{molten:?} {crust:?}");
    assert!(edge[1] < molten[1] && edge[0] > crust[0] + 50, "{edge:?}");
}

/// Returns a planet of `class` with `ocean` of its surface under its liquid, and the image of
/// a patch 4.2 km wide on it as one grey per texel.
fn painted(class: PlanetClass, ocean: f64) -> (Globe, Vec<u8>) {
    let record = PlanetRecord {
        class,
        ocean,
        ..PlanetRecord::TEMPERATE
    };
    let globe = Globe::new(7, 6000.0, 22.0).with_record(record);
    let patch = Patch::around(DVec3::new(0.6, 0.3, -0.74).normalize(), 6000.0, 1050.0);
    let pixels = paint(&globe, &patch);
    assert_eq!(pixels.len(), TEXELS * TEXELS * BYTES);
    let grey = |texel: &[u8]| {
        assert_eq!((texel[0], texel[0], 255), (texel[1], texel[2], texel[3]));
        texel[0]
    };
    let greys = pixels
        .as_chunks::<BYTES>()
        .0
        .iter()
        .map(|texel| grey(texel))
        .collect();
    (globe, greys)
}

/// Returns how many texels of `greys` have one of the greys of `covers` on a planet of `class`.
fn count(greys: &[u8], class: PlanetClass, covers: &[Cover]) -> usize {
    let wanted: Vec<u8> = covers
        .iter()
        .flat_map(|cover| self::greys(class, *cover))
        .collect();
    greys.iter().filter(|grey| wanted.contains(grey)).count()
}

/// A world of seas shows its water, more of it the more the record gives it, and a dry world
/// shows none: its low ground is a dotted basin.
#[test]
fn a_dry_world_shows_no_liquid() {
    let wet = [Cover::Deep, Cover::Shallow];
    let (_, island) = painted(PlanetClass::Island, PlanetRecord::BALANCED_OCEAN);
    let (_, ocean) = painted(PlanetClass::Ocean, 0.9);
    let seas = (
        count(&island, PlanetClass::Island, &wet),
        count(&ocean, PlanetClass::Ocean, &wet),
    );
    assert!(seas.0 > 500 && seas.1 > seas.0, "{seas:?}");
    assert!(count(&island, PlanetClass::Island, &COVERS[3..]) > 500);
    for class in [PlanetClass::Desert, PlanetClass::Barren] {
        let (globe, dry) = painted(class, 0.0);
        assert!(!globe.has_sea());
        assert_eq!(count(&dry, class, &wet), 0, "{class:?}");
        assert!(count(&dry, class, &[Cover::Basin]) > 500, "{class:?}");
        assert_eq!(count(&dry, class, &COVERS[2..]), dry.len(), "{class:?}");
    }
}

/// A lava world shows its molten sea, the brightest thing on a dark crust, and a frozen world
/// reads as ice: bright all over, without any liquid.
#[test]
fn lava_shows_its_molten_sea_and_ice_reads_as_ice() {
    let (globe, lava) = painted(PlanetClass::Lava, 0.4);
    assert!(globe.has_sea());
    let molten = lava.iter().filter(|grey| **grey >= 160).count();
    let crust = lava.iter().filter(|grey| **grey <= 98).count();
    assert!(
        molten > 500 && crust > 500,
        "{molten} molten, {crust} crust"
    );
    assert_eq!(molten + crust, lava.len());
    let (_, ice) = painted(PlanetClass::Ice, 0.0);
    let (_, island) = painted(PlanetClass::Island, PlanetRecord::BALANCED_OCEAN);
    let mean =
        |greys: &[u8]| greys.iter().map(|grey| usize::from(*grey)).sum::<usize>() / greys.len();
    assert!(
        mean(&ice) > 135 && mean(&ice) > mean(&island) + 60,
        "{}",
        mean(&ice)
    );
    assert_eq!(
        count(&ice, PlanetClass::Ice, &[Cover::Deep, Cover::Shallow]),
        0
    );
    assert_eq!(count(&ice, PlanetClass::Ice, &COVERS[2..]), ice.len());
}
