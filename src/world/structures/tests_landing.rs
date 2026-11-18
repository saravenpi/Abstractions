//! Tests of landing on a roof: a body dropped on a gable comes to rest on it, and a player on a roof is still
//! in reach of the village.

use std::time::Duration;

use avian3d::prelude::*;
use bevy::prelude::*;
use bevy::time::TimeUpdateStrategy;

use super::brush::Mat;
use super::globes::{faces, villages};
use super::site::{CELL, Kind};
use super::tests::built;
use super::{Terrain, compound, entries, survey};

/// Radius of the ball dropped on the roof, the radius of the player.
const BALL: f32 = 0.35;

/// Returns a headless physics app at 30 frames a second that holds `roof` as a static body.
fn world(roof: Collider) -> App {
    let frame = TimeUpdateStrategy::ManualDuration(Duration::from_secs_f64(1.0 / 30.0));
    let mut app = App::new();
    app.add_plugins((MinimalPlugins, AssetPlugin::default(), TransformPlugin))
        .add_plugins(PhysicsPlugins::default())
        .init_asset::<Mesh>()
        .insert_resource(frame);
    app.finish();
    app.cleanup();
    let body = (RigidBody::Static, roof, Friction::new(1.0));
    app.world_mut().spawn((body, Transform::default()));
    app
}

/// Spawns a ball the size of the player at `at`, free to fall but not to roll, and returns it.
fn drop_ball(app: &mut App, at: Vec3) -> Entity {
    let ball = (
        RigidBody::Dynamic,
        Collider::sphere(BALL),
        Friction::new(1.0),
    );
    let still = (LockedAxes::ROTATION_LOCKED, Transform::from_translation(at));
    app.world_mut().spawn((ball, still)).id()
}

/// A ball dropped onto the slope of a gabled roof in a headless physics app comes to rest on its drawn
/// surface instead of falling through.
#[test]
fn bodies_rest_on_a_gable() {
    let t = Terrain::new(1);
    let mut b = built(&t, Kind::Village, 3);
    let (lo, hi, rise) = (Vec3::new(-2.0, 2.0, -1.5), Vec3::new(2.0, 2.0, 1.5), 0.9);
    b.gable(lo, hi, rise, Mat::deco([0.5; 3]));
    let on = b.pt(Vec3::new(0.0, lo.y + rise * 0.5, hi.z - 0.75));
    let lean = (rise / 1.5).atan().cos();
    let mut app = world(compound(&b.parts).expect("the gable is solid"));
    let id = drop_ball(&mut app, on + Vec3::Y * 1.5);
    for _ in 0..150 {
        app.update();
    }
    let at = app.world().get::<Transform>(id).expect("the ball exists");
    let speed = app
        .world()
        .get::<LinearVelocity>(id)
        .expect("the ball moves");
    let off = at.translation - on - Vec3::Y * (BALL / lean);
    let still = speed.0.length() < 0.05;
    assert!(
        off.length() < 0.05 && still,
        "the ball rests {off} from the drawn roof"
    );
}

/// A player standing on a house roof is still in reach of the village: the survey around them finds it.
#[test]
fn roofs_stay_in_their_village() {
    let face = faces(42, 3000.0).swap_remove(0);
    let site = villages(&face, 1).pop().expect("a village on the face");
    let view = face.around(site.origin.x, site.origin.y);
    let at = |y: f32| view.embed(site.origin.x, y, site.origin.y);
    let up = (at(1.0) - at(0.0)).normalize();
    let houses = entries(&view, &site);
    assert!(!houses.is_empty());
    for (middle, _, _) in houses {
        let roof = middle + up * 4.0;
        let seen = survey(&face, roof, 120.0);
        let near = seen.nearest(roof, 100.0);
        let near = near.expect("the village is in reach from its roofs");
        assert!(near.kind == Kind::Village && near.distance < CELL);
    }
}
