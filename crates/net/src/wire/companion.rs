//! The body of a companion record: where a player's companion hovers and its body language.
//!
//! 25 bytes on a surface, 37 in the other frames. It holds no velocity: a companion is drawn
//! by blending straight between two samples.

use std::f32::consts::{PI, TAU};

use glam::{DVec3, Quat};

use super::Frame;
use super::bytes::{Reader, put_position};
use crate::pack::{pack_i8, pack_i16, pack_rotation, pack_unit};
use crate::pack::{unpack_i8, unpack_i16, unpack_rotation, unpack_unit};

/// Largest extra height a record holds, either way, in metres.
pub const LIFT_RANGE: f32 = 1.0;

/// What a remote companion needs to be drawn: its place and the fields of the game's companion
/// `Pose`. Values out of their range are clamped when encoded.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct CompanionBody {
    /// Position in the frame of the record, in metres. Kept to the millimetre on a surface.
    pub position: DVec3,
    /// Orientation of the plain hover, before the pose is added. Kept within 0.005 rad.
    pub rotation: Quat,
    /// Extra height of the pose in metres, within [`LIFT_RANGE`].
    pub lift: f32,
    /// Turn of the pose about the vertical, from -2 pi to 2 pi in 16 bits.
    pub yaw: f32,
    /// Nod of the pose, positive looks up, from -pi to pi in 8 bits.
    pub pitch: f32,
    /// Head tilt of the pose to the side, from -pi to pi in 8 bits.
    pub roll: f32,
    /// How far the hands are stretched out, from 0 to 1.
    pub spread: f32,
    /// How much the hands wave in turn, from 0 to 1.
    pub wave: f32,
    /// Expression the face shows, as the game numbers them. Kept as it is.
    pub expr: u8,
    /// Face state in bits: lid in bits 0 and 1, gaze x in bits 2 and 3, gaze y in bits 4 and 5,
    /// heartbeat in bit 6. Kept as it is.
    pub face: u8,
}

impl CompanionBody {
    /// Adds the body to `out`, with its position packed for `frame`.
    pub fn encode(&self, frame: &Frame, out: &mut Vec<u8>) {
        put_position(out, frame, self.position);
        out.extend_from_slice(&pack_rotation(self.rotation).to_le_bytes());
        out.push(pack_i8(self.lift, LIFT_RANGE) as u8);
        out.extend_from_slice(&pack_i16(self.yaw, TAU).to_le_bytes());
        out.extend_from_slice(&[
            pack_i8(self.pitch, PI) as u8,
            pack_i8(self.roll, PI) as u8,
            pack_unit(self.spread),
            pack_unit(self.wave),
            self.expr,
            self.face,
        ]);
    }

    /// Returns the body [`CompanionBody::encode`] wrote for `frame`, or nothing when `bytes`
    /// has the wrong length or holds a number that is not finite.
    pub fn decode(frame: &Frame, bytes: &[u8]) -> Option<Self> {
        let mut reader = Reader(bytes);
        let position = reader.position(frame)?;
        let rotation = unpack_rotation(u32::from_le_bytes(reader.take()?));
        let [lift] = reader.take()?;
        let yaw = unpack_i16(i16::from_le_bytes(reader.take()?), TAU);
        let [pitch, roll, spread, wave, expr, face] = reader.take()?;
        reader.end(Self {
            position,
            rotation,
            lift: unpack_i8(lift as i8, LIFT_RANGE),
            yaw,
            pitch: unpack_i8(pitch as i8, PI),
            roll: unpack_i8(roll as i8, PI),
            spread: unpack_unit(spread),
            wave: unpack_unit(wave),
            expr,
            face,
        })
    }
}
