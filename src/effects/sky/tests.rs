//! Tests of the weather particles: what falls, nothing under a roof, and a downpour that stays
//! inside the fixed pool.

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
use crate::weather::Weather;
use crate::wind::Wind;
use crate::world::globe::Face;
use crate::world::structures::{entries, stilt_village};
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
        flowers: 0.0,
    }
}

/// Each weather particle takes its share of the rolls, nothing falls from a clear sky, under
/// a roof, under the sea or above every summit.
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
    let roof = Sky {
        sheltered: true,
        ..rain
    };
    for (sky, height) in [
        (Sky::default(), 5.0),
        (roof, 5.0),
        (rain, -3.0),
        (rain, 900.0),
    ] {
        assert!(rolls().all(|roll| fall(&sky, &meadow(height), roll).is_none()));
    }
    assert!(!Sky::default().falling() && !roof.falling() && rain.falling());
}

/// A roof is only over the plank floor of a house or a hut, or the floor of a cabin or a
/// lighthouse: not over a pier or bare ground.
#[test]
fn a_roof_is_over_roofed_floors_only() {
    assert!(roofed(Some(FloorKind::Wood), Some(Site::Village)));
    assert!(roofed(Some(FloorKind::Stone), Some(Site::Cabin)));
    assert!(!roofed(Some(FloorKind::Wood), Some(Site::Dock)));
    assert!(!roofed(None, Some(Site::Village)));
}

/// In a stilt village, whose house floors, platforms and streets are all planks, rain falls on the plaza
/// boardwalk and not inside a house.
#[test]
fn rain_falls_on_a_boardwalk_and_not_indoors() {
    let (view, site, seen) = stilt_village();
    let (village, lift) = (Some(Site::Village), 0.02);
    let rain = Weather {
        rain: 1.0,
        ..default()
    };
    let plaza = view.embed(site.origin.x, site.floor + lift, site.origin.y);
    assert_eq!(seen.floor_at(plaza), Some(FloorKind::Wood));
    let open = Sky::of(&rain, roofed(seen.room_at(plaza), village));
    assert_eq!(fall(&open, &meadow(5.0), 0.1), Some(Kind::Rain));
    let houses = entries(&view, &site);
    assert!(!houses.is_empty());
    for (middle, ..) in houses {
        let floor = seen.room_at(middle + middle.normalize() * f64::from(lift));
        let indoors = Sky::of(&rain, roofed(floor, village));
        assert_eq!(fall(&indoors, &meadow(5.0), 0.1), None);
    }
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
/// a roof the same pool carries no rain at all.
#[test]
fn a_downpour_stays_within_the_pool() {
    let rain = Sky {
        rain: 1.0,
        ..default()
    };
    let (all, streaks) = rain_after(downpour(rain));
    assert_eq!(all, POOL_SIZE);
    assert!(streaks >= POOL_SIZE / 2, "{streaks}");
    let roof = Sky {
        sheltered: true,
        ..rain
    };
    assert_eq!(rain_after(downpour(roof)), (POOL_SIZE, 0));
}
