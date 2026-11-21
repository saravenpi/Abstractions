//! HUD tests: the climate line says the temperature and the weather on a planet and hides in
//! space and above the air.

use std::sync::Arc;

use bevy::ecs::system::RunSystemOnce;
use bevy::math::{DQuat, DVec3};
use bevy::prelude::*;

use super::climate::{
    CLEAR, ClimateGauge, Degrees, HudWeather, Sky, WORDS, reading, sky_icon, update,
};
use super::eye::{HudPose, capture};
use crate::render::GameCamera;
use crate::space::{Anchor, Clock, Frame};
use crate::universe::Universe;
use crate::world::{Globe, Planet};

/// Returns an app on the home planet with the camera `height` metres above its sea level, and
/// the climate line.
fn standing(height: f64) -> App {
    let universe = Universe::new(7);
    let spec = universe.planet(universe.home());
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.insert_resource(Anchor {
        frame: Frame::Surface(spec.id),
        origin: DVec3::X * (spec.radius + height),
        rotation: DQuat::IDENTITY,
        flat: false,
    });
    app.insert_resource(Planet {
        body: spec.id,
        globe: Arc::new(Globe::of(&spec)),
        day_seconds: spec.day_seconds,
    });
    app.insert_resource(universe)
        .insert_resource(Clock::default());
    app.world_mut().spawn((GameCamera, Transform::IDENTITY));
    app.init_resource::<HudPose>();
    app.world_mut().spawn(super::climate::line());
    app
}

/// Runs the line once and returns what it shows, or nothing while it is hidden.
fn read(app: &mut App) -> Option<String> {
    app.world_mut().run_system_once(capture).unwrap();
    app.world_mut().run_system_once(update).unwrap();
    let world = app.world_mut();
    let mut lines = world.query_filtered::<&Node, With<ClimateGauge>>();
    let shown = lines.single(world).unwrap().display == Display::Flex;
    let mut says = |word: bool| {
        let mut degrees = world.query_filtered::<&Text, With<Degrees>>();
        let mut skies = world.query_filtered::<&Text, With<Sky>>();
        match word {
            true => skies.single(world).unwrap().0.clone(),
            false => degrees.single(world).unwrap().0.clone(),
        }
    };
    shown.then(|| format!("{} - {}", says(false), says(true)))
}

/// On the ground the line gives the temperature and the weather, clear until the weather says
/// otherwise; it hides when the world is anchored in space.
#[test]
fn the_climate_line_shows_on_a_planet_and_hides_in_space() {
    let mut app = standing(2.0);
    let shown = read(&mut app).expect("shown on the ground");
    assert!(shown.ends_with(" C - clear"), "{shown}");
    app.insert_resource(HudWeather { word: "rain" });
    assert!(read(&mut app).unwrap().ends_with(" C - rain"));
    let star = app.world().resource::<Planet>().body.star;
    app.world_mut().resource_mut::<Anchor>().frame = Frame::System(star);
    assert_eq!(read(&mut app), None);
    assert_eq!(HudWeather::default().word, CLEAR);
}

/// Above the air of the planet the line hides, even while the world still streams it.
#[test]
fn the_climate_line_hides_above_the_air() {
    let mut app = standing(500_000.0);
    assert_eq!(read(&mut app), None);
    let universe = Universe::new(7);
    let spec = universe.planet(universe.home());
    let globe = Globe::of(&spec);
    let low = DVec3::Y * (spec.radius + 10.0);
    assert!(reading(&spec, &globe, low).is_some());
    let high = DVec3::Y * (spec.radius + spec.atmosphere_height + 3000.0);
    assert_eq!(reading(&spec, &globe, high), None);
}

/// Every word the weather feed writes has an icon of its own, a clear sky has a sun by day and a
/// moon by night, and a word the feed does not write has none.
#[test]
fn every_weather_word_has_an_icon() {
    let icons: Vec<_> = WORDS.iter().map(|word| sky_icon(word, true)).collect();
    assert!(icons.iter().all(Option::is_some), "{icons:?}");
    let mut seen = icons.clone();
    seen.sort_by_key(|icon| icon.map(|icon| icon as usize));
    seen.dedup();
    assert_eq!(seen.len(), WORDS.len());
    assert_ne!(sky_icon("clear", true), sky_icon("clear", false));
    assert_eq!(sky_icon("hail", true), None);
}
