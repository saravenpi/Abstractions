//! Cocking: the pose of a javelin drawn back along the aim and the whip that lets it go.
//!
//! Drawn, the throwing hand sits behind the right shoulder, the arm almost straight, the chest
//! coiled away from the throw, the free arm pointing at the target, and the shaft lies along the
//! line from the hand to the point the crosshair picked, whatever the pitch. In first person the
//! hand sits at the lower right of the view instead, the shaft running into the crosshair. On the
//! release the hand whips forward through the aim line and the chest turns through.

use bevy::prelude::*;

use super::super::super::pose::{Core, Pose};
use super::super::super::rig::{Seg, globals};
use super::super::super::tune::SHOULDER;
use super::super::ik::reach;
use super::super::key::Key;
use super::tune::{COIL, DRAWN_BACK, EYES, POINT, THROUGH, VIEW_HOLD, WHIP};

/// Returns the turn whose -Z is the unit `dir`, its +Y as near the up as it can be: the frame of
/// a view along `dir`, its columns right, up and back.
pub fn frame(dir: Vec3) -> Quat {
    let right = dir.cross(Vec3::Y).normalize_or(Vec3::X);
    let up = right.cross(dir).normalize_or(Vec3::Y);
    Quat::from_mat3(&Mat3::from_cols(right, up, -dir))
}

/// Returns the place of the torso in the frame of the body for a chest turned and bent by
/// `trunk`, over hips at rest.
pub fn torso(trunk: [f32; 2]) -> Transform {
    let pose = Pose::default()
        .with(Core::Twist, trunk[0])
        .with(Core::Spine, trunk[1]);
    globals(&pose)[Seg::Torso as usize]
}

/// Returns where the hand holds a javelin drawn at `aim`, a point in the frame of the body, when
/// the chest is placed at `chest`: behind the shoulder, or at the lower right of the view in
/// `first` person.
fn hold(aim: Vec3, chest: &Transform, first: bool) -> Vec3 {
    let shoulder = chest.transform_point(Vec3::from_array(SHOULDER));
    if !first {
        let ahead = (aim - shoulder).with_y(0.0).normalize_or(Vec3::NEG_Z);
        return shoulder + frame(ahead) * Vec3::from_array(DRAWN_BACK);
    }
    let (neck, face) = (Vec3::from_array(EYES.0), Vec3::from_array(EYES.1));
    let look = (aim - neck - face).normalize_or(Vec3::NEG_Z);
    let eye = neck + frame(look) * face;
    let look = (aim - eye).normalize_or(look);
    eye + frame(look) * Vec3::from_array(VIEW_HOLD)
}

/// Returns the key of a javelin drawn back at `aim`, a point in the frame of the body, in the frame
/// of the coiled torso, and the unit way of its shaft in the frame of the body: from the hand to
/// the aim.
pub fn drawn(aim: Vec3, first: bool) -> (Key, Vec3) {
    let chest = torso(COIL);
    let grip = hold(aim, &chest, first);
    let way = (aim - grip).normalize_or(Vec3::NEG_Z);
    let key = Key {
        grip: chest.compute_affine().inverse().transform_point3(grip),
        axis: (chest.rotation.inverse() * way).normalize(),
        twist: COIL[0],
        spine: COIL[1],
    };
    (key, way)
}

/// Returns the key of the hand at the end of the whip of a javelin thrown along the unit `way` of the
/// frame of the body, in the frame of the torso turned through: the arm straight out along the throw.
pub fn whipped(way: Vec3) -> Key {
    let chest = torso(THROUGH);
    let axis = (chest.rotation.inverse() * way).normalize();
    Key {
        grip: Vec3::from_array(SHOULDER) + axis * WHIP,
        axis,
        twist: THROUGH[0],
        spine: THROUGH[1],
    }
}

/// Returns the shoulder, lift and elbow of the free left arm pointing along `axis`, a unit way in
/// the frame of the torso: toward the target, for balance.
pub fn pointing(axis: Vec3) -> [f32; 3] {
    let shoulder = Vec3::new(-SHOULDER[0], SHOULDER[1], SHOULDER[2]);
    reach(shoulder + axis * POINT, 0)
}
