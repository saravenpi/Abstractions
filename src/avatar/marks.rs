//! Where the flight gear sits on the posed body: the wingtips and the boot nozzles.
//!
//! The trails and the exhaust of the gear leave from these points, so they follow the body in
//! every pose instead of a fixed offset from its centre. The body at this machine publishes
//! them as a resource; every other body that carries [`Marked`] gets its own.

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

/// The marks of a body other than the one at this machine, in the frame of its root. A body
/// that carries it gets the marks of its pose each frame.
#[derive(Component, Debug, Default, Clone, Copy, PartialEq)]
pub struct Marked(pub Marks);

impl Default for Marks {
    /// The marks of the body standing at rest.
    fn default() -> Self {
        Self::of(&Pose::default())
    }
}

/// Publishes the marks of the pose applied this frame to the body at this machine: the gear
/// effects that leave from them are the ones of the local player. Every body that carries
/// its own marks gets those of its pose.
pub fn mark(own: Own, mut bodies: Query<(&Anim, Option<&mut Marked>)>, mut marks: ResMut<Marks>) {
    let body = own.body().and_then(|(_, body)| bodies.get(body).ok());
    if let Some((body, _)) = body {
        marks.set_if_neq(Marks::of(&body.pose));
    }
    for (body, mine) in &mut bodies {
        if let Some(mut mine) = mine {
            mine.set_if_neq(Marked(Marks::of(&body.pose)));
        }
    }
}
