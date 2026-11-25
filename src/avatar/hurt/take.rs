//! Taking a blow: the contact point picks the part, the energy and the edge decide the harm.

use bevy::prelude::*;

use super::super::animate::Anim;
use super::super::blow::Blow;
use super::super::own::Own;
use super::super::rig::globals;
use super::plates::nearest;
use crate::condition::{Condition, Part, Shield};
use crate::craft::Pack;

/// Returns the part of the body at `placed` (its transform in render space) that a blow at `at`
/// lands on, from the segments of `anim`: the one whose plate is nearest the contact point.
pub fn struck(placed: &GlobalTransform, anim: &Anim, at: Vec3) -> Part {
    let local = placed.affine().inverse().transform_point3(at);
    nearest(local, &globals(&anim.pose)).0
}

/// Lets every blow that lands on the robot hit its energy shield first, then reduce any overflow
/// by equipped armor defense before hurting the struck part. A blow with no body falls on the torso.
pub fn take(
    mut blows: MessageReader<Blow>,
    (mut condition, mut shield, pack): (
        ResMut<Condition>,
        Option<ResMut<Shield>>,
        Option<Res<Pack>>,
    ),
    own: Own,
    bodies: Query<(&GlobalTransform, &Anim)>,
) {
    let body = own.body().and_then(|(_, body)| bodies.get(body).ok());
    for blow in blows.read() {
        let part = body.map_or(Part::Torso, |(placed, anim)| struck(placed, anim, blow.at));
        let overflow = match shield.as_deref_mut() {
            Some(s) => s.absorb(blow.joules),
            None => blow.joules,
        };
        if overflow <= 0.0 {
            continue;
        }
        let defense = pack.as_deref().map_or(0.0, |p| p.defense_for(part));
        let reduced = overflow * (1.0 - defense).clamp(0.0, 1.0);
        condition.hurt(part, reduced, blow.cut);
    }
}
