//! Visibility: how much of the body is drawn in each view.

use bevy::app::Propagate;
use bevy::camera::visibility::RenderLayers;
use bevy::prelude::*;

use super::animate::{Bone, Skeleton};
use super::own::Own;
use super::rig::Seg;
use super::strike::{Act, Strike};
use super::{Avatar, AvatarPose};
use crate::render::mirror_only;

/// How much of the body is drawn. Hidden parts cast no shadow either.
#[derive(Resource, Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum Shown {
    /// The whole body: third person.
    Full,
    /// Everything but the head, which would clip a camera at the eyes: first person.
    #[default]
    FirstPerson,
    /// Nothing.
    Hidden,
}

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

/// A bone of the body: its segment, its visibility and whether only the water mirror draws it.
type BoneParts = (
    &'static Bone,
    &'static mut Visibility,
    Option<&'static Propagate<RenderLayers>>,
);

/// Shows the bone `entity`, whose visibility is `visibility` and which is on the mirror layer
/// alone when `own` holds, as `want`; or, where the `body` is drawn and only the camera at the
/// eyes must not see it, on the mirror layer alone: the head of a first person view still shows
/// in calm water. A part on the mirror layer alone casts no shadow, as a hidden one.
fn part(
    commands: &mut Commands,
    (entity, mut visibility, own): (Entity, Mut<Visibility>, bool),
    want: Visibility,
    body: Visibility,
) {
    let mirrored = want == Visibility::Hidden && body != Visibility::Hidden;
    let shown = if mirrored {
        Visibility::Inherited
    } else {
        want
    };
    visibility.set_if_neq(shown);
    match (mirrored, own) {
        (true, false) => {
            commands.entity(entity).insert(Propagate(mirror_only()));
        }
        (false, true) => {
            commands.entity(entity).remove::<Propagate<RenderLayers>>();
        }
        _ => {}
    }
}

/// Applies the visibility setting to the body at this machine: its root, its head and its
/// arms, which stay in a first person view that looks up while they strike. Any other body is
/// drawn whole. What the camera at the eyes must not see stays in the water mirror.
pub fn show(
    (shown, own): (Res<Shown>, Own),
    mut commands: Commands,
    bodies: Query<(&AvatarPose, &Skeleton, Option<&Strike>)>,
    mut roots: Query<&mut Visibility, (With<Avatar>, Without<Bone>)>,
    mut bones: Query<(Entity, BoneParts), Without<Avatar>>,
) {
    let Some((_, body)) = own.body() else {
        return;
    };
    let (Ok((input, skeleton, strike)), Ok(mut root)) = (bodies.get(body), roots.get_mut(body))
    else {
        return;
    };
    let striking = strike.is_some_and(|strike| strike.act != Act::Ready);
    let parts = drawn(*shown, if striking { 0.0 } else { input.look.y });
    root.set_if_neq(parts.body);
    let mut bones = bones.iter_many_mut(skeleton.0);
    while let Some(found) = bones.fetch_next() {
        let Ok((entity, (bone, visibility, layers))) = found else {
            continue;
        };
        let want = match bone.0 {
            Seg::Head => parts.head,
            seg if arm(seg) => parts.arms,
            _ => continue,
        };
        part(
            &mut commands,
            (entity, visibility, layers.is_some()),
            want,
            parts.body,
        );
    }
}
