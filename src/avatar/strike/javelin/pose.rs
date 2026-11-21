//! The layer of a throw: the javelin drawn back along the aim with the body coiled behind it, and
//! the whip of the hand through the aim line once it is let go, fading back to the pose of the body.

use bevy::prelude::*;

use super::super::super::AvatarPose;
use super::super::act::{Act, Strike, Timing};
use super::super::feel::bend::Bend;
use super::super::feel::body::still;
use super::super::held::Held;
use super::super::key::{Key, carry};
use super::super::layer::Layer;
use super::super::path::Path;
use super::super::tune::DRAW;
use super::cock::{drawn, pointing, torso, whipped};
use super::tune::RELEASE;
use crate::craft::build::Hands;

/// A javelin just let go by a body, on its root while the hand whips through.
#[derive(Component, Debug, Clone, Copy, PartialEq)]
pub struct Release {
    /// The point it was thrown at, in the frame of the body.
    pub at: Vec3,
    /// The share of a full draw it was let go at.
    pub share: f32,
    /// Seconds since it let go.
    pub age: f32,
}

/// Returns the point `strike` draws a javelin back at, in the frame of the body, and the share of a
/// full draw, while it draws one with `held`.
pub fn drawing(strike: &Strike, held: &Held) -> Option<(Vec3, f32)> {
    match strike.act {
        Act::Draw(t) if held.thrown() => strike.aim.map(|aim| (aim, (t / DRAW).min(1.0))),
        _ => None,
    }
}

/// Returns the key of a javelin drawn `share` of the way back at `aim` in `first` person, and the way
/// of its shaft in the frame of the body.
pub fn cocked((aim, share): (Vec3, f32), first: bool) -> (Key, Vec3) {
    let (full, _) = drawn(aim, first);
    let key = carry().toward(&full, share);
    let chest = torso([key.twist, key.spine]);
    (key, (chest.rotation * key.axis).normalize())
}

/// Returns the layer of `strike` drawing `held` back at `aim` on a body posed by `pose`: one hand on
/// the shaft along the aim, the free arm toward the target, the legs behind the throw, eased from
/// the layer `before` over `dt` seconds.
pub fn draw(
    (strike, held): (&Strike, &Held),
    pose: &AvatarPose,
    aim: (Vec3, f32),
    (before, dt): (&Layer, f32),
    first: bool,
) -> Layer {
    let (key, way) = cocked(aim, first);
    let timing = Timing::of(held, &Path::of(held.style, strike.dir)).felt(&strike.hurt);
    let bend = Bend {
        still: still(pose),
        ..default()
    };
    let mut layer = Layer::held(&key, Hands::One, strike.hurt.usable).bodied(
        (strike, held),
        (&bend, &timing),
        (before, dt),
    );
    layer.aim = Some(way);
    layer.free = strike.hurt.usable[0].then(|| pointing(key.axis));
    layer
}

/// Returns the layer of the hand whipping through after `release` in `first` person, held by the
/// arms `usable` left, or nothing once it has faded.
pub fn released(release: &Release, usable: [bool; 2], first: bool) -> Option<Layer> {
    let (whip, fade) = RELEASE;
    if release.age >= whip + fade {
        return None;
    }
    let (from, way) = cocked((release.at, release.share), first);
    let key = from.toward(&whipped(way), release.age / whip);
    let gone = ((release.age - whip) / fade).clamp(0.0, 1.0);
    let mut layer = Layer::held(&key, Hands::One, usable);
    layer.weight = 1.0 - gone * gone * (3.0 - 2.0 * gone);
    layer.engage = 1.0;
    layer.free = usable[0].then(|| pointing(key.axis));
    Some(layer)
}
