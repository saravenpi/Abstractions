//! Computer tests: the climate line of a planet, its season words and its span of temperatures.

use super::season::{climate_line, season_words, span_words};
use crate::climate::season as orbit_season;
use crate::universe::{Orbit, PlanetRecord, PlanetSpec, Universe};

/// Returns the home planet of seed 7 made a temperate world tilted like the Earth that turns in a
/// day, so that the test does not depend on which planet the seed chooses as home.
pub(super) fn home() -> PlanetSpec {
    let universe = Universe::new(7);
    PlanetSpec {
        record: PlanetRecord::TEMPERATE,
        day_seconds: 86_400.0,
        orbit_seconds: 365.0 * 86_400.0,
        ..universe.planet(universe.home())
    }
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
        day_seconds: 86_400.0,
        ..home()
    };
    let shown = climate_line(&planet, 0.5);
    assert!(shown.starts_with("northern winter - "), "{shown}");
    assert!(
        shown.ends_with(" C") || shown.ends_with(" everywhere"),
        "{shown}"
    );
    let mut moon = planet.clone();
    (moon.id.moon, moon.day_seconds) = (1, moon.orbit_seconds);
    assert!(climate_line(&moon, 0.5).starts_with("near side warm - "));
}
