//! Visibility: how much of the body is drawn in each view.

use bevy::prelude::*;

use super::animate::Bone;
use super::rig::Seg;
use super::{Avatar, Shown};

/// Returns the visibility of the whole body and of its head for a setting.
///
/// The first person view hides the head and what hangs from it, the screen, and nothing else:
/// arms, legs, boots and wings stay in view. A hidden part casts no shadow.
pub fn drawn(shown: Shown) -> (Visibility, Visibility) {
    match shown {
        Shown::Full => (Visibility::Inherited, Visibility::Inherited),
        Shown::FirstPerson => (Visibility::Inherited, Visibility::Hidden),
        Shown::Hidden => (Visibility::Hidden, Visibility::Inherited),
    }
}

/// Applies the visibility setting to the root of the body and to its head.
pub fn show(
    shown: Res<Shown>,
    mut root: Single<&mut Visibility, (With<Avatar>, Without<Bone>)>,
    mut bones: Query<(&Bone, &mut Visibility), Without<Avatar>>,
) {
    let (body, head) = drawn(*shown);
    root.set_if_neq(body);
    for (bone, mut visibility) in &mut bones {
        if bone.0 == Seg::Head {
            visibility.set_if_neq(head);
        }
    }
}
