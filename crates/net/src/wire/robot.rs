//! The body of a robot record: where a player's robot is and how it is posed.
//!
//! 46 bytes on a surface, 55 in the other frames. Angles are radians, speeds metres per second.

use std::f32::consts::{FRAC_PI_2, PI, TAU};

use glam::{DVec3, Vec2, Vec3};

use super::bytes::{Reader, put_position, put_vec3};
use super::{Body, Frame, Kind};
use crate::pack::{pack_i8, pack_i16, pack_u16, pack_unit};
use crate::pack::{unpack_i8, unpack_i16, unpack_u16, unpack_unit};

/// Fastest horizontal speed a record holds, in metres per second. The step is 1 cm/s.
pub const SPEED_RANGE: f32 = 655.35;

/// Fastest speed along the local up a record holds, either way, in metres per second.
pub const CLIMB_RANGE: f32 = 327.67;

/// Metres after which the stride distance on the wire starts again from zero.
pub const STRIDE_WRAP: f32 = 655.36;

/// Fastest airspeed a record holds, in metres per second. The step is 1 m/s.
pub const AIRSPEED_RANGE: f32 = 255.0;

/// What a remote robot needs to be drawn: the fields of the game's `AvatarPose`, its place and
/// its heading. Values out of their range are clamped when encoded.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct RobotBody {
    /// Position of the body in the frame of the record, in metres. Kept to the millimetre on a surface.
    pub position: DVec3,
    /// Velocity in the frame of the record. Used to interpolate between two samples.
    pub velocity: Vec3,
    /// Heading about the local up, kept from -pi to pi in 16 bits.
    pub yaw: f32,
    /// What the body is doing: the rank of the game's `Mode` in `Mode::ALL`.
    pub mode: u8,
    /// Horizontal speed, from 0 to [`SPEED_RANGE`].
    pub speed: f32,
    /// Speed along the local up, negative when falling, within [`CLIMB_RANGE`].
    pub vertical_speed: f32,
    /// Distance walked in metres. It comes back between 0 and [`STRIDE_WRAP`]: add up
    /// [`stride_delta`] of successive samples to get an ever growing distance again.
    pub stride_distance: f32,
    /// Bank into a turn, from -pi to pi in 8 bits.
    pub bank: f32,
    /// Pitch of the flight path, from -pi to pi in 8 bits.
    pub pitch: f32,
    /// Thrust of the boot boosters, from 0 to 1.
    pub thrust: f32,
    /// How far the wingsuit is deployed, from 0 to 1.
    pub wings: f32,
    /// Speed through the air, from 0 to [`AIRSPEED_RANGE`].
    pub airspeed: f32,
    /// Where the head looks: `x` is the yaw to the left within pi, `y` the pitch up within pi/2.
    pub look: Vec2,
    /// How hard the last landing was, from 0 to 1.
    pub landed_hard: f32,
    /// How far the body is crouched, from 0 (standing) to 1 (fully down). The step is 1/15.
    pub crouch: f32,
    /// What the hands do with the tool: the rank of the game's act in bits 0 to 2 (0 ready, 1
    /// wind-up, 2 swing, 3 recover, 4 guard, 5 draw, 6 stun) and the side of the strike in bits 3
    /// and 4. Kept as it is.
    pub swing: u8,
    /// Seconds since the act began, from 0 to [`PHASE_RANGE`]. The step is 0.13 s.
    pub phase: f32,
}

/// Returns two shares from 0 to 1 as the two halves of one byte, the first in the high half.
fn nibbles(high: f32, low: f32) -> u8 {
    let half = |share: f32| (share.clamp(0.0, 1.0) * 15.0).round() as u8;
    half(high) << 4 | half(low)
}

/// Longest time into an act a record holds, in seconds.
pub const PHASE_RANGE: f32 = 2.0;

/// Returns how far the stride moved from the sample `before` to the sample `after`, in metres,
/// seeing through the wrap at [`STRIDE_WRAP`].
pub fn stride_delta(before: f32, after: f32) -> f32 {
    (after - before + STRIDE_WRAP / 2.0).rem_euclid(STRIDE_WRAP) - STRIDE_WRAP / 2.0
}

impl RobotBody {
    /// Returns the ten fields that take one byte each, in wire order.
    fn small(&self) -> [u8; 10] {
        [
            pack_i8(self.bank, PI) as u8,
            pack_i8(self.pitch, PI) as u8,
            pack_unit(self.thrust),
            pack_unit(self.wings),
            pack_unit(self.airspeed / AIRSPEED_RANGE),
            pack_i8(self.look.x, PI) as u8,
            pack_i8(self.look.y, FRAC_PI_2) as u8,
            pack_unit(self.landed_hard),
            nibbles(self.crouch, self.phase / PHASE_RANGE),
            self.swing,
        ]
    }

    /// Returns a body at rest at the origin with the ten one-byte fields read from `small`.
    fn with_small(small: [u8; 10]) -> Self {
        Self {
            bank: unpack_i8(small[0] as i8, PI),
            pitch: unpack_i8(small[1] as i8, PI),
            thrust: unpack_unit(small[2]),
            wings: unpack_unit(small[3]),
            airspeed: unpack_unit(small[4]) * AIRSPEED_RANGE,
            look: Vec2::new(
                unpack_i8(small[5] as i8, PI),
                unpack_i8(small[6] as i8, FRAC_PI_2),
            ),
            landed_hard: unpack_unit(small[7]),
            crouch: f32::from(small[8] >> 4) / 15.0,
            swing: small[9],
            phase: f32::from(small[8] & 15) / 15.0 * PHASE_RANGE,
            ..Self::default()
        }
    }
}

impl Body for RobotBody {
    const KIND: Kind = Kind::Robot;

    const MAX_LEN: usize = 55;

    /// Adds the body to `out`, with its position packed for `frame`.
    fn encode(&self, frame: &Frame, out: &mut Vec<u8>) {
        let yaw = (self.yaw + PI).rem_euclid(TAU) - PI;
        let stride = (self.stride_distance.rem_euclid(STRIDE_WRAP) * 100.0) as u16;
        put_position(out, frame, self.position);
        put_vec3(out, self.velocity);
        out.extend_from_slice(&pack_i16(yaw, PI).to_le_bytes());
        out.push(self.mode);
        out.extend_from_slice(&pack_u16(self.speed, SPEED_RANGE).to_le_bytes());
        out.extend_from_slice(&pack_i16(self.vertical_speed, CLIMB_RANGE).to_le_bytes());
        out.extend_from_slice(&stride.to_le_bytes());
        out.extend_from_slice(&self.small());
    }

    /// Returns the body [`Body::encode`] wrote for `frame`, or nothing when `bytes` has the
    /// wrong length or holds a number that is not finite.
    fn decode(frame: &Frame, bytes: &[u8]) -> Option<Self> {
        let mut reader = Reader(bytes);
        let position = reader.position(frame)?;
        let velocity = reader.vec3()?;
        let yaw = unpack_i16(i16::from_le_bytes(reader.take()?), PI);
        let [mode] = reader.take()?;
        let speed = unpack_u16(u16::from_le_bytes(reader.take()?), SPEED_RANGE);
        let vertical_speed = unpack_i16(i16::from_le_bytes(reader.take()?), CLIMB_RANGE);
        let stride_distance = f32::from(u16::from_le_bytes(reader.take()?)) / 100.0;
        let body = Self {
            position,
            velocity,
            yaw,
            mode,
            speed,
            vertical_speed,
            stride_distance,
            ..Self::with_small(reader.take()?)
        };
        reader.end(body)
    }
}
