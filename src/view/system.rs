//! The rig inside the game: it reads the mouse, casts against the physics world and moves the game camera.
//!
//! Everything here is off until [`Enabled`] is switched on, so the rig does not fight the
//! cameras of the player and of the ship before the game is moved over to it.

use avian3d::prelude::*;
use bevy::ecs::system::SystemParam;
use bevy::prelude::*;

use super::io::{BodyFade, CameraPose, Heading, Look, Moment, Occluder, Target};
use super::keys::{read_look, toggle};
use super::mode::ViewPrefs;
use super::rig::Rig;
use super::tune::RADIUS;
use crate::player::Player;
use crate::render::GameCamera;
use crate::screens::{in_control, playing};
use crate::settings::{Persistent, Settings};
use crate::ship::Ship;

/// Whether the rig drives the game camera. Off by default.
#[derive(Resource, Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct Enabled(pub bool);

/// A deliberate cut: a teleport, boarding, a respawn. The camera goes straight to where it
/// belongs instead of gliding there.
#[derive(Message, Debug, Default, Clone, Copy, PartialEq)]
pub struct Cut {
    /// Look angles to take, or `None` to keep the current ones.
    pub heading: Option<Heading>,
}

/// Marker for a collider the camera looks through: foliage, thin posts, anything it should not pull in for.
///
/// The body of the player and the ship are looked through without it.
#[derive(Component, Debug, Default, Clone, Copy)]
pub struct Unseen;

/// When the view systems run, in `Update`.
#[derive(SystemSet, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ViewSet {
    /// The mouse is read into [`Look`]. Change [`Look`] after this set to override it.
    Input,
    /// The rig runs and moves the camera. Write [`Target`] before this set.
    Rig,
}

/// The sphere the camera casts.
#[derive(Resource, Debug, Clone)]
struct Ball(Collider);

/// Filter of the colliders the camera looks through.
type Hidden = Or<(With<Unseen>, With<Player>, With<Ship>)>;

/// The physics world as the rig sees it.
#[derive(SystemParam)]
struct Sight<'w, 's> {
    /// Spatial queries, valid in `Update`.
    query: SpatialQuery<'w, 's>,
    /// Colliders to look through.
    hidden: Query<'w, 's, (), Hidden>,
    /// The camera sphere.
    ball: Res<'w, Ball>,
}

impl Occluder for Sight<'_, '_> {
    fn cast(&self, origin: Vec3, direction: Vec3, max: f32, radius: f32) -> Option<f32> {
        let dir = Dir3::new(direction).ok()?;
        let filter = SpatialQueryFilter::default();
        let seen = |entity: Entity| !self.hidden.contains(entity);
        if radius <= 0.0 {
            let hit = self
                .query
                .cast_ray_predicate(origin, dir, max, true, &filter, &seen);
            return hit.map(|hit| hit.distance);
        }
        let config = ShapeCastConfig::from_max_distance(max);
        let fresh = (radius != RADIUS).then(|| Collider::sphere(radius));
        let ball = fresh.as_ref().unwrap_or(&self.ball.0);
        self.query
            .cast_shape_predicate(ball, origin, Quat::IDENTITY, dir, &config, &filter, &seen)
            .map(|hit| hit.distance)
    }
}

/// Run condition: the rig drives the game camera.
pub fn enabled(on: Res<Enabled>) -> bool {
    on.0
}

/// What the rig writes for the rest of the game.
#[derive(SystemParam)]
struct Results<'w> {
    /// The camera.
    pose: ResMut<'w, CameraPose>,
    /// How much of the body to draw.
    fade: ResMut<'w, BodyFade>,
    /// The look angles.
    heading: ResMut<'w, Heading>,
}

/// What the rig reads besides the physics world.
#[derive(SystemParam)]
struct Asked<'w, 's> {
    /// Frame time.
    time: Res<'w, Time>,
    /// What the camera follows.
    target: Res<'w, Target>,
    /// What the player asks. It is taken, so the same turn is never applied twice.
    look: ResMut<'w, Look>,
    /// The field of view setting.
    settings: Res<'w, Persistent<Settings>>,
    /// The cuts asked for since the last frame.
    cuts: MessageReader<'w, 's, Cut>,
}

/// Runs the rig for the frame: applies the cuts and the zoom, steps, and publishes the results.
fn drive(
    mut asked: Asked,
    sight: Sight,
    mut prefs: ResMut<ViewPrefs>,
    mut rig: ResMut<Rig>,
    mut results: Results,
) {
    for cut in asked.cuts.read() {
        rig.cut();
        cut.heading
            .into_iter()
            .for_each(|heading| rig.face(heading));
    }
    let look = std::mem::take(&mut *asked.look);
    if look.zoom_delta != 0.0 {
        prefs.zoom_by(look.zoom_delta);
    }
    let moment = Moment {
        target: *asked.target,
        look,
        prefs: *prefs,
        fov: (asked.settings.graphics.fov as f32).to_radians(),
        dt: asked.time.delta_secs(),
    };
    let shown = rig.step(&moment, &sight);
    (*results.pose, *results.heading) = (shown.pose, shown.heading);
    results.fade.set_if_neq(shown.fade);
}

/// Moves the game camera to the pose of the rig. A camera that is the child of a body gets
/// the pose relative to that body, which must be a root entity.
fn apply(
    pose: Res<CameraPose>,
    camera: Single<(&mut Transform, &mut Projection, Option<&ChildOf>), With<GameCamera>>,
    bodies: Query<&Transform, Without<GameCamera>>,
) {
    let (mut transform, mut projection, parent) = camera.into_inner();
    let body = parent.and_then(|parent| bodies.get(parent.parent()).ok());
    let back = body.map_or(Quat::IDENTITY, |body| body.rotation.inverse());
    let origin = body.map_or(Vec3::ZERO, |body| body.translation);
    let next = Transform {
        translation: back * (pose.position - origin),
        rotation: (back * pose.rotation).normalize(),
        scale: transform.scale,
    };
    transform.set_if_neq(next);
    if let Projection::Perspective(perspective) = &mut *projection
        && perspective.fov != pose.fov
    {
        perspective.fov = pose.fov;
    }
}

/// Registers the resources of the rig and its systems, which wait for [`Enabled`].
pub(super) fn plugin(app: &mut App) {
    app.init_resource::<Enabled>()
        .init_resource::<Target>()
        .init_resource::<Look>()
        .init_resource::<ViewPrefs>()
        .init_resource::<Rig>()
        .init_resource::<CameraPose>()
        .init_resource::<BodyFade>()
        .init_resource::<Heading>()
        .insert_resource(Ball(Collider::sphere(RADIUS)))
        .add_message::<Cut>()
        .configure_sets(
            Update,
            (ViewSet::Input, ViewSet::Rig)
                .chain()
                .run_if(enabled)
                .run_if(playing),
        )
        .add_systems(
            Update,
            (
                (read_look, toggle)
                    .run_if(in_control)
                    .in_set(ViewSet::Input),
                (drive, apply).chain().in_set(ViewSet::Rig),
            ),
        );
}
