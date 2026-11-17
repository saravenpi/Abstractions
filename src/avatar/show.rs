//! Visibility: how much of the body is drawn in each view.

use bevy::prelude::*;

use super::animate::Bone;
use super::rig::Seg;
use super::{Avatar, AvatarPose, Shown};

/// Pitch of the view above the horizon, in radians, past which the arms are hidden in first
/// person: swinging or raised, they sit right beside the eyes and would fill a view that looks up.
pub const ARMS_PITCH: f32 = 0.45;

/// Visibility of the parts of the body that are switched on and off.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Drawn {
    /// The root of the body.
    pub body: Visibility,
    /// The head and what hangs from it, the screen.
    pub head: Visibility,
    /// Both arms: upper arms, forearms and hands.
    pub arms: Visibility,
}

/// Returns the visibility of the parts of the body for a setting, with the view `pitch` radians
/// above the horizon.
///
/// The first person view hides the head and what hangs from it, the screen, and the arms when
/// it looks up; legs, boots and wings stay in view, and so does everything looking down. A
/// hidden part casts no shadow.
pub fn drawn(shown: Shown, pitch: f32) -> Drawn {
    let (on, off) = (Visibility::Inherited, Visibility::Hidden);
    let up = pitch > ARMS_PITCH;
    match shown {
        Shown::Full => Drawn {
            body: on,
            head: on,
            arms: on,
        },
        Shown::FirstPerson => Drawn {
            body: on,
            head: off,
            arms: if up { off } else { on },
        },
        Shown::Hidden => Drawn {
            body: off,
            head: on,
            arms: on,
        },
    }
}

/// Returns whether `seg` is part of an arm.
fn arm(seg: Seg) -> bool {
    matches!(
        seg,
        Seg::UpperArmL | Seg::UpperArmR | Seg::ForearmL | Seg::ForearmR | Seg::HandL | Seg::HandR
    )
}

/// Applies the visibility setting to the root of the body, its head and its arms.
pub fn show(
    shown: Res<Shown>,
    input: Res<AvatarPose>,
    mut root: Single<&mut Visibility, (With<Avatar>, Without<Bone>)>,
    mut bones: Query<(&Bone, &mut Visibility), Without<Avatar>>,
) {
    let parts = drawn(*shown, input.look.y);
    root.set_if_neq(parts.body);
    for (bone, mut visibility) in &mut bones {
        match bone.0 {
            Seg::Head => visibility.set_if_neq(parts.head),
            seg if arm(seg) => visibility.set_if_neq(parts.arms),
            _ => false,
        };
    }
}
