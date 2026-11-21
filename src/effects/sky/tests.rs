//! Tests of the weather particles: what falls where, and a downpour that stays inside the fixed
//! pool.

use std::sync::Arc;
use std::time::Duration;

use bevy::ecs::system::RunSystemOnce;
use bevy::prelude::*;
use bevy::time::TimeUpdateStrategy;

use super::*;
use crate::effects::eye::Eye;
use crate::effects::pool::{self, POOL_SIZE, Particle};
use crate::effects::sense::Sensed;
use crate::effects::{rng, sim};
use crate::render::DayClock;
use crate::space::Anchor;
use crate::universe::Universe;
use crate::wind::Wind;
use crate::world::globe::Face;
use crate::world::{Biome, Globe, Surface, Terrain};

/// Returns open meadow surroundings `height` metres above sea level.
fn meadow(height: f32) -> Ambient {
    Ambient {
        biome: Biome::Meadow,
        surface: Surface::Grass,
        height,
        daylight: 1.0,
        wind: 0.3,
        leaves: 0.0,
        petals: 0.0,
        pollen: 0.0,
    }
}

/// Each weather particle takes its share of the rolls, nothing falls from a clear sky, under
/// the sea or above every summit.
#[test]
fn weather_falls_only_in_the_open() {
    let rolls = || (0..100).map(|step| step as f32 / 100.0);
    let rain = Sky {
        rain: 1.0,
        ..default()
    };
    let rainy = rolls().filter(|&roll| fall(&rain, &meadow(5.0), roll) == Some(Kind::Rain));
    assert_eq!(rainy.count(), 90);
    let snow = Sky {
        snow: 0.5,
        dust: 0.5,
        ..default()
    };
    assert_eq!(fall(&snow, &meadow(5.0), 0.1), Some(Kind::Snow));
    assert_eq!(fall(&snow, &meadow(5.0), 0.6), Some(Kind::Dust));
    for (sky, height) in [(Sky::default(), 5.0), (rain, -3.0), (rain, 900.0)] {
        assert!(rolls().all(|roll| fall(&sky, &meadow(height), roll).is_none()));
    }
    assert!(!Sky::default().falling() && rain.falling());
}

/// Returns an app that moves the particle pool around a camera two metres over the ground of
/// a 3 km planet under `sky`, one frame every 16 ms.
fn downpour(sky: Sky) -> App {
    let globe = Arc::new(Globe::new(4, 3000.0, 22.0));
    let view = Terrain::on(globe.clone(), globe.face_chart(Face::PosX)).around(40.0, -25.0);
    let eye_at = Vec3::new(40.0, view.height(40.0, -25.0).max(0.0) + 2.0, -25.0);
    let anchor = Anchor::surface(Universe::new(4).home(), &view, eye_at);
    let mut app = App::new();
    app.add_plugins(MinimalPlugins)
        .insert_resource(TimeUpdateStrategy::ManualDuration(Duration::from_millis(
            16,
        )))
        .insert_resource(Eye::new(&anchor, &globe, eye_at, Quat::IDENTITY))
        .insert_resource(anchor)
        .insert_resource(view)
        .insert_resource(Sensed { sky, ..default() })
        .init_resource::<Wind>()
        .init_resource::<crate::wind::Thickness>()
        .init_resource::<DayClock>()
        .init_resource::<rng::Rng>()
        .init_resource::<Assets<Mesh>>()
        .init_resource::<Assets<StandardMaterial>>()
        .add_systems(Update, sim::step);
    let world = app.world_mut();
    world
        .run_system_once(pool::build)
        .expect("the particle looks");
    world.run_system_once(pool::spawn).expect("the particles");
    app
}

/// Returns how many particles of the pool exist, and how many are drawn rain streaks, after
/// three seconds of `app`.
fn rain_after(mut app: App) -> (usize, usize) {
    for _ in 0..190 {
        app.update();
    }
    let world = app.world_mut();
    let mut particles = world.query::<(&Particle, &Transform)>();
    let all = particles.iter(world).count();
    let rain = particles.iter(world);
    let drawn = rain.filter(|(p, t)| p.kind == Kind::Rain && t.scale != Vec3::ZERO);
    (all, drawn.count())
}

/// A downpour fills most of the fixed pool with rain streaks and never adds a particle; under
/// a clear sky the same pool carries no rain at all.
#[test]
fn a_downpour_stays_within_the_pool() {
    let rain = Sky {
        rain: 1.0,
        ..default()
    };
    let (all, streaks) = rain_after(downpour(rain));
    assert_eq!(all, POOL_SIZE);
    assert!(streaks >= POOL_SIZE / 2, "{streaks}");
    assert_eq!(rain_after(downpour(Sky::default())), (POOL_SIZE, 0));
}
