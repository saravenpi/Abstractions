//! The body of a vitals record: how hurt each part of a robot is, and the glow around it.
//!
//! 11 bytes in every frame. It holds no position: its head names the frame, which is the one of the
//! robot record sent with it. It travels apart from the robot so that sixteen robots fit one datagram.

use super::bytes::Reader;
use crate::pack::{pack_unit, unpack_unit};

/// Number of parts of the robot that carry health, the length of [`VitalsBody::harm`].
pub const HARM_PARTS: usize = 10;

/// What a watcher needs to show a robot hurt or glowing.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct VitalsBody {
    /// How much each part is hurt, from 0 (whole) to 1 (destroyed), in the order of the game's
    /// `Part::ALL`: head, torso, left arm, right arm, left leg, right leg, left wing, right wing,
    /// left boot, right boot.
    pub harm: [f32; HARM_PARTS],
    /// Glow of an atmospheric entry around the robot, from 0 to 1.
    pub heat: f32,
}

impl VitalsBody {
    /// Adds the body to `out`.
    pub fn encode(&self, out: &mut Vec<u8>) {
        out.extend(self.harm.map(pack_unit));
        out.push(pack_unit(self.heat));
    }

    /// Returns the body [`VitalsBody::encode`] wrote, or nothing when `bytes` has the wrong length.
    pub fn decode(bytes: &[u8]) -> Option<Self> {
        let mut reader = Reader(bytes);
        let harm = reader.take::<HARM_PARTS>()?.map(unpack_unit);
        let [heat] = reader.take()?;
        reader.end(Self {
            harm,
            heat: unpack_unit(heat),
        })
    }
}
