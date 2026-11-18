//! Tests of the weather in time: smooth and repeatable, thunder, the season and the system.

use std::f64::consts::TAU;

use bevy::ecs::system::RunSystemOnce;
use bevy::prelude::*;

use super::rules::{Planet, Weather, forecast, thunder};
use super::season::season;
use super::tests::{climate, planet, times};
use crate::universe::{PlanetClass, Universe};

/// The weather changes slowly, over minutes and never by much in a second, comes back the
/// same for the same seed and second, and differs from one planet to the next.
#[test]
fn weather_is_smooth_and_repeats() {
    let home = planet(PlanetClass::Island, 1.0);
    let other = Planet { seed: 7, ..home };
    let wet = climate(285.0, 0.8, 5.0);
    let values = |w: Weather| [w.cloud, w.rain, w.snow, w.dust, w.storm];
    let (mut low, mut high) = (1.0_f32, 0.0_f32);
    for second in 0..7200 {
        let t = f64::from(second);
        let (now, next) = (forecast(&home, &wet, t), forecast(&home, &wet, t + 1.0));
        let step = values(now)
            .into_iter()
            .zip(values(next))
            .map(|(a, b)| (a - b).abs());
        assert!(step.fold(0.0, f32::max) < 0.03, "at {t}");
        assert_eq!(now, forecast(&home, &wet, t));
        (low, high) = (low.min(now.cloud), high.max(now.cloud));
    }
    assert!(high - low > 0.4);
    assert!(times().any(|t| forecast(&home, &wet, t) != forecast(&other, &wet, t)));
}

/// Thunder claps only out of a storm, the same way for the same slot.
#[test]
fn thunder_needs_a_storm() {
    let calm = Weather {
        cloud: 1.0,
        ..default()
    };
    let storm = Weather { storm: 1.0, ..calm };
    assert!((0..200).all(|slot| thunder(3, slot, &calm).is_none()));
    let claps = (0..200)
        .filter(|&slot| thunder(3, slot, &storm).is_some())
        .count();
    assert!((40..=160).contains(&claps), "{claps}");
    assert!((0..200).all(|slot| thunder(3, slot, &storm) == thunder(3, slot, &storm)));
}

/// Stand-in until `climate::season` lands: the season is the place on the orbit, from 0 to 1,
/// a quarter further after a quarter orbit.
#[test]
fn the_season_stand_in_follows_the_orbit() {
    let universe = Universe::new(4);
    let home = universe.planet(universe.home());
    let start = (home.orbit_phase / TAU).rem_euclid(1.0) as f32;
    assert!((season(&home, 0.0) - start).abs() < 1e-4);
    let quarter = season(&home, home.orbit_seconds / 4.0);
    assert!(((quarter - start).rem_euclid(1.0) - 0.25).abs() < 1e-3);
    assert!((0..100).all(|k| (0.0..1.0).contains(&season(&home, f64::from(k) * 1e5))));
}

/// Away from any planet the weather system leaves the sky clear.
#[test]
fn the_weather_system_is_clear_in_space() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins).add_plugins(super::plugin);
    app.world_mut().insert_resource(Weather {
        rain: 1.0,
        ..default()
    });
    app.world_mut()
        .run_system_once(super::watch::watch)
        .unwrap();
    assert_eq!(*app.world().resource::<Weather>(), Weather::default());
}
