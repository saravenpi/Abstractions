//! A headless run: the body is spawned under a player entity, stepped through every mode, shown,
//! hidden, carried along and despawned, with no window.

use std::time::Duration;

use bevy::app::Propagate;
use bevy::camera::visibility::RenderLayers;
use bevy::prelude::*;
use bevy::time::TimeUpdateStrategy;

use super::{FRAME, scene};
use crate::avatar::animate::Bone;
use crate::avatar::rig::{SEGS, Seg};
use crate::avatar::spawn::{Lamp, Plume};
use crate::avatar::wing::Panel;
use crate::avatar::{Avatar, AvatarPose, AvatarSet, Mode, Shown};
use crate::player::{Motion, Player};

/// The state an integration system writes over what the filler found.
#[derive(Resource, Debug, Clone, Copy)]
pub struct Forced(pub AvatarPose);

/// Marks a body the forced state leaves alone, so a test can drive it by itself.
#[derive(Component)]
pub struct Free;

/// Overwrites the pose input, the way the integration is meant to: after the fill, before the animation.
fn force(forced: Res<Forced>, mut poses: Query<&mut AvatarPose, Without<Free>>) {
    for mut pose in &mut poses {
        *pose = AvatarPose {
            stride_distance: pose.stride_distance + forced.0.speed * FRAME,
            ..forced.0
        };
    }
}

/// Returns an app with the avatar systems, a fixed frame time and a player standing somewhere tilted.
pub fn world() -> (App, Entity) {
    let mut app = App::new();
    app.add_plugins((MinimalPlugins, TransformPlugin))
        .init_resource::<Assets<Mesh>>()
        .init_resource::<Assets<StandardMaterial>>()
        .init_resource::<Assets<Image>>()
        .insert_resource(TimeUpdateStrategy::ManualDuration(Duration::from_secs_f32(
            FRAME,
        )))
        .insert_resource(Forced(scene(Mode::Idle, 0.0)));
    crate::avatar::systems(&mut app);
    app.add_systems(
        Update,
        force.after(AvatarSet::Fill).before(AvatarSet::Animate),
    );
    let stand = Transform::from_xyz(40.0, 12.0, -7.0).with_rotation(Quat::from_euler(
        EulerRot::YXZ,
        1.1,
        0.2,
        0.1,
    ));
    let player = app
        .world_mut()
        .spawn((Player, Motion::default(), stand, Visibility::default()))
        .id();
    app.update();
    (app, player)
}

/// Returns the number of entities that carry the component `C`.
pub fn count<C: Component>(app: &mut App) -> usize {
    app.world_mut().query::<&C>().iter(app.world()).count()
}

/// Asserts that every segment is finite and sits at its joint on its parent, in world space.
fn assert_whole(app: &mut App, what: &str) {
    let mut bones = app
        .world_mut()
        .query::<(&Bone, &Transform, &GlobalTransform, &ChildOf)>();
    let world = app.world();
    let mut seen = 0;
    for (bone, local, global, parent) in bones.iter(world) {
        assert!(local.is_finite(), "{what}: {:?} is {local:?}", bone.0);
        let above = world
            .get::<GlobalTransform>(parent.parent())
            .expect("a parent with a place");
        let joint = above.transform_point(local.translation);
        assert!(
            joint.distance(global.translation()) < 1e-3,
            "{what}: {:?} left its joint",
            bone.0
        );
        assert_eq!(
            world.get::<Bone>(parent.parent()).map(|bone| bone.0),
            bone.0.parent()
        );
        seen += 1;
    }
    assert_eq!(seen, SEGS, "{what}");
}

/// Returns the segments and other parts of the body that the game camera does not see: switched
/// off themselves, or a segment put on the water mirror layer alone.
fn hidden(app: &mut App) -> (Vec<Seg>, usize, bool) {
    let mut bones = app
        .world_mut()
        .query::<(&Bone, &Visibility, Option<&Propagate<RenderLayers>>)>();
    let off = |v: &Visibility| *v == Visibility::Hidden;
    let seen = RenderLayers::layer(0);
    let unseen =
        |layers: Option<&Propagate<RenderLayers>>| layers.is_some_and(|l| !l.0.intersects(&seen));
    let segs = bones
        .iter(app.world())
        .filter(|(_, v, layers)| off(v) || unseen(*layers))
        .map(|(bone, ..)| bone.0)
        .collect();
    let mut rest = app
        .world_mut()
        .query_filtered::<&Visibility, (Without<Bone>, Without<Avatar>, Without<Player>)>();
    let others = rest.iter(app.world()).filter(|v| off(v)).count();
    let mut root = app
        .world_mut()
        .query_filtered::<&Visibility, With<Avatar>>();
    (
        segs,
        others,
        off(root.single(app.world()).expect("one body")),
    )
}

/// One body is spawned under the player and steps through every mode, and from flight back to
/// the ground, without a panic or a loose joint.
#[test]
fn body_steps_through_every_mode() {
    let (mut app, _) = world();
    let back = [Mode::Fly, Mode::Fall, Mode::Glide, Mode::Walk];
    for (mode, k) in Mode::ALL
        .into_iter()
        .chain(back)
        .flat_map(|mode| [(mode, 0.2), (mode, 1.0)])
    {
        app.insert_resource(Forced(scene(mode, k)));
        (0..12).for_each(|_| app.update());
        assert_whole(&mut app, &format!("{mode:?} at {k}"));
    }
    let mut poses = app.world_mut().query::<&AvatarPose>();
    let pose = poses.single(app.world()).expect("one body");
    assert_eq!(pose.mode, Mode::Walk);
    let parts = (
        count::<Avatar>(&mut app),
        count::<Bone>(&mut app),
        count::<Panel>(&mut app),
    );
    assert_eq!(parts, (1, SEGS, 3));
    assert_eq!((count::<Plume>(&mut app), count::<Lamp>(&mut app)), (2, 1));
}

/// The body follows the player wherever it is put, as a rebase does, and leaves with it.
#[test]
fn body_follows_the_player_and_leaves_with_it() {
    let (mut app, player) = world();
    let far = Transform::from_xyz(-9000.0, 300.0, 4000.0).with_rotation(Quat::from_rotation_y(2.0));
    *app.world_mut()
        .get_mut::<Transform>(player)
        .expect("a player") = far;
    app.update();
    let mut roots = app
        .world_mut()
        .query_filtered::<(&GlobalTransform, &ChildOf), With<Avatar>>();
    let (root, parent) = roots.single(app.world()).expect("one body");
    assert_eq!(parent.parent(), player);
    assert!(
        root.translation().distance(far.translation) < 1e-2,
        "left behind at {}",
        root.translation()
    );
    assert!(root.rotation().angle_between(far.rotation) < 1e-3);
    app.world_mut().entity_mut(player).despawn();
    app.update();
    assert_eq!(
        (
            count::<Avatar>(&mut app),
            count::<Bone>(&mut app),
            count::<Panel>(&mut app)
        ),
        (0, 0, 0)
    );
}

/// The first person view hides the head from the game camera and nothing else, the full view
/// hides nothing, and the hidden view switches the whole body off at its root. The first person
/// head is on the water mirror layer alone, so it still shows in calm water.
#[test]
fn first_person_hides_exactly_the_head() {
    let (mut app, _) = world();
    app.insert_resource(Forced(scene(Mode::Glide, 1.0)));
    (0..30).for_each(|_| app.update());
    assert_eq!(*app.world().resource::<Shown>(), Shown::FirstPerson);
    assert_eq!(hidden(&mut app), (vec![Seg::Head], 0, false));
    let mut heads = app
        .world_mut()
        .query::<(&Bone, &Visibility, Has<Propagate<RenderLayers>>)>();
    let head = heads
        .iter(app.world())
        .find(|(bone, ..)| bone.0 == Seg::Head)
        .map(|(_, shown, mirrored)| (*shown, mirrored));
    assert_eq!(head, Some((Visibility::Inherited, true)));
    app.insert_resource(Shown::Full);
    app.update();
    assert_eq!(hidden(&mut app), (vec![], 0, false));
    app.insert_resource(Shown::Hidden);
    app.update();
    assert_eq!(hidden(&mut app), (vec![], 0, true));
    app.insert_resource(Forced(scene(Mode::Idle, 0.0)));
    app.insert_resource(Shown::FirstPerson);
    (0..40).for_each(|_| app.update());
    assert_eq!(hidden(&mut app), (vec![Seg::Head], 5, false));
}
