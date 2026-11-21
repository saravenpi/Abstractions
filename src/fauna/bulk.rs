//! The bulk of the animals near the robot: a drawn living animal within [`BULK_NEAR`] of the robot gets a
//! kinematic capsule the size of its body, on the layer of the animals, so the body of the player and
//! the ship meet it instead of walking through it. Farther animals and carcasses have none, which keeps
//! the physics to the few the player can touch.

use std::f32::consts::FRAC_PI_2;

use avian3d::prelude::*;
use bevy::prelude::*;

use super::Dead;
use super::beast::Beast;
use super::motion::Motion;
use super::tune::BULK_NEAR;
use crate::life::body::{Body, Locomotion};
use crate::player::Player;
use crate::screens::playing;
use crate::space::Anchor;
use crate::world::{ANIMALS, Terrain};

/// The capsule child of a drawn animal near the robot, kept on the root of the animal.
#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub struct Bulk(pub Entity);

/// Marks the capsule child of an animal.
#[derive(Component, Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Shape;

/// The capsules of the animals, moved with their bodies.
type Shapes<'w, 's> = Query<'w, 's, &'static mut Transform, (With<Shape>, Without<Player>)>;

/// Returns the radius and the length between its ends of the capsule of a body `length` long with hips
/// `height` high: as thick as four tenths of its hips, never wider than its length.
pub fn capsule(length: f32, height: f32) -> (f32, f32) {
    let radius = (0.4 * height).clamp(0.05, 0.5 * length.max(0.1));
    (radius, (length - 2.0 * radius).max(0.0))
}

/// Returns the physics of the body of an animal `length` long with hips `height` high: a kinematic
/// capsule along its length on the layer of the animals.
pub fn bulk(length: f32, height: f32) -> impl Bundle {
    let (radius, long) = capsule(length, height);
    (
        RigidBody::Kinematic,
        Collider::capsule(radius, long),
        CollisionLayers::new(ANIMALS, LayerMask::ALL),
    )
}

/// Returns where the capsule of `body` lies in the frame of its animal moving as `motion`: along its
/// heading, at the height of its hips over its drawn origin on the ground, or on its drawn origin for a
/// body that flies, swims or floats, whose origin is the body itself.
fn torso(motion: &Motion, body: &Body) -> Transform {
    let grounded = matches!(
        body.locomotion,
        Locomotion::Walk | Locomotion::Slither | Locomotion::Ooze | Locomotion::Tumble
    );
    let lift = [0.0, body.height.max(0.05)][usize::from(grounded)];
    Transform {
        translation: motion.root.place + motion.root.turn * Vec3::Y * lift,
        rotation: motion.root.turn * Quat::from_rotation_x(FRAC_PI_2),
        scale: Vec3::ONE,
    }
}

/// Gives the animal `entity`, drawn as `beast` with `body`, the capsule it should have when `near` the
/// robot, moves the one it has with it, or takes it away.
fn fit(
    commands: &mut Commands,
    (entity, beast, bulk): (Entity, &Beast, Option<&Bulk>),
    (body, near): (&Body, bool),
    shapes: &mut Shapes,
) {
    match (near, bulk) {
        (true, None) => {
            let place = torso(&beast.motion, body);
            let shape = (Shape, place, ChildOf(entity));
            let child = commands
                .spawn((shape, self::bulk(body.length, body.height)))
                .id();
            commands.entity(entity).insert(Bulk(child));
        }
        (true, Some(bulk)) => {
            if let Ok(mut place) = shapes.get_mut(bulk.0) {
                place.set_if_neq(torso(&beast.motion, body));
            }
        }
        (false, Some(bulk)) => {
            commands.entity(bulk.0).try_despawn();
            commands.entity(entity).remove::<Bulk>();
        }
        (false, None) => {}
    }
}

/// Gives the drawn living animals within [`BULK_NEAR`] of the robot their capsule, moves it with their
/// bodies, and takes it from those that went farther or died.
pub fn bulk_up(
    mut commands: Commands,
    (terrain, anchor): (Res<Terrain>, Res<Anchor>),
    robot: Query<&Transform, (With<Player>, Without<Shape>)>,
    beasts: Query<(Entity, &Beast, Option<&Bulk>, Has<Dead>)>,
    mut shapes: Shapes,
) {
    let (Some(life), Ok(robot)) = (terrain.globe().life(), robot.single()) else {
        return;
    };
    let robot = anchor.to_frame(robot.translation);
    for (entity, beast, bulk, dead) in &beasts {
        let Some(species) = life.species.get(usize::from(beast.index)) else {
            continue;
        };
        let here = beast.spot.planet(beast.motion.root.at);
        let near = !dead && here.distance(robot) < BULK_NEAR;
        fit(
            &mut commands,
            (entity, beast, bulk),
            (&species.body, near),
            &mut shapes,
        );
    }
}

/// Registers the bulk of the animals: in play on a living planet, after the walk moved them.
pub(super) fn plugin(app: &mut App) {
    app.add_systems(
        Update,
        bulk_up
            .after(super::walk::walk)
            .run_if(playing)
            .run_if(super::gate::living)
            .run_if(any_with_component::<Beast>),
    );
}
