//! Ground below sea level as it sounds on each class of planet: footsteps and open air in a dry basin,
//! a muffled silence where a liquid fills it.

use std::sync::Arc;

use bevy::prelude::*;

use super::app::{app, enter_world};
use crate::audio::cue::StepSurface;
use crate::audio::scene::AudioScene;
use crate::audio::sense::Here;
use crate::audio::steps::footing;
use crate::player::{BODY_LENGTH, BODY_RADIUS, EYE_OFFSET};
use crate::render::GameCamera;
use crate::space::Anchor;
use crate::universe::{HOME_GRAVITY, HOME_RADIUS, PlanetClass, PlanetRecord, Water};
use crate::world::globe::Chart;
use crate::world::{Globe, Terrain};

/// Returns one hand-built record per planet class with the share of liquid its class really has, and
/// whether that makes a sea.
fn classes() -> [(PlanetRecord, bool); 6] {
    let world = |class, pressure, water, ocean| PlanetRecord {
        class,
        pressure,
        water,
        ocean,
        ..PlanetRecord::TEMPERATE
    };
    [
        (world(PlanetClass::Ocean, 1.4, Water::Liquid, 0.92), true),
        (PlanetRecord::TEMPERATE, true),
        (world(PlanetClass::Desert, 0.4, Water::None, 0.0), false),
        (world(PlanetClass::Ice, 0.7, Water::Ice, 0.0), false),
        (world(PlanetClass::Lava, 2.5, Water::Steam, 0.3), true),
        (world(PlanetClass::Barren, 0.0, Water::None, 0.0), false),
    ]
}

/// Returns the chart position of a body standing on ground of `terrain` that lies 3 to 10 m
/// under sea level.
fn basin(terrain: &Terrain) -> Vec3 {
    let spots = (0..40_000).map(|k| Vec2::new((k % 200) as f32, (k / 200) as f32) * 25.0);
    let ground = spots.map(|at| Vec3::new(at.x, terrain.height(at.x, at.y), at.y));
    let low = ground.into_iter().find(|at| at.y < -3.0 && at.y > -10.0);
    low.expect("the world has ground under sea level") + Vec3::Y * (0.5 * BODY_LENGTH + BODY_RADIUS)
}

/// Stands the listener in a basin of world 7 turned into a planet with `record`, and returns
/// the scene sensed there and the footstep its ground gives.
fn heard(name: &str, record: PlanetRecord) -> (AudioScene, Option<StepSurface>) {
    let mut app = app(name);
    enter_world(&mut app);
    app.update();
    let globe = Globe::new(7, HOME_RADIUS, HOME_GRAVITY).with_record(record);
    let terrain = Terrain::on(Arc::new(globe), Chart::Flat);
    let body = basin(&terrain);
    let eyes = terrain.embed(body.x, body.y + EYE_OFFSET, body.z);
    let eyes = app.world().resource::<Anchor>().to_render(eyes);
    let mut here = app.world_mut().resource_mut::<Here>();
    here.0.as_mut().expect("a planet was sensed").1.record = record;
    let mut cameras = app
        .world_mut()
        .query_filtered::<&mut GlobalTransform, With<GameCamera>>();
    for mut camera in cameras.iter_mut(app.world_mut()) {
        *camera = GlobalTransform::from_translation(eyes);
    }
    app.insert_resource(terrain.clone());
    app.update();
    (
        *app.world().resource::<AudioScene>(),
        footing(&terrain, body),
    )
}

/// In a basin of a world without a sea the sound is open and the ground gives footsteps, as on
/// any ground. Where water or molten rock fills the basin the head is under it: the sound is
/// muffled and no step is heard.
#[test]
fn a_dry_basin_sounds_like_open_ground() {
    for (record, wet) in classes() {
        let class = record.class;
        let (scene, step) = heard(&format!("basin-{}", class.label()), record);
        assert_eq!(scene.submerged, f32::from(u8::from(wet)), "{class:?}");
        assert_eq!(step.is_none(), wet, "{class:?}: {step:?}");
        assert!(wet || scene.shore == 0.0, "{class:?}: a shore is heard");
    }
}
