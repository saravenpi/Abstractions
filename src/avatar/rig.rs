//! The rig: fifteen rigid segments in a tree, and where a pose puts each of them.
//!
//! Limbs hang along -Y of their own frame. A child segment sits at a fixed point of its parent,
//! the centre of a joint cap, and only turns there, so no gap can open at any pose.

use bevy::prelude::*;

use super::pose::{Core, Limb, Pose};
use super::tune::{FOREARM, HIP, NECK, PELVIS, SHIN, SHOULDER, THIGH, UPPER_ARM, WAIST};

/// A rigid segment of the body. Left and right follow each other, left first.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Seg {
    /// The hips, root of the tree.
    Pelvis,
    /// The chest.
    Torso,
    /// The head with its screen.
    Head,
    /// Left upper arm.
    UpperArmL,
    /// Right upper arm.
    UpperArmR,
    /// Left forearm.
    ForearmL,
    /// Right forearm.
    ForearmR,
    /// Left hand.
    HandL,
    /// Right hand.
    HandR,
    /// Left thigh.
    ThighL,
    /// Right thigh.
    ThighR,
    /// Left shin.
    ShinL,
    /// Right shin.
    ShinR,
    /// Left boot.
    BootL,
    /// Right boot.
    BootR,
}

/// Number of segments.
pub const SEGS: usize = 15;

impl Seg {
    /// Every segment, parents before children.
    pub const ALL: [Seg; SEGS] = [
        Seg::Pelvis,
        Seg::Torso,
        Seg::Head,
        Seg::UpperArmL,
        Seg::UpperArmR,
        Seg::ForearmL,
        Seg::ForearmR,
        Seg::HandL,
        Seg::HandR,
        Seg::ThighL,
        Seg::ThighR,
        Seg::ShinL,
        Seg::ShinR,
        Seg::BootL,
        Seg::BootR,
    ];

    /// Returns the segment this one hangs from, or nothing for the pelvis.
    pub fn parent(self) -> Option<Seg> {
        let index = self as usize;
        match self {
            Seg::Pelvis => None,
            Seg::Torso => Some(Seg::Pelvis),
            Seg::Head | Seg::UpperArmL | Seg::UpperArmR => Some(Seg::Torso),
            Seg::ThighL | Seg::ThighR => Some(Seg::Pelvis),
            _ => Some(Seg::ALL[index - 2]),
        }
    }
}

/// Returns the transform of a segment at the point `at` of its parent, turned by `turn`.
fn joint(at: [f32; 3], turn: Quat) -> Transform {
    Transform::from_translation(Vec3::from_array(at)).with_rotation(turn)
}

/// Returns the transforms of the three trunk segments, each in the frame of its parent.
fn trunk(pose: &Pose) -> [Transform; 3] {
    let lean = pose.lean
        * Quat::from_rotation_z(-pose.core(Core::Roll))
        * Quat::from_rotation_x(-pose.core(Core::Pitch));
    let chest = Quat::from_rotation_y(pose.core(Core::Twist))
        * Quat::from_rotation_x(-pose.core(Core::Spine));
    let head = Quat::from_rotation_y(pose.core(Core::HeadYaw))
        * Quat::from_rotation_x(pose.core(Core::HeadPitch));
    let shift = [pose.core(Core::Sway), pose.core(Core::Lift), 0.0];
    let mut pelvis = joint(PELVIS, lean);
    pelvis.translation += Vec3::from_array(shift);
    [pelvis, joint(WAIST, chest), joint(NECK, head)]
}

/// Returns the transforms of the six limb segments of `side`, each in the frame of its parent:
/// upper arm, forearm, hand, thigh, shin, boot.
fn limbs(pose: &Pose, side: usize) -> [Transform; 6] {
    let sign = [-1.0, 1.0][side];
    let out = |joint: Limb| Quat::from_rotation_z(sign * pose.limb(joint, side));
    let bend = |joint: Limb, way: f32| Quat::from_rotation_x(way * pose.limb(joint, side));
    let mirror = |at: [f32; 3]| [at[0] * sign, at[1], at[2]];
    [
        joint(
            mirror(SHOULDER),
            out(Limb::ArmOut) * bend(Limb::Shoulder, 1.0),
        ),
        joint([0.0, -UPPER_ARM, 0.0], bend(Limb::Elbow, 1.0)),
        joint([0.0, -FOREARM, 0.0], Quat::IDENTITY),
        joint(mirror(HIP), out(Limb::LegOut) * bend(Limb::Hip, 1.0)),
        joint([0.0, -THIGH, 0.0], bend(Limb::Knee, -1.0)),
        joint([0.0, -SHIN, 0.0], bend(Limb::Ankle, -1.0)),
    ]
}

/// Returns the transform of every segment in the frame of its parent, in the order of [`Seg::ALL`].
pub fn locals(pose: &Pose) -> [Transform; SEGS] {
    let mut all = [Transform::IDENTITY; SEGS];
    all[..3].copy_from_slice(&trunk(pose));
    for side in 0..2 {
        for (part, local) in limbs(pose, side).into_iter().enumerate() {
            all[3 + part * 2 + side] = local;
        }
    }
    all
}

/// Returns the transform of every segment in the frame of the player body, in the order of [`Seg::ALL`].
pub fn globals(pose: &Pose) -> [Transform; SEGS] {
    let mut all = locals(pose);
    for seg in Seg::ALL {
        if let Some(parent) = seg.parent() {
            all[seg as usize] = all[parent as usize] * all[seg as usize];
        }
    }
    all
}
