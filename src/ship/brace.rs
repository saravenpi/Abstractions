//! The shake of rough air and of a crash on the view, as the body feels it.
//!
//! From outside the ship the camera shakes on its own. In the seat the head, the frame and the
//! canopy are one rigid body, so the whole ship and the camera turn together by the small tilt
//! of the shake: the cockpit stays still in the view and only the world outside trembles. The
//! seat keeps no offset, so the eye never leaves the cabin, and nothing shakes the image itself.

use bevy::prelude::*;
use bevy::transform::TransformSystems;

use super::body::Ship;
use super::shake::{Buffet, JOLT, shake};
use super::view::{HULL_RADIUS, seated};
use crate::player::Player;
use crate::render::GameCamera;
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

/// Returns how hard the view shakes now, scaled by the "Speed effects" setting.
fn strength(level: &Buffet, prefs: &ViewPrefs) -> f32 {
    (level.level + level.jolt * JOLT.0) * prefs.speed_fx.clamp(0.0, 1.0)
}

/// Returns true when the camera at `eye` is inside the hull that `target` follows: the seat.
fn aboard(eye: Vec3, target: &Target) -> bool {
    eye.distance(target.position) < HULL_RADIUS
}

/// Shakes the view the rig made in rough air and after a crash: an offset and a tilt on top of
/// it from outside, the tilt alone from the seat. The rig writes the whole camera pose every
/// frame, so the shake never adds up.
pub fn buffet(
    (time, level, prefs, target): (Res<Time>, Res<Buffet>, Res<ViewPrefs>, Res<Target>),
    mut eyes: Single<&mut Transform, With<GameCamera>>,
) {
    let strength = strength(&level, &prefs);
    if strength <= 0.0 {
        return;
    }
    let inside = aboard(eyes.translation, &target);
    let (offset, tilt) = felt(time.elapsed_secs_wrapped(), strength, inside);
    let turned = eyes.rotation * offset;
    eyes.translation += turned;
    eyes.rotation = (eyes.rotation * tilt).normalize();
}

/// The player body in the seat, which is neither the camera nor the hull.
type Seated = (With<Player>, Without<GameCamera>, Without<Ship>);

/// Turns the drawn hull, with the cockpit and the canopy on it, and the body in the seat with the
/// camera, about the eye, by the tilt the camera took from the seat. The hull is posed from its
/// [`Placed`] pose every frame, so nothing adds up and a calm view leaves it as the frame copy
/// wrote it.
pub fn brace(
    (time, level, prefs, target, anchor): (
        Res<Time>,
        Res<Buffet>,
        Res<ViewPrefs>,
        Res<Target>,
        Res<Anchor>,
    ),
    eye: Single<&Transform, (With<GameCamera>, Without<Ship>, Without<Player>)>,
    mut hull: Single<(&Placed, &mut Transform), (With<Ship>, Without<GameCamera>)>,
    mut body: Single<&mut Transform, Seated>,
) {
    if !aboard(eye.translation, &target) {
        return;
    }
    let (_, tilt) = felt(time.elapsed_secs_wrapped(), strength(&level, &prefs), true);
    let turn = eye.rotation * tilt * eye.rotation.inverse();
    let (placed, transform) = &mut *hull;
    let pose = Transform {
        translation: anchor.to_render(placed.pos),
        rotation: anchor.rotation_to_render(placed.rot),
        scale: transform.scale,
    };
    transform.set_if_neq(braced(pose, eye.translation, turn));
    let seat = **body;
    **body = braced(seat, eye.translation, turn);
}

/// Registers the shake: on the camera after the rig, and on the hull after the frame copy.
pub(super) fn plugin(app: &mut App) {
    app.add_systems(Update, buffet.after(ViewSet::Rig).run_if(seated))
        .add_systems(
            PostUpdate,
            brace
                .run_if(seated)
                .after(FrameSet::Sync)
                .before(TransformSystems::Propagate),
        );
}
