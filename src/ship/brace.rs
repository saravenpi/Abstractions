//! The shake of rough air and of a crash on the view, as the body feels it.
//!
//! From outside the ship the camera shakes on its own. In the seat the head, the frame, the
//! displays and the canopy are one rigid body, so the whole ship and the camera turn together
//! by the small tilt of the shake about the eye: the cockpit stays still in the view and only
//! the world outside trembles. The seat keeps no offset, so the eye never leaves the cabin, and
//! nothing shakes the image itself. The eye is taken in render space even when the camera rides
//! the body of the pilot.

#[cfg(test)]
#[path = "brace_tests.rs"]
mod tests;

use bevy::prelude::*;
use bevy::transform::TransformSystems;

use super::body::Ship;
use super::shake::{self as shaking, Buffet, JOLT, Steady, shake};
use super::view::{HULL_RADIUS, seated};
use crate::player::Player;
use crate::render::GameCamera;
use crate::screens::in_control;
use crate::space::{Anchor, FrameSet, Placed};
use crate::view::{Target, ViewPrefs, ViewSet};

/// Returns the offset in metres and the tilt of the view at `time` seconds for a shake of
/// `strength`: both from outside, the tilt alone from the seat when `inside`.
pub fn felt(time: f32, strength: f32, inside: bool) -> (Vec3, Quat) {
    let (offset, tilt) = shake(time, strength);
    match inside {
        true => (Vec3::ZERO, tilt),
        false => (offset, tilt),
    }
}

/// Returns `pose` turned by `turn` about the point `eye`, as a rigid body that holds the eye.
pub fn braced(pose: Transform, eye: Vec3, turn: Quat) -> Transform {
    Transform {
        translation: eye + turn * (pose.translation - eye),
        rotation: (turn * pose.rotation).normalize(),
        scale: pose.scale,
    }
}

/// Returns how hard the view shakes now, scaled by the "Speed effects" setting and by how much
/// of the shake the view takes, none under a menu.
pub fn strength(level: &Buffet, prefs: &ViewPrefs, steady: &Steady) -> f32 {
    (level.level + level.jolt * JOLT.0) * prefs.speed_fx.clamp(0.0, 1.0) * steady.0.clamp(0.0, 1.0)
}

/// Returns true when the camera at `eye` is inside the hull that `target` follows: the seat.
fn aboard(eye: Vec3, target: &Target) -> bool {
    eye.distance(target.position) < HULL_RADIUS
}

/// Returns where the camera posed `local` under `parent` is in render space: the rig poses a
/// camera that rides a body in the axes of that body.
pub fn eye_world(local: &Transform, parent: Option<&Transform>) -> Transform {
    parent.map_or(*local, |parent| parent.mul_transform(*local))
}

/// The game camera, which is neither the hull nor the body, with the body it may ride.
type Eye = (With<GameCamera>, Without<Ship>, Without<Player>);

/// The player body in the seat, which is neither the camera nor the hull.
type Seated = (With<Player>, Without<GameCamera>, Without<Ship>);

/// Shakes the view the rig made in rough air and after a crash, from outside: an offset and a
/// tilt on top of it. In the seat the camera is left alone: [`brace`] turns it with the hull.
/// The rig writes the whole camera pose every frame, so the shake never adds up.
pub fn buffet(
    (time, level, prefs, target): (Res<Time>, Res<Buffet>, Res<ViewPrefs>, Res<Target>),
    steady: Res<Steady>,
    mut eyes: Single<(&mut Transform, Option<&ChildOf>), Eye>,
    bodies: Query<&Transform, Without<GameCamera>>,
) {
    let strength = strength(&level, &prefs, &steady);
    let (eye, parent) = &mut *eyes;
    let riding = parent.and_then(|parent| bodies.get(parent.parent()).ok());
    if strength <= 0.0 || aboard(eye_world(eye, riding).translation, &target) {
        return;
    }
    let (offset, tilt) = felt(time.elapsed_secs_wrapped(), strength, false);
    let turned = eye.rotation * offset;
    eye.translation += turned;
    eye.rotation = (eye.rotation * tilt).normalize();
}

/// The clock, the buffet, the view, the target, the anchor and the share of the shake the view
/// takes, which the brace reads.
type Bracing<'w> = (
    Res<'w, Time>,
    Res<'w, Buffet>,
    Res<'w, ViewPrefs>,
    Res<'w, Target>,
    Res<'w, Anchor>,
    Res<'w, Steady>,
);

/// The hull, which is not the camera.
type Hull = (With<Ship>, Without<GameCamera>);

/// Turns the drawn hull, with the cockpit, its displays and the canopy on it, the body in the
/// seat and the camera together, as one rigid body, about the eye by the tilt of the shake:
/// the cockpit holds still in the view and the world outside trembles. A camera that rides the
/// body turns with it. The hull is posed from its [`Placed`] pose every frame, so nothing adds
/// up and a calm view leaves it as the frame copy wrote it.
pub fn brace(
    (time, level, prefs, target, anchor, steady): Bracing,
    mut eye: Single<(&mut Transform, Option<&ChildOf>), Eye>,
    mut hull: Single<(&Placed, &mut Transform), Hull>,
    mut body: Single<(Entity, &mut Transform), Seated>,
) {
    let (camera, parent) = &mut *eye;
    let (seat, sitting) = &mut *body;
    let rides = parent.map(|parent| parent.parent());
    if rides.is_some_and(|ridden| ridden != *seat) {
        return;
    }
    let world = eye_world(camera, rides.map(|_| &**sitting));
    if !aboard(world.translation, &target) {
        return;
    }
    let (_, tilt) = felt(
        time.elapsed_secs_wrapped(),
        strength(&level, &prefs, &steady),
        true,
    );
    let turn = world.rotation * tilt * world.rotation.inverse();
    let (placed, transform) = &mut *hull;
    let pose = Transform {
        translation: anchor.to_render(placed.pos),
        rotation: anchor.rotation_to_render(placed.rot),
        scale: transform.scale,
    };
    transform.set_if_neq(braced(pose, world.translation, turn));
    **sitting = braced(**sitting, world.translation, turn);
    if rides.is_none() {
        **camera = braced(**camera, world.translation, turn);
    }
}

/// Registers the shake: how much of it the view takes, on the camera after the rig, and on the
/// hull after the frame copy.
pub(super) fn plugin(app: &mut App) {
    let steady = (
        shaking::hush.run_if(not(in_control)),
        shaking::wake.run_if(in_control),
    );
    app.init_resource::<Steady>()
        .add_systems(Update, steady.before(buffet))
        .add_systems(Update, buffet.after(ViewSet::Rig).run_if(seated))
        .add_systems(
            PostUpdate,
            brace
                .run_if(seated)
                .after(FrameSet::Sync)
                .before(TransformSystems::Propagate),
        );
}
