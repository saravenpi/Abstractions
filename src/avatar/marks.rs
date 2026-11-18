//! Where the flight gear sits on the posed body: the wingtips and the boot nozzles.
//!
//! The trails and the exhaust of the gear leave from these points, so they follow the body in
//! every pose instead of a fixed offset from its centre.

use bevy::prelude::*;

use super::animate::Anim;
use super::own::Own;
use super::pose::Pose;
use super::rig::{Seg, globals};
use super::tune::{ANKLE_RISE, NOZZLE};

/// Points of the posed body in the frame of the player entity: origin at the capsule centre,
/// +Y up, -Z ahead. Left comes first.
#[derive(Resource, Debug, Clone, Copy, PartialEq)]
pub struct Marks {
    /// The wingtips: the wrists, where the fabric of the arms ends.
    pub tips: [Vec3; 2],
    /// The nozzles under the soles.
    pub soles: [Vec3; 2],
    /// Unit direction the exhaust leaves each nozzle along.
    pub jets: [Vec3; 2],
}

impl Marks {
    /// Returns the marks of a body in `pose`.
    pub fn of(pose: &Pose) -> Self {
        let joints = globals(pose);
        let boots = [Seg::BootL, Seg::BootR].map(|seg| joints[seg as usize]);
        let nozzle = Vec3::new(0.0, -ANKLE_RISE, NOZZLE.1);
        Self {
            tips: [Seg::HandL, Seg::HandR].map(|seg| joints[seg as usize].translation),
            soles: boots.map(|boot| boot.transform_point(nozzle)),
            jets: boots.map(|boot| boot.rotation * Vec3::NEG_Y),
        }
    }
}

impl Default for Marks {
    /// The marks of the body standing at rest.
    fn default() -> Self {
        Self::of(&Pose::default())
    }
}

/// Publishes the marks of the pose applied this frame to the body at this machine: the gear
/// effects that leave from them are the ones of the local player.
pub fn mark(own: Own, bodies: Query<&Anim>, mut marks: ResMut<Marks>) {
    let body = own.body().and_then(|(_, body)| bodies.get(body).ok());
    if let Some(body) = body {
        marks.set_if_neq(Marks::of(&body.pose));
    }
}
