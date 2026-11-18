//! Tests of the weather in time: smooth and repeatable, thunder, the system and its feed, and
//! the gas giant, which has only storm wind.

use bevy::ecs::system::RunSystemOnce;
use bevy::prelude::*;

use super::field::Field;
use super::rules::{Weather, thunder};
use super::tests::{Sky, climate, forecast, most, planet, times};
use crate::universe::PlanetClass;

/// The weather changes slowly, over minutes and never by much in a second, comes back the
/// same for the same seed and second, and differs from one planet to the next.
#[test]
fn weather_is_smooth_and_repeats() {
    let home = planet(PlanetClass::Island, 1.0);
    let other = Sky {
        field: Field {
            seed: 7,
            ..home.field
        },
        ..home
    };
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
        assert!(step.fold(0.0, f32::max) < 0.05, "at {t}");
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

/// The HUD says the strongest thing that is happening, and the sky gets the weather only while
/// the camera is in the air of a planet.
#[test]
fn the_feed_names_the_weather_and_keeps_orbit_clear() {
    let calm = Weather::default();
    assert_eq!(super::word(&calm), "clear");
    let grey = Weather { cloud: 0.9, ..calm };
    assert_eq!(super::word(&grey), "overcast");
    let wet = Weather {
        rain: 0.6,
        storm: 0.4,
        ..grey
    };
    assert_eq!(super::word(&wet), "rain");
    let wild = Weather { storm: 0.9, ..wet };
    assert_eq!(super::word(&wild), "storm");
    let mut app = App::new();
    app.init_resource::<crate::render::LocalWeather>()
        .init_resource::<crate::menus::HudWeather>()
        .insert_resource(wet)
        .add_systems(Update, super::feed::feed);
    app.update();
    assert_eq!(
        app.world().resource::<crate::render::LocalWeather>().0,
        None
    );
    assert_eq!(
        app.world().resource::<crate::menus::HudWeather>().word,
        "rain"
    );
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
