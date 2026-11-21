//! A glide flown into the ground at 60 m/s: onto a collider and onto the bare ground of a world
//! whose colliders have not loaded, it lands on the surface, never under it, and stands.

use std::sync::Arc;

use avian3d::prelude::*;
use bevy::prelude::*;

use super::stage::{app, body};
use crate::gear::{FlightState, Gear, Mode, Servo};
use crate::player::asks::gather;
use crate::player::controller::{Motion, drive};
use crate::player::flight::{STILL, WATCH, aground, touching};
use crate::player::ground::Ground;
use crate::player::{BODY_LENGTH, BODY_RADIUS, Player};
use crate::sim::PlayerId;
use crate::space::Anchor;
use crate::universe::{PlanetClass, PlanetRecord, STANDARD_GRAVITY, Water};
use crate::world::globe::Face;
use crate::world::{Chunks, Globe, Terrain};

/// A dry world, so the ground the body lands on is never under a sea.
const DESERT: PlanetRecord = PlanetRecord {
    class: PlanetClass::Desert,
    pressure: 0.4,
    water: Water::None,
    ocean: 0.0,
    ..PlanetRecord::TEMPERATE
};

/// Velocity of the glide in the upright frame: 60 m/s along a path 20 degrees down, heading -Z.
const DIVE: Vec3 = Vec3::new(0.0, -20.5, -56.4);

/// Puts the body `above` metres over render height `ground`, gliding at [`DIVE`].
fn launch(app: &mut App, ground: f32, above: f32) {
    let world = app.world_mut();
    let mut bodies =
        world.query_filtered::<(&mut Transform, &mut Motion, &mut Gear), With<Player>>();
    for (mut transform, mut motion, mut gear) in bodies.iter_mut(world) {
        transform.translation = Vec3::Y * (ground + above);
        (motion.velocity, motion.wings, motion.grounded) = (DIVE, true, false);
        let pitch = DIVE.y.atan2(-DIVE.z);
        gear.state = FlightState {
            mode: Mode::Glide,
            velocity: DIVE,
            wings: 1.0,
            prone: 1.0,
            pitch,
            servo: Servo {
                climb: pitch,
                ..default()
            },
            ..FlightState::default()
        };
    }
}

/// Flies the body of `app` for `frames` frames and returns, for each, how far its soles are
/// above `floor`, the height of the surface under them, and whether it stood on the ground.
fn flown(app: &mut App, frames: usize, floor: &dyn Fn(&World, Vec3) -> f32) -> Vec<(f32, bool)> {
    (0..frames)
        .map(|_| {
            app.update();
            let (transform, motion, gear) = body(app, PlayerId::LOCAL);
            let at = transform.translation;
            let soles = at.y - BODY_LENGTH * 0.5 - BODY_RADIUS;
            let stood = motion.grounded && !gear.state.mode.airborne();
            (soles - floor(app.world(), at), stood)
        })
        .collect()
}

/// Checks that no frame of `track` put the soles under the surface, that the body stood within
/// three frames of reaching it, and that it stands at the end.
fn landed(track: &[(f32, bool)], what: &str) {
    let lowest = track.iter().map(|(gap, _)| *gap).fold(f32::MAX, f32::min);
    assert!(
        lowest > -0.06,
        "{what}: a frame {lowest} m under the surface"
    );
    let reached = track.iter().position(|(gap, _)| *gap < 0.1);
    let reached = reached.unwrap_or_else(|| panic!("{what}: never reached the ground"));
    let stood = track[reached..].iter().position(|(_, stood)| *stood);
    assert!(
        stood.is_some_and(|after| after <= 3),
        "{what}: stood {stood:?} frames after"
    );
    assert!(
        track.last().is_some_and(|(_, stood)| *stood),
        "{what}: not on its feet at the end"
    );
}

/// Into a static slope of 15 degrees rising ahead, at 60 m/s, the body lands on the slope: no frame under it,
/// on the ground within three frames, and four seconds later on its feet with the wings folded.
#[test]
fn a_fast_glide_lands_on_a_slope() {
    let mut app = app(&[(PlayerId::LOCAL, 0.0)]);
    app.add_systems(Update, (gather, drive).chain());
    let tilt = Quat::from_rotation_x(15.0_f32.to_radians());
    let normal = tilt * Vec3::Y;
    let centre = Vec3::new(0.0, 20.0, -120.0);
    app.world_mut().spawn((
        RigidBody::Static,
        Collider::cuboid(400.0, 1.0, 400.0),
        Transform::from_translation(centre).with_rotation(tilt),
    ));
    let top = centre + normal * 0.5;
    launch(&mut app, 60.0, 0.0);
    let plane =
        move |_: &World, at: Vec3| (top - Vec3::new(at.x, 0.0, at.z)).dot(normal) / normal.y;
    let track = flown(&mut app, 150, &plane);
    landed(&track, "slope");
    let (_, motion, gear) = body(&mut app, PlayerId::LOCAL);
    assert!(
        !motion.wings && gear.state.mode == Mode::Off,
        "{motion:?} {:?}",
        gear.state.mode
    );
}

/// On a streamed world whose ground colliders have not loaded, the body flies into the ground
/// the terrain draws at 60 m/s and lands on it as on a collider: no frame under it, on the
/// ground within three frames, and standing there five seconds on.
#[test]
fn a_fast_glide_lands_on_bare_ground() {
    let mut app = app(&[(PlayerId::LOCAL, 0.0)]);
    app.add_systems(Update, (gather, drive).chain())
        .init_resource::<Chunks>();
    let globe = Arc::new(Globe::new(4, 3000.0, STANDARD_GRAVITY).with_record(DESERT));
    let terrain = Terrain::on(globe.clone(), globe.face_chart(Face::PosY));
    app.insert_resource(terrain.clone());
    let world = app.world_mut();
    let mut grounds = world.query_filtered::<Entity, (With<RigidBody>, Without<Player>)>();
    for entity in grounds.iter(world).collect::<Vec<_>>() {
        world.entity_mut(entity).insert(ColliderDisabled);
    }
    let anchor = *app.world().resource::<Anchor>();
    let chart = terrain.locate(anchor.to_frame(Vec3::ZERO));
    launch(&mut app, terrain.height(chart.x, chart.z), 40.0);
    let bare = |world: &World, at: Vec3| {
        let (anchor, terrain) = (world.resource::<Anchor>(), world.resource::<Terrain>());
        let chart = terrain.locate(anchor.to_frame(at));
        let centre = chart.y - at.y;
        terrain.mesh_height(chart.x, chart.z) - centre
    };
    let track = flown(&mut app, 150, &bare);
    landed(&track, "bare ground");
}

/// The watchdog: a body the gear carries that rests on walkable ground, slower than a skim,
/// counts as landed after [`WATCH`] frames whatever the gap under it, and a fast skim over the
/// ground, as a take-off run, never does.
#[test]
fn no_state_stays_falling_on_the_ground() {
    let near = Ground {
        normal: Vec3::Y,
        gap: 0.3,
    };
    let mut rested = 0;
    for frame in 0..10_u8 {
        assert_eq!(
            touching(Mode::Fall, Some(near), rested),
            frame >= WATCH,
            "frame {frame}"
        );
        rested = aground(rested, Mode::Fall, Some(near), 0.0);
    }
    let mut skim = 0;
    for _ in 0..30 {
        skim = aground(skim, Mode::Fly, Some(near), STILL + 20.0);
        assert!(!touching(Mode::Fly, Some(near), skim));
    }
    assert_eq!(aground(9, Mode::Fall, None, 0.0), 0);
    assert_eq!(aground(9, Mode::Off, Some(near), 0.0), 0);
}
