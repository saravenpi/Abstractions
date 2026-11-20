//! The systems of the seat: how much of the seated body is drawn, and the seated pose written
//! over the animated one, hands on the controls of the ship the body sits in.

use bevy::prelude::*;

use super::{held, offset, seated, shod};
use crate::avatar::animate::{Anim, Bone, Skeleton};
use crate::avatar::neck::reach;
use crate::avatar::pose::{Core, Pose};
use crate::avatar::rig::locals;
use crate::avatar::{Avatar, Mode, Shown};
use crate::player::wrap;
use crate::ship::cockpit::{Cockpit, Levers};
use crate::sim::PlayerId;
use crate::space::{Location, Travel};
use crate::view::{BodyFade, Heading};

/// Share of the body fade above which the whole body is drawn, head and screen included.
const SHOW_HEAD: f32 = 0.5;

/// Returns where the head of the pilot looks and how much of the body is drawn, for the look
/// angles `heading` of the view around the nose and the body `fade` of the camera rig: at the
/// eyes the head follows the view and is hidden, behind the ship it faces ahead and is drawn.
pub fn piloting(heading: &Heading, fade: f32) -> (Vec2, Shown) {
    let fade = fade.clamp(0.0, 1.0);
    let look = Vec2::new(wrap(heading.yaw), heading.pitch) * (1.0 - fade);
    let shown = match fade > SHOW_HEAD {
        true => Shown::Full,
        false => Shown::FirstPerson,
    };
    (look, shown)
}

/// Draws the body of the player at this machine while it sits in the ship, over what the
/// dresser chose: whole from outside, without its head at the eyes.
pub fn shown(
    location: Option<Res<Location>>,
    fade: Option<Res<BodyFade>>,
    mut drawn: ResMut<Shown>,
) {
    let aboard = location.is_some_and(|location| location.travel == Travel::InShip);
    if let Some(fade) = fade.filter(|_| aboard) {
        drawn.set_if_neq(piloting(&Heading::default(), fade.0).1);
    }
}

/// Returns the pose of a body whose animation gives `animated`, `share` of the way into the
/// seat with its hands on the controls `levers` and its head turned to `look` as far as the
/// neck reaches.
pub fn posed(animated: &Pose, levers: &Levers, look: Vec2, share: f32) -> Pose {
    let head = reach(look);
    let sat = shod(seated(held(levers)), levers)
        .with(Core::HeadYaw, head.x)
        .with(Core::HeadPitch, head.y);
    animated.plus(animated, -share).plus(&sat, share)
}

/// What the seat reads besides the body.
type View<'w> = (Option<Res<'w, Heading>>, Option<Res<'w, BodyFade>>);

/// Returns how much of the seated pose a body takes: all of it in a pilot seat, else the
/// weight its animation still gives the seat while it blends out.
fn seat_share(piloting: bool, anim: &Anim) -> f32 {
    match piloting {
        true => 1.0,
        false => anim.weights.eased()[Mode::Seated as usize],
    }
}

/// Returns where the head of a pilot looks: the view of the player at this machine when
/// `turns`, straight ahead for any other body.
fn pilot_look(view: &View, turns: bool) -> Vec2 {
    match (view, turns) {
        ((Some(heading), Some(fade)), true) => piloting(heading, fade.0).0,
        _ => Vec2::ZERO,
    }
}

/// The parts of a body the seat moves: its animation, the pose of its root, the player it
/// hangs from and its segments.
type Seated = (
    &'static Anim,
    &'static mut Transform,
    &'static ChildOf,
    &'static Skeleton,
);

/// Seats every body: while its player is the pilot of a ship the root of the body is in the
/// seat and the pose is the seated one, hands on the controls of that ship. The head of the
/// player at this machine turns with the view. After leaving, the pose blends back out with
/// the weight the animation still gives the seat.
pub fn sit(
    view: View,
    (cockpits, players): (Query<&Cockpit>, Query<&PlayerId>),
    mut bodies: Query<Seated, With<Avatar>>,
    mut bones: Query<(&Bone, &mut Transform), Without<Avatar>>,
) {
    for (anim, mut root, parent, skeleton) in &mut bodies {
        let player = parent.parent();
        let cockpit = cockpits.iter().find(|ship| ship.pilot == Some(player));
        let place = cockpit.map_or(Vec3::ZERO, |_| offset());
        if root.translation != place {
            root.translation = place;
        }
        let share = seat_share(cockpit.is_some(), anim);
        if share <= 0.0 {
            continue;
        }
        let local = players.get(player).is_ok_and(|id| *id == PlayerId::LOCAL);
        let look = pilot_look(&view, local && cockpit.is_some());
        let levers = cockpit.map_or(Levers::default(), |ship| ship.shown);
        let frames = locals(&posed(&anim.pose, &levers, look, share));
        for &segment in &skeleton.0 {
            if let Ok((bone, mut transform)) = bones.get_mut(segment) {
                *transform = frames[bone.0 as usize];
            }
        }
    }
}
