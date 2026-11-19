//! Tests of the weather rules: no weather without air, snow only in the cold, dust only where
//! dry and windy. The tests in time and of the gas giant are in `tests_time`.
//! The tests of bare ground, of the calm opening at home and of the feed are in `tests_ground`,
//! and the tests of the cloud field over a real planet in `tests_field`.

use bevy::math::DVec3;
use bevy::prelude::*;

use super::field::{Field, Overhead};
use super::rules::{self, Planet, Weather};
use crate::climate::Climate;
use crate::universe::{HOME_RADIUS, PlanetClass, PlanetRecord};

/// A planet as these tests read its weather: its ground and its cloud field.
#[derive(Debug, Clone, Copy, PartialEq)]
pub(super) struct Sky {
    /// What the weather rules read of the planet.
    pub ground: Planet,
    /// The cloud field over it.
    pub field: Field,
}

/// The planet-fixed direction of the one place these tests watch the weather at.
const PLACE: DVec3 = DVec3::new(0.6, 0.48, 0.64);

/// Returns the weather over [`PLACE`] of `sky` where the climate is `at`, at `seconds`, under
/// the cloud band: the cover and the storm of the field there, and what the rules make of them.
pub(super) fn forecast(sky: &Sky, at: &Climate, seconds: f64) -> Weather {
    let overhead = Overhead {
        cover: sky.field.raw(at.moisture, PLACE, seconds),
        storm: sky.field.storm(PLACE, seconds),
        share: sky.field.share(seconds),
        under: 1.0,
    };
    rules::forecast(&sky.ground, at, &overhead)
}

/// Seconds of universe time the tests look at: two hours, one reading every 7 seconds.
pub(super) fn times() -> impl Iterator<Item = f64> {
    (0..1000).map(|step| f64::from(step) * 7.3)
}

/// Returns a climate of `temperature` kelvin and `moisture`, with a prevailing wind of `wind`
/// metres per second toward the east.
pub(super) fn climate(temperature: f32, moisture: f32, wind: f32) -> Climate {
    Climate {
        temperature,
        moisture,
        wind: Vec2::new(wind, 0.0),
    }
}

/// Returns the planet of the home record changed into `class` with air of `pressure`, of the
/// size of the home planet and of seed 42.
pub(super) fn planet(class: PlanetClass, pressure: f64) -> Sky {
    let record = PlanetRecord {
        class,
        pressure,
        ..PlanetRecord::TEMPERATE
    };
    Sky {
        ground: Planet::of(&record),
        field: Field::new(&record, 42, HOME_RADIUS),
    }
}

/// Returns the greatest value `pick` takes over [`times`].
pub(super) fn most(planet: &Sky, at: &Climate, pick: impl Fn(&Weather) -> f32) -> f32 {
    times()
        .map(|t| pick(&forecast(planet, at, t)))
        .fold(0.0, f32::max)
}

/// Without air the sky is clear and still on every class, whatever the climate says.
#[test]
fn no_weather_without_air() {
    for class in PlanetClass::ALL {
        for pressure in [0.0, 0.005] {
            let bare = planet(class, pressure);
            let wild = climate(280.0, 1.5, 30.0);
            assert!(times().all(|t| forecast(&bare, &wild, t) == Weather::default()));
        }
    }
    let home = planet(PlanetClass::Island, 1.0);
    assert!(most(&home, &climate(290.0, 1.0, 0.0), |w| w.rain) > 0.3);
}

/// Snow falls only below freezing and rain only above it, and both fall in a wet climate.
#[test]
fn snow_only_when_cold() {
    let home = planet(PlanetClass::Island, 1.0);
    for temperature in [230.0, 260.0, 274.0] {
        assert_eq!(
            most(&home, &climate(temperature, 1.0, 0.0), |w| w.rain),
            0.0
        );
    }
    for temperature in [276.0, 290.0, 320.0] {
        assert_eq!(
            most(&home, &climate(temperature, 1.0, 0.0), |w| w.snow),
            0.0
        );
    }
    assert!(most(&home, &climate(260.0, 1.0, 0.0), |w| w.snow) > 0.3);
    let lava = planet(PlanetClass::Lava, 1.0);
    assert_eq!(most(&lava, &climate(290.0, 1.0, 0.0), |w| w.rain), 0.0);
}

/// Dust rises only where the ground is dry and the wind strong: never in a wet climate, only
/// in a storm when the air is calm, and on a windy desert world.
#[test]
fn dust_only_where_dry_and_windy() {
    let home = planet(PlanetClass::Island, 1.0);
    let desert = planet(PlanetClass::Desert, 1.0);
    assert_eq!(most(&home, &climate(300.0, 1.0, 30.0), |w| w.dust), 0.0);
    for t in times() {
        let calm = forecast(&home, &climate(300.0, 0.0, 0.0), t);
        assert!(calm.dust == 0.0 || calm.storm > 0.4, "{calm:?}");
    }
    assert!(most(&desert, &climate(300.0, 0.3, 15.0), |w| w.dust) > 0.3);
}
