//! HUD tests: the climate line says the temperature and the weather on a planet and hides in
//! space and above the air.

use std::sync::Arc;

use bevy::ecs::system::RunSystemOnce;
use bevy::math::{DQuat, DVec3};
use bevy::prelude::*;

use super::climate::{CLEAR, ClimateGauge, HudWeather, reading, update};
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
    app.world_mut()
        .spawn((GameCamera, GlobalTransform::IDENTITY));
    let hidden = Node {
        display: Display::None,
        ..default()
    };
    app.world_mut()
        .spawn((ClimateGauge, hidden, Text::default()));
    app
}

/// Runs the line once and returns what it shows, or nothing while it is hidden.
fn read(app: &mut App) -> Option<String> {
    app.world_mut().run_system_once(update).unwrap();
    let world = app.world_mut();
    let mut gauges = world.query_filtered::<(&Node, &Text), With<ClimateGauge>>();
    let (node, text) = gauges.single(world).unwrap();
    (node.display == Display::Flex).then(|| text.0.clone())
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
    assert!(reading(&spec, &globe, low, CLEAR).is_some());
    let high = DVec3::Y * (spec.radius + spec.atmosphere_height + 3000.0);
    assert_eq!(reading(&spec, &globe, high, CLEAR), None);
}
