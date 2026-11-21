//! The strike layer: what a strike lays over the blended pose of the body, the chest turned, the
//! arms on the grip, the legs and the hips behind the blow and the free arm against it.

use bevy::prelude::*;

use super::super::AvatarPose;
use super::super::ground::Stance;
use super::super::own::Own;
use super::super::pose::{Core, Limb, Pose};
use super::super::show::Shown;
use super::act::{Strike, Timing, key};
use super::feel::bend::Bend;
use super::feel::body::{phase, stand, still};
use super::held::Held;
use super::ik::reach;
use super::javelin::{Release, draw, drawing, released};
use super::key::Key;
use super::path::Path;
use super::tune::SECOND_HAND;
use crate::craft::build::Hands;

/// The joints of an arm the layer sets, in the order [`reach`] gives them.
const ARM: [Limb; 3] = [Limb::Shoulder, Limb::ArmOut, Limb::Elbow];

/// What a strike lays over the pose of a body, on its root. A weight of 0 leaves the pose alone.
#[derive(Component, Debug, Clone, Copy, PartialEq, Default)]
pub struct Layer {
    /// How much of the layer is in the pose, 0 to 1.
    pub weight: f32,
    /// Chest turn and bend added to the pose, in radians.
    pub trunk: [f32; 2],
    /// Shoulder, lift and elbow of each arm that holds the grip, left then right.
    pub arms: [Option<[f32; 3]>; 2],
    /// Unit direction of the tool from the grip, in the frame of the torso.
    pub axis: Vec3,
    /// How far the strike is under way, 0 at rest to 1 in the blow.
    pub engage: f32,
    /// How far the strike is coiled, -1 cocked to 1 followed through.
    pub coil: f32,
    /// How much of the stance the legs take, 0 to 1.
    pub legs: f32,
    /// Where the legs and the hips stand behind the blow.
    pub stance: Stance,
    /// Shoulder, lift and elbow of the free left arm that balances a one-handed blow.
    pub free: Option<[f32; 3]>,
    /// Unit direction of a drawn javelin in the frame of the body, which [`Layer::axis`] gives in
    /// the torso only as near as the torso follows the layer: the shaft lies along the aim exactly.
    pub aim: Option<Vec3>,
}

impl Layer {
    /// Returns the layer that poses `key` held in `hands`: the right hand on the grip, and the left
    /// one further down the shaft when both hold it.
    pub fn of(key: &Key, hands: Hands) -> Self {
        Self::held(key, hands, [true; 2])
    }

    /// Returns the layer that poses `key` held in `hands` by the arms that `usable` leaves, left then
    /// right: a tool of two hands is held by the one arm that is left, and a body with no arm left
    /// holds nothing.
    pub fn held(key: &Key, hands: Hands, usable: [bool; 2]) -> Self {
        let second = || reach(key.grip - key.axis * SECOND_HAND, 0);
        let arms = match usable {
            [true, true] => [(hands == Hands::Two).then(second), Some(reach(key.grip, 1))],
            [_, true] => [None, Some(reach(key.grip, 1))],
            [true, false] => [Some(reach(key.grip, 0)), None],
            [false, false] => [None, None],
        };
        Self {
            weight: 1.0,
            trunk: [key.twist, key.spine],
            arms,
            axis: key.axis,
            ..default()
        }
    }

    /// Returns `pose` with this layer laid over it: the chest turn and bend added, the arms that
    /// hold the grip blended to their reach, the legs and the free arm behind the blow, every joint
    /// kept inside its limits.
    pub fn apply(&self, pose: Pose) -> Pose {
        if self.weight <= 0.0 {
            return pose;
        }
        let w = self.weight.min(1.0);
        let mut out = pose
            .with(Core::Twist, pose.core(Core::Twist) + self.trunk[0] * w)
            .with(Core::Spine, pose.core(Core::Spine) + self.trunk[1] * w);
        for (side, arm) in self.arms.iter().enumerate() {
            let Some(arm) = arm else { continue };
            for (joint, value) in ARM.into_iter().zip(arm) {
                let base = out.sides[side][joint as usize];
                out.sides[side][joint as usize] = base + (value - base) * w;
            }
        }
        stand(out, self, w).bounded()
    }
}

/// Returns the layer `strike` lays with `held` on a body posed by `pose`: the key of its act turned
/// toward its aim as far as the strike is under way, with the body behind it, eased from the layer
/// `before` over `dt` seconds.
pub fn posed(
    strike: &Strike,
    held: &Held,
    pose: &AvatarPose,
    (before, dt): (&Layer, f32),
) -> Layer {
    let path = Path::of(held.style, strike.dir);
    let timing = Timing::of(held, &path)
        .felt(&strike.hurt)
        .braked(strike.result);
    let bend = Bend::of(strike.aim, still(pose));
    let (engage, _) = phase(strike.act, &timing);
    let key = bend.turn(&key(strike.act, &path, &timing, strike.dir), engage);
    Layer::held(&key, held.build.hands, strike.hurt.usable).bodied(
        (strike, held),
        (&bend, &timing),
        (before, dt),
    )
}

/// The bodies a strike lays its layer on.
type Bodies<'w, 's> = Query<
    'w,
    's,
    (
        Entity,
        &'static mut Layer,
        &'static Strike,
        Option<&'static Held>,
        Option<&'static AvatarPose>,
        Option<&'static mut Release>,
    ),
>;

/// Returns the layer of a body whose strike is `strike`, holding `held` out, posed by `pose`, eased
/// from `before` over `dt` seconds, seen in `first` person: a javelin drawn along the aim, else the
/// key of its act.
fn holding(
    strike: &Strike,
    held: &Held,
    pose: &AvatarPose,
    (before, dt, first): (&Layer, f32, bool),
) -> Layer {
    match drawing(strike, held) {
        Some(aim) => draw((strike, held), pose, aim, (before, dt), first),
        None => posed(strike, held, pose, (before, dt)),
    }
}

/// Lays the strike of every body over its pose: the key of its act with the tool out, a javelin
/// drawn back along the aim, the whip of one just let go, and nothing with the tool stowed or gone.
/// The local body in first person holds a drawn javelin where its eyes see it.
pub fn lay(
    mut commands: Commands,
    (time, shown, own): (Res<Time>, Option<Res<Shown>>, Own),
    mut bodies: Bodies,
) {
    let dt = time.delta_secs();
    let mine = own.body().map(|(_, body)| body);
    let seen = shown.is_some_and(|shown| *shown == Shown::FirstPerson);
    for (body, mut layer, strike, held, pose, release) in &mut bodies {
        let pose = pose.copied().unwrap_or_default();
        let first = seen && mine == Some(body);
        let thrown = release.and_then(|mut release| {
            release.age += dt;
            let whip = released(&release, strike.hurt.usable, first);
            if whip.is_none() {
                commands.entity(body).remove::<Release>();
            }
            whip
        });
        let next = match held.filter(|held| held.out) {
            Some(held) => holding(strike, held, &pose, (&layer, dt, first)),
            None => thrown.unwrap_or_default(),
        };
        layer.set_if_neq(next);
    }
}
