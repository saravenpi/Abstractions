//! Tests of the colours of the lands: the hue of each class and its match with the look seen from afar.

use bevy::color::{LinearRgba, Srgba};
use bevy::math::Vec3;

use super::Lands;
use super::tests::lands;
use crate::universe::PlanetClass::*;
use crate::world::biome::Biome;
use crate::world::surface::Surface;

/// Every surface, in table order.
const SURFACES: [Surface; 8] = [
    Surface::Water,
    Surface::Shallows,
    Surface::Reef,
    Surface::Pond,
    Surface::Sand,
    Surface::Grass,
    Surface::Rock,
    Surface::Snow,
];

/// Returns every colour the ground of `lands` can take, each with the surface it paints.
fn palette(lands: Lands) -> Vec<(Surface, Vec3)> {
    let spots = SURFACES
        .into_iter()
        .flat_map(|s| Biome::ALL.map(|b| (s, b)));
    let both = spots.flat_map(|(s, b)| [-8.0, 12.0].map(|h| (s, b, h)));
    both.map(|(s, b, h)| (s, Vec3::from(lands.rgb(s, b, h))))
        .collect()
}

/// Returns the linear colour of an sRGB one.
fn linear(c: Vec3) -> Vec3 {
    let l = LinearRgba::from(Srgba::rgb(c.x, c.y, c.z));
    Vec3::new(l.red, l.green, l.blue)
}

/// No ground colour of a desert, an ice, a lava or a barren world is green: green never leads both others.
#[test]
fn no_bare_class_has_green_ground() {
    for class in [Desert, Ice, Lava, Barren] {
        for ocean in [0.0, 0.4] {
            for (surface, c) in palette(lands(class, ocean)) {
                let green = c.y > c.x && c.y > c.z;
                assert!(!green, "{class:?} paints {surface:?} green: {c}");
            }
        }
    }
}

/// Each class has the hue it should: warm sand, pale blue ice, near black basalt and grey regolith.
#[test]
fn each_class_has_its_hue() {
    for (surface, c) in palette(lands(Desert, 0.0)) {
        assert!(
            c.x > c.y && c.y > c.z,
            "desert {surface:?} is not sand: {c}"
        );
    }
    for (surface, c) in palette(lands(Ice, 0.0)) {
        assert!(c.z >= c.y && c.y >= c.x, "ice {surface:?} is not blue: {c}");
        assert!(c.min_element() > 0.4, "ice {surface:?} is dark: {c}");
    }
    for (surface, c) in palette(lands(Barren, 0.0)) {
        let spread = c.max_element() - c.min_element();
        assert!(spread < 0.03, "barren {surface:?} has a hue: {c}");
    }
    let basalt = palette(lands(Lava, 0.3));
    let dry = basalt
        .iter()
        .filter(|(s, _)| matches!(s, Surface::Sand | Surface::Rock));
    for (surface, c) in dry {
        assert!(c.max_element() < 0.45, "lava {surface:?} is not black: {c}");
    }
}

/// The open ground of every bare class is the land colour of its look, and its dry basins the sea colour.
#[test]
fn the_ground_matches_the_look_seen_from_afar() {
    for class in [Desert, Ice, Lava, Barren] {
        let (lands, look) = (lands(class, 0.0), class.look());
        let open = linear(Vec3::from(lands.rgb(Surface::Sand, Biome::Desert, 9.0)));
        assert!((open - look.land).length() < 0.01, "{class:?}: {open}");
        let basin = linear(Vec3::from(lands.rgb(Surface::Sand, Biome::Desert, -9.0)));
        let want = if class == Lava { look.land } else { look.sea };
        assert!((basin - want).length() < 0.12, "{class:?}: {basin}");
        let rock = Vec3::from(lands.rgb(Surface::Rock, Biome::Desert, 9.0));
        assert!(linear(rock).length() < open.length());
    }
    let lava = linear(Vec3::from_array(lands(Lava, 0.3).sea_rgb()));
    assert!((lava - Lava.look().sea).length() < 0.01 && lava.x > 3.0 * lava.y);
}
