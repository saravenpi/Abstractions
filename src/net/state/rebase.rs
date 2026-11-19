//! Moving the records of a thing into another wire frame when its newest record changed frame,
//! each one at its own time, so the thing goes on smoothly instead of starting over.

use abstractions_net::jitter::{Buffer, Snap};
use abstractions_net::wire;
use bevy::math::DVec3;

use super::bodies::Bodies;
use super::frame::{Place, Pose};

/// Moves every snapshot of `buffer` from the frame `from` into the frame `to` at the time of
/// the snapshot. A snapshot that cannot be moved, around another star, is dropped.
pub fn rebase<E: Copy + Default>(
    buffer: &mut Buffer<E>,
    (from, to): (wire::Frame, wire::Frame),
    bodies: &mut Bodies,
) {
    let (from, to) = (Place::from(from), Place::from(to));
    buffer.rebase(|snap| {
        let pose = Pose {
            pos: snap.position,
            vel: snap.velocity.map_or(DVec3::ZERO, |vel| vel.as_dvec3()),
            rot: snap.rotation.as_dquat(),
        };
        let moved = bodies.convert(pose, from, to, snap.time)?;
        Some(Snap {
            position: moved.pos,
            velocity: snap.velocity.map(|_| moved.vel.as_vec3()),
            rotation: moved.rot.as_quat(),
            ..snap
        })
    });
}
