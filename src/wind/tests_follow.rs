//! Tests of the wind of the weather: calm weather keeps the home wind, a storm gusts, and the
//! prevailing wind turns the direction toward it.

use std::sync::Arc;

use super::follow::{to_render, weathered};
use super::*;
use crate::weather::Weather;
use crate::world::globe::Face;
use crate::world::{Globe, Terrain};

/// Under calm weather on Kodya the wind is today's wind: with no prevailing wind bit for bit,
/// and with one only its direction turns, its strength and its breeze stay.
#[test]
fn kodya_calm_wind_is_unchanged() {
    for step in 0..500 {
        let clock = wind_at(step as f32 * 7.3);
        assert_eq!(carried(weathered(clock, Vec2::ZERO, 0.0), 1.0), clock);
        let leaning = carried(weathered(clock, Vec2::new(0.0, 6.0), 0.0), 1.0);
        assert_eq!((leaning.strength, leaning.air), (clock.strength, clock.air));
        assert!((leaning.direction.length() - 1.0).abs() < 1e-4);
        assert!(leaning.direction.y >= clock.direction.y - 1e-6);
    }
    let mut app = App::new();
    app.add_plugins(MinimalPlugins).add_plugins(plugin);
    app.init_resource::<Weather>();
    app.update();
    app.update();
    let seconds = app.world().resource::<Time>().elapsed_secs();
    assert_eq!(*app.world().resource::<Wind>(), wind_at(seconds));
}

/// A storm gusts the wind up to full strength, never down, and the air still holds it: without
/// air even a storm is still.
#[test]
fn a_storm_gusts_as_the_air_carries_it() {
    for step in 0..500 {
        let clock = wind_at(step as f32 * 7.3);
        let half = weathered(clock, Vec2::ZERO, 0.5);
        assert!(half.strength >= clock.strength && half.strength <= 1.0);
        assert_eq!(weathered(clock, Vec2::ZERO, 1.0).strength, 1.0);
        assert_eq!(
            carried(weathered(clock, Vec2::ZERO, 1.0), 0.0).velocity(),
            Vec3::ZERO
        );
    }
}

/// The compass wind comes out on the render plane of a planet anchor with its speed, and is
/// zero on the flat world.
#[test]
fn the_prevailing_wind_lies_on_the_render_plane() {
    let globe = Arc::new(Globe::new(4, 3000.0, 22.0));
    let view = Terrain::on(globe.clone(), globe.face_chart(Face::PosX)).around(40.0, -25.0);
    let anchor = Anchor::surface(Universe::new(4).home(), &view, Vec3::new(40.0, 0.0, -25.0));
    let blown = to_render(&anchor, Vec2::new(3.0, 4.0));
    assert!((blown.length() - 5.0).abs() < 0.1, "{blown}");
    let flat = Anchor {
        flat: true,
        ..anchor
    };
    assert_eq!(to_render(&flat, Vec2::new(3.0, 4.0)), Vec2::ZERO);
}
