//! Cramp: a held tool kept out of the world. Every frame the tool as carried at rest is laid along
//! its arm and its blade, from the shoulder to a little past the tip, against the walls of the base,
//! the ship, rocks, trees and the ground. Where it would enter them the hand draws it in to the chest
//! and the shaft rises, as a rifle is raised near a wall, just as far as keeps it clear; once clear
//! it comes back down over [`CRAMP_EASE`] seconds. A blow started raised begins from there, and
//! [`super::bump`] stops it at the wall.

use avian3d::prelude::*;
use bevy::prelude::*;

use super::super::pose::Pose;
use super::super::rig::{Seg, globals};
use super::super::tune::SHOULDER;
use super::feel::tune::{CRAMP_EASE, CRAMP_HALVES, MARGIN};
use super::held::Held;
use super::key::{carry, raised};
use super::layer::Layer;
use super::solid::{Land, Meets, Solids};
use super::tune::HAND_GRIP;
use crate::craft::build::Hands;

/// How far the world asks the tool of a body to be raised, 0 carried to 1 raised to the chest, on
/// its root.
#[derive(Component, Debug, Clone, Copy, PartialEq, Default)]
pub struct Cramped(pub f32);

/// Returns the shoulder, the grip and the tip of a tool `reach` metres from the hand carried at rest
/// and raised `raise` of the way, in the frame of the body, through the rig the body is drawn with.
pub fn rest(raise: f32, reach: f32) -> [Vec3; 3] {
    let key = carry().toward(&raised(), raise);
    let bones = globals(&Layer::of(&key, Hands::One).apply(Pose::default()));
    let (torso, hand) = (bones[Seg::Torso as usize], bones[Seg::HandR as usize]);
    let grip = hand.transform_point(Vec3::new(0.0, -HAND_GRIP, 0.0));
    let shoulder = torso.transform_point(Vec3::from_array(SHOULDER));
    [shoulder, grip, grip + torso.rotation * key.axis * reach]
}

/// Returns true when the arm or the blade of `chain`, a shoulder, a grip and a tip in render space,
/// meets `solids` up to [`MARGIN`] past the tip.
fn blocked(solids: &impl Meets, [shoulder, grip, tip]: [Vec3; 3]) -> bool {
    let past = tip + (tip - grip).normalize_or_zero() * MARGIN;
    solids.meet(shoulder, grip).is_some() || solids.meet(grip, past).is_some()
}

/// Returns the least raise that keeps a tool `reach` metres from the hand clear of `solids` when
/// the body that carries it is placed by `place`, from the frame of the body to render space: none
/// when it is clear carried, all the way when even raised it is not.
pub fn need(solids: &impl Meets, place: impl Fn(Vec3) -> Vec3, reach: f32) -> f32 {
    let clear = |raise: f32| !blocked(solids, rest(raise, reach).map(&place));
    if clear(0.0) {
        return 0.0;
    }
    let (mut low, mut high) = (0.0, 1.0);
    for _ in 0..CRAMP_HALVES {
        let mid = 0.5 * (low + high);
        match clear(mid) {
            true => high = mid,
            false => low = mid,
        }
    }
    high
}

/// Returns the raise of a tool `dt` seconds after it was `was` when the world asks `need`: as high
/// as asked at once, and down by a whole raise over [`CRAMP_EASE`] seconds once it is asked less.
pub fn ease(was: f32, need: f32, dt: f32) -> f32 {
    (was - dt / CRAMP_EASE).max(need).clamp(0.0, 1.0)
}

/// Asks of every body that holds a tool out how far the world raises it.
pub fn cramp(
    mut commands: Commands,
    (world, land): (SpatialQuery, Land),
    mut bodies: Query<(
        Entity,
        &GlobalTransform,
        &Held,
        &ChildOf,
        Option<&mut Cramped>,
    )>,
) {
    for (body, placed, held, of, cramped) in &mut bodies {
        let want = match held.out {
            true => {
                let solids = Solids::new(&world, &land, of.parent());
                need(&solids, |p| placed.transform_point(p), held.build.reach())
            }
            false => 0.0,
        };
        match cramped {
            Some(mut cramped) => cramped.set_if_neq(Cramped(want)),
            None => {
                commands.entity(body).insert(Cramped(want));
                true
            }
        };
    }
}
