//! Tests of the weather over its ground and on the way down: dust only from bare ground, a calm
//! opening on the home planet, a storm only under its clouds, and a sky whose haze and dust
//! come in on the way down without a step.

use bevy::ecs::system::RunSystemOnce;
use bevy::prelude::*;

use super::feed::{local, seen};
use super::field::Field;
use super::rules::{Planet, Weather};
use super::tests::{Sky, climate, forecast, most, planet, times};
use crate::menus::HudWeather;
use crate::render::{LocalWeather, WeatherLook};
use crate::universe::{PlanetClass, Universe};

/// Returns `sky` over bare ground, which can give dust.
fn bare(sky: Sky) -> Sky {
    let ground = Planet {
        bare: true,
        ..sky.ground
    };
    Sky { ground, ..sky }
}

/// Dust never rises over a meadow, a forest or snow, whatever the wind and however dry the
/// air: there a gale is only wind. It rises over the desert biome of the same planet.
#[test]
fn no_dust_over_a_meadow_whatever_the_wind() {
    let green = planet(PlanetClass::Island, 1.0);
    assert!(!green.ground.bare);
    for (moisture, wind) in [(0.0, 15.0), (0.1, 40.0), (0.3, 100.0), (0.34, 30.0)] {
        let dry = climate(281.0, moisture, wind);
        assert_eq!(most(&green, &dry, |w| w.dust), 0.0, "{moisture} {wind}");
    }
    let sand = bare(green);
    assert!(most(&sand, &climate(305.0, 0.1, 15.0), |w| w.dust) > 0.3);
    assert_eq!(most(&sand, &climate(305.0, 0.1, 0.0), |w| w.rain), 0.0);
}

/// A gale lifts dust over the ground of a desert world and of a barren one, and a wet climate
/// keeps it down everywhere but on the desert world.
#[test]
fn dust_over_desert_with_a_gale() {
    let desert = planet(PlanetClass::Desert, 1.0);
    let barren = planet(PlanetClass::Barren, 1.0);
    assert!(desert.ground.bare && barren.ground.bare);
    let gale = climate(300.0, 0.05, 15.0);
    assert!(most(&desert, &gale, |w| w.dust) > 0.3);
    assert!(most(&barren, &gale, |w| w.dust) > 0.3);
    assert_eq!(most(&barren, &climate(300.0, 0.5, 30.0), |w| w.dust), 0.0);
    assert!(most(&desert, &climate(300.0, 0.5, 30.0), |w| w.dust) > 0.3);
    let still = times().map(|t| forecast(&desert, &climate(300.0, 0.05, 0.0), t));
    assert!(still.into_iter().all(|w| w.dust == 0.0 || w.storm > 0.4));
}

/// The first five minutes of a new game on the home planet are calm or lightly clouded for
/// seeds 0 to 20, in the wettest and windiest climate: no storm, rain, snow or dust. The
/// weather then comes in smoothly, and no other planet waits.
#[test]
fn the_first_minutes_at_home_are_calm() {
    let wild = [climate(285.0, 1.5, 30.0), climate(260.0, 1.5, 30.0)];
    for seed in 0..=20 {
        let universe = Universe::new(seed);
        let spec = universe.planet(universe.home());
        let away = bare(Sky {
            ground: Planet::of(&spec.record),
            field: Field::of(&spec, false),
        });
        let home = Sky {
            field: Field::of(&spec, true),
            ..away
        };
        for (second, at) in (0..=300).flat_map(|s| wild.iter().map(move |at| (s, at))) {
            let sky = forecast(&home, at, f64::from(second));
            let events = [sky.storm, sky.rain, sky.snow, sky.dust];
            assert_eq!(events, [0.0; 4], "seed {seed} at {second}");
            assert!(sky.cloud < 0.35 + 1.0e-6, "seed {seed} at {second}");
        }
        for second in 0..900 {
            let t = f64::from(second);
            let (now, next) = (
                forecast(&home, &wild[0], t),
                forecast(&home, &wild[0], t + 1.0),
            );
            assert!((now.cloud - next.cloud).abs() < 0.03 && (now.rain - next.rain).abs() < 0.03);
            assert!(second < 480 || now == forecast(&away, &wild[0], t));
        }
    }
}

/// A storm under a clear sky is wind: it does not haze the sky and the HUD does not call it a
/// storm. Under its clouds it does both.
#[test]
fn a_storm_needs_its_clouds() {
    let bare = Weather {
        storm: 1.0,
        ..default()
    };
    assert_eq!((local(&bare).haze, super::word(&bare)), (0.0, "wind"));
    let thin = Weather { cloud: 0.5, ..bare };
    assert_eq!((local(&thin).haze, super::word(&thin)), (0.0, "wind"));
    let dark = Weather { cloud: 0.9, ..bare };
    assert_eq!((local(&dark).haze, super::word(&dark)), (1.0, "storm"));
    let nearly = Weather {
        cloud: 0.85,
        ..bare
    };
    assert_eq!(super::word(&nearly), "storm");
    let breeze = Weather { storm: 0.2, ..bare };
    assert_eq!(super::word(&breeze), "clear");
}

/// On the way down from orbit the haze and the dust of the place come into the sky over a band
/// of air, without a step in either or in the sight of the fog, and the weather is always
/// reported inside that band. The clouds are not blended: they are the same from every height.
#[test]
fn the_sky_blends_into_the_local_weather() {
    let storm = Weather {
        cloud: 1.0,
        rain: 1.0,
        dust: 1.0,
        storm: 1.0,
        ..default()
    };
    let drawn = |step: u16| {
        let air = f32::from(step) / 2000.0;
        LocalWeather(seen(&storm, air)).look()
    };
    let values = |look: WeatherLook| [look.haze, look.dust, look.damp];
    for step in 0..2000 {
        let (here, next) = (drawn(step), drawn(step + 1));
        let (now, then) = (values(here), values(next));
        let jump = now.iter().zip(then).map(|(a, b)| (a - b).abs());
        assert!(jump.fold(0.0, f32::max) < 0.01, "at {step}");
        assert!(
            (here.sight() - next.sight()).abs() < 0.06,
            "sight at {step}"
        );
    }
    assert_eq!(seen(&storm, 0.0), None);
    assert_eq!(seen(&storm, 0.05), None);
    assert!((1..=2000u16).all(|step| step <= 100 || drawn(step) != WeatherLook::CALM));
    assert!((101..=2000u16).all(|step| seen(&storm, f32::from(step) / 2000.0).is_some()));
    assert_eq!((drawn(600), drawn(2000)), (local(&storm), local(&storm)));
}

/// Leaving the world clears the weather, what the sky was told of it and the word of the HUD.
#[test]
fn leaving_the_world_clears_the_sky_and_the_hud() {
    let mut world = World::new();
    world.insert_resource(Weather {
        rain: 1.0,
        ..default()
    });
    world.insert_resource(LocalWeather(Some(WeatherLook {
        haze: 1.0,
        ..WeatherLook::CALM
    })));
    world.insert_resource(HudWeather { word: "rain" });
    world.run_system_once(super::watch::clear).unwrap();
    assert_eq!(*world.resource::<Weather>(), Weather::default());
    assert_eq!(*world.resource::<LocalWeather>(), LocalWeather::default());
    assert_eq!(*world.resource::<HudWeather>(), HudWeather::default());
}
