//! Far view tests: a planet seen from afar shows its class, and its seed only moves it a little.

use bevy::prelude::*;

use super::material::{Far, far};
use crate::universe::{PlanetClass, PlanetRecord, Water};

/// Linear colour of the sea the far view of every planet was built on before planets had a record.
const OLD_SEA: Vec3 = Vec3::new(0.05, 0.22, 0.42);

/// Returns a record a planet of this class could have.
fn record(class: PlanetClass) -> PlanetRecord {
    let (pressure, ocean, water) = match class {
        PlanetClass::Ocean => (1.2, 0.92, Water::Liquid),
        PlanetClass::Island => return PlanetRecord::TEMPERATE,
        PlanetClass::Desert => (0.4, 0.0, Water::None),
        PlanetClass::Ice => (0.7, 0.0, Water::Ice),
        PlanetClass::Lava => (3.0, 0.25, Water::Steam),
        PlanetClass::Barren => (0.0, 0.0, Water::None),
    };
    PlanetRecord {
        class,
        pressure,
        ocean,
        water,
        ..PlanetRecord::TEMPERATE
    }
}

/// Returns the far view of a class for a handful of seeds.
fn views(class: PlanetClass) -> Vec<Far> {
    (0..48u64)
        .map(|seed| seed.wrapping_mul(0x5851_F42D_4C95_7F2D) ^ 7)
        .map(|seed| far(&record(class), seed))
        .collect()
}

/// Returns the brightness of a linear colour.
fn brightness(color: Vec3) -> f32 {
    color.dot(Vec3::new(0.2126, 0.7152, 0.0722))
}

/// An ocean world is blue, the home world a blue green close to the sea colour it had, a desert
/// sand, an ice world white with a touch of blue and a barren one grey, and none of them glows.
#[test]
fn far_planets_show_their_class() {
    for Far { color, .. } in views(PlanetClass::Ocean) {
        assert!(color.z > 1.5 * color.y && color.y > color.x, "{color}");
    }
    for Far { color, .. } in views(PlanetClass::Island) {
        assert!(
            color.z > 2.0 * color.x && color.y > 2.0 * color.x,
            "{color}"
        );
        assert!(color.distance(OLD_SEA) < 0.2, "{color}");
    }
    for Far { color, .. } in views(PlanetClass::Desert) {
        assert!(
            color.x > 1.3 * color.y && color.y > 1.5 * color.z,
            "{color}"
        );
    }
    for Far { color, .. } in views(PlanetClass::Ice) {
        assert!(color.min_element() > 0.65 && color.z > color.x, "{color}");
    }
    for Far { color, .. } in views(PlanetClass::Barren) {
        let spread = color.max_element() - color.min_element();
        assert!(
            spread < 0.04 && (0.2..0.4).contains(&brightness(color)),
            "{color}"
        );
    }
    for class in PlanetClass::ALL {
        let glows = views(class).iter().any(|view| view.ember != Vec3::ZERO);
        assert_eq!(glows, class == PlanetClass::Lava, "{class:?}");
    }
}

/// A lava world is dark and red by day, and its molten sea still shines on the night side,
/// brighter than the day side of the rock, even when no pool shows from afar.
#[test]
fn lava_glows_on_the_night_side() {
    let dark = brightness(PlanetClass::Lava.look().land);
    for Far { color, ember } in views(PlanetClass::Lava) {
        assert!(
            brightness(color) < 0.25 && color.x > 2.0 * color.y,
            "{color}"
        );
        assert!(ember.x > 3.0 * ember.y && ember.y > ember.z, "{ember}");
        assert!(
            brightness(ember) > dark && ember.max_element() < 1.0,
            "{ember}"
        );
    }
    let dry = PlanetRecord {
        ocean: 0.0,
        ..record(PlanetClass::Lava)
    };
    assert!(far(&dry, 3).ember.x > 0.05);
}

/// The same planet always looks the same, two planets of one class differ, and the seed moves
/// the colour by a small share only.
#[test]
fn seed_moves_the_far_view_a_little() {
    for class in PlanetClass::ALL {
        let all = views(class);
        assert_eq!(all, views(class), "{class:?}");
        let mean = all.iter().map(|view| view.color).sum::<Vec3>() / all.len() as f32;
        let mut apart = 0;
        for (view, next) in all.iter().zip(&all[1..]) {
            let off = view.color.distance(mean);
            assert!(off < 0.3 * mean.length() + 0.02, "{class:?}: {off}");
            apart += usize::from(view.color != next.color);
        }
        assert!(apart > 40, "{class:?}: {apart}");
    }
}

/// The share of the surface under liquid moves the far colour from the land to the sea.
#[test]
fn ocean_share_mixes_land_and_sea() {
    let look = PlanetClass::Island.look();
    let with = |ocean: f64| {
        let record = PlanetRecord {
            ocean,
            ..PlanetRecord::TEMPERATE
        };
        far(&record, 11).color
    };
    let shares: Vec<Vec3> = (0..=10).map(|i| with(f64::from(i) / 10.0)).collect();
    for pair in shares.windows(2) {
        assert!(pair[1].z >= pair[0].z);
        assert!(pair[1].distance(look.sea) <= pair[0].distance(look.sea));
    }
    assert!(shares[0].y > shares[0].z && shares[10].z > shares[10].y);
}
