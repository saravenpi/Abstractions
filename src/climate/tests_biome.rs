//! Biome rule tests: every branch, the jitter only blurring the inputs, the snow line and
//! the places Kodya puts each biome.

use bevy::math::Vec2;

use super::tests::{kodya, spot};
use super::*;
use crate::world::Biome;

/// Returns a still climate of `temperature` kelvin and wetness `moisture`.
fn air(temperature: f32, moisture: f32) -> Climate {
    Climate {
        temperature,
        moisture,
        wind: Vec2::ZERO,
    }
}

/// Each rule wins where it should, in order: cold, hot and dry, hot, wet, dry and cool, the rest.
#[test]
fn every_rule_has_its_biome() {
    let cases = [
        (250.0, 2.0, Biome::Taiga),
        (262.0, 0.0, Biome::Taiga),
        (305.0, 0.1, Biome::Desert),
        (303.0, 1.5, Biome::Tropical),
        (290.0, 1.5, Biome::Forest),
        (280.0, 0.1, Biome::Cherry),
        (295.0, 0.1, Biome::Meadow),
        (290.0, 0.28, Biome::Meadow),
    ];
    for (heat, wet, want) in cases {
        assert_eq!(biome(&air(heat, wet), 0.0), want, "{heat} K, wetness {wet}");
    }
}

/// The jitter never picks a biome: it is the biome of the moved inputs, and it never moves the
/// temperature by more than 1.5 K or the wetness by more than `exp(0.2)`.
#[test]
fn jitter_only_moves_the_inputs() {
    for heat in (240..330).step_by(3) {
        for wet in [0.0, 0.1, 0.3, 0.5, 0.8, 1.2, 2.0] {
            let place = air(heat as f32, wet);
            for jitter in [-1.0, -0.4, 0.0, 0.7, 1.0] {
                let moved = air(heat as f32 + 1.5 * jitter, wet * (0.2 * jitter).exp());
                assert_eq!(biome(&place, jitter), biome(&moved, 0.0));
            }
        }
    }
    assert_eq!(biome(&air(300.0, 1.0), 0.5), biome(&air(300.0, 1.0), 0.5));
}

/// Snow lies under 271 K and not above.
#[test]
fn snow_lies_under_the_snow_line() {
    assert!(snow(&air(270.9, 0.5)) && !snow(&air(271.0, 0.5)) && !snow(&air(300.0, 0.0)));
    assert_eq!(SNOW_LINE, 271.0);
}

/// On Kodya the poles and high ground are taiga, the wet equator tropical, the dry subtropics
/// inland desert, wet mid latitudes forest, and the rest meadow or cherry.
#[test]
fn kodya_puts_each_biome_in_its_place() {
    let home = kodya();
    let at_spot = |lat: f64, inland: f32, altitude: f32| {
        let place = Spot {
            inland,
            altitude,
            ..spot(lat, 40.0)
        };
        biome(&at(&home, &place, 0.25), 0.0)
    };
    assert_eq!(at_spot(85.0, 0.0, 0.0), Biome::Taiga);
    assert_eq!(at_spot(-80.0, 0.5, 0.0), Biome::Taiga);
    assert_eq!(at_spot(3.0, 0.0, 0.0), Biome::Tropical);
    assert_eq!(at_spot(12.0, 0.6, 0.0), Biome::Desert);
    assert_eq!(at_spot(48.0, 0.0, 0.0), Biome::Forest);
    assert_eq!(at_spot(48.0, 0.9, 0.0), Biome::Cherry);
    assert_eq!(at_spot(-25.0, 0.0, 0.0), Biome::Meadow);
    assert_eq!(at_spot(48.0, 0.0, 2500.0), Biome::Taiga);
}
