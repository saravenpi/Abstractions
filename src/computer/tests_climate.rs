//! Computer tests: the climate of the card, its season words, its span of temperatures and the
//! greys of the climate map.

use std::collections::HashSet;

use super::climap::{BYTES, COLS, ROWS, SEA, WHITE, grid, mapped, tone};
use super::season::{climate_line, orbit_season, season_words, span_words};
use crate::universe::{Orbit, PlanetClass, PlanetSpec, Universe};
use crate::world::Biome;

/// Returns the home planet of seed 7.
pub(super) fn home() -> PlanetSpec {
    let universe = Universe::new(7);
    universe.planet(universe.home())
}

/// The north goes through summer, autumn, winter and spring as the year turns, and a planet
/// whose axis stands straight, either way up, has no seasons.
#[test]
fn season_words_follow_the_phase_and_the_tilt() {
    let tilt = 0.41;
    let phases = [(0.0, "summer"), (0.25, "autumn"), (0.5, "winter")];
    for (phase, word) in phases
        .into_iter()
        .chain([(0.75, "spring"), (0.95, "summer")])
    {
        assert_eq!(
            season_words(tilt, phase),
            format!("northern {word}"),
            "{phase}"
        );
    }
    assert_eq!(season_words(0.0, 0.0), "no seasons");
    assert_eq!(season_words(std::f64::consts::PI, 0.5), "no seasons");
    assert_eq!(season_words(2.7, 0.5), "northern winter");
}

/// The season is the angle along the orbit around the star: the planet's own, or for a moon
/// the orbit of its planet.
#[test]
fn the_season_is_the_place_on_the_orbit_around_the_star() {
    let planet = PlanetSpec {
        orbit_phase: 0.0,
        orbit_seconds: 400.0,
        ..home()
    };
    assert!(orbit_season(&planet, 0.0).abs() < 1e-6);
    assert!((orbit_season(&planet, 100.0) - 0.25).abs() < 1e-6);
    assert!((orbit_season(&planet, 700.0) - 0.75).abs() < 1e-6);
    let host = Orbit {
        radius: 1.0e9,
        seconds: 800.0,
        phase: std::f64::consts::PI,
    };
    let moon = PlanetSpec {
        host: Some(host),
        ..planet
    };
    assert!((orbit_season(&moon, 200.0) - 0.75).abs() < 1e-6);
}

/// The span reads from the coldest to the warmest place, as one value when both round alike,
/// and a moon that always shows its planet one face says its near side is warm.
#[test]
fn the_card_writes_the_span_of_temperatures() {
    assert_eq!(span_words((261.15, 304.15)), "-12 C to 31 C");
    assert_eq!(span_words((288.0, 288.3)), "15 C everywhere");
    let planet = PlanetSpec {
        orbit_phase: 0.0,
        ..home()
    };
    let shown = climate_line(&planet, 0.5);
    assert!(shown.starts_with("northern winter - "), "{shown}");
    assert!(
        shown.ends_with(" C") || shown.ends_with(" everywhere"),
        "{shown}"
    );
    let mut moon = planet.clone();
    (moon.id.moon, moon.day_seconds) = (1, -600.0);
    assert!(climate_line(&moon, 0.5).starts_with("near side warm - "));
}

/// The map has one pixel per cell, all of them grey: black sea, white snow or one of the six
/// biome greys, the home planet showing both sea and land. A gas giant has no map.
#[test]
fn the_climate_map_is_a_grid_of_greys() {
    let pixels = grid(&home(), 0.25);
    assert_eq!(pixels.len(), COLS * ROWS * BYTES);
    let mut allowed: HashSet<u8> = Biome::ALL.into_iter().map(tone).collect();
    assert_eq!(allowed.len(), 6);
    allowed.extend([SEA, WHITE]);
    for pixel in pixels.chunks(BYTES) {
        assert!(pixel[0] == pixel[1] && pixel[1] == pixel[2], "{pixel:?}");
        assert_eq!(pixel[3], u8::MAX);
        assert!(allowed.contains(&pixel[0]), "{pixel:?}");
    }
    let greys: HashSet<u8> = pixels.chunks(BYTES).map(|pixel| pixel[0]).collect();
    assert!(greys.contains(&SEA) && greys.len() > 1, "{greys:?}");
    let mut giant = home();
    giant.record.class = PlanetClass::GasGiant;
    assert!(!mapped(&giant) && mapped(&home()));
}
