//! Tests of the weather rules: no weather without air, snow only in the cold, dust only where
//! dry and windy, nothing but storm wind on a gas giant, smooth and repeatable in time.
//! The tests of bare ground, of the calm opening at home and of the feed are in `tests_ground`.

use bevy::prelude::*;

use super::rules::{Planet, Weather, forecast};
use crate::climate::Climate;
use crate::universe::{PlanetClass, PlanetRecord};

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

/// Returns the planet of the home record changed into `class` with air of `pressure`.
pub(super) fn planet(class: PlanetClass, pressure: f64) -> Planet {
    let record = PlanetRecord {
        class,
        pressure,
        ..PlanetRecord::TEMPERATE
    };
    Planet::of(&record, 42)
}

/// Returns the greatest value `pick` takes over [`times`].
pub(super) fn most(planet: &Planet, at: &Climate, pick: impl Fn(&Weather) -> f32) -> f32 {
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

/// A gas giant has no deck for rain, snow, dust or clouds: only storm wind.
#[test]
fn a_gas_giant_has_only_storm_wind() {
    let giant = planet(PlanetClass::GasGiant, 3.0);
    for t in times() {
        let sky = forecast(&giant, &climate(150.0, 1.0, 20.0), t);
        assert_eq!(
            (sky.cloud, sky.rain, sky.snow, sky.dust),
            (0.0, 0.0, 0.0, 0.0)
        );
    }
    assert!(most(&giant, &climate(150.0, 1.0, 20.0), |w| w.storm) > 0.3);
}
