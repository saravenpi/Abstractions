//! Taking a blow: the contact point picks the part, the energy and the edge decide the harm.

use bevy::prelude::*;

use super::super::animate::Anim;
use super::super::blow::Blow;
use super::super::own::Own;
use super::super::rig::globals;
use super::plates::nearest;
use crate::condition::{Condition, Part};

/// Returns the part of the body at `placed` (its transform in render space) that a blow at `at`
/// lands on, from the segments of `anim`: the one whose plate is nearest the contact point.
pub fn struck(placed: &GlobalTransform, anim: &Anim, at: Vec3) -> Part {
    let local = placed.affine().inverse().transform_point3(at);
    nearest(local, &globals(&anim.pose)).0
}

/// Lets every blow that lands on the robot hurt the part it struck. A blow with no body to land on
/// falls on the torso.
pub fn take(
    mut blows: MessageReader<Blow>,
    mut condition: ResMut<Condition>,
    own: Own,
    bodies: Query<(&GlobalTransform, &Anim)>,
) {
    let body = own.body().and_then(|(_, body)| bodies.get(body).ok());
    for blow in blows.read() {
        let part = body.map_or(Part::Torso, |(placed, anim)| struck(placed, anim, blow.at));
        condition.hurt(part, blow.joules, blow.cut);
    }
}
