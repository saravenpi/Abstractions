//! The body of a ship record: where a player's ship is and what a watcher sees of its state.
//!
//! 37 bytes on a surface, 49 in the other frames.

use glam::{DVec3, Quat, Vec3};

use super::Frame;
use super::bytes::{Reader, put_position, put_vec3};
use crate::pack::{pack_i8, pack_rotation, pack_unit, unpack_i8, unpack_rotation, unpack_unit};

/// Value of [`ShipBody::warp`] while the warp drive is off.
pub const WARP_OFF: u8 = 0;

/// Value of [`ShipBody::warp`] while the warp drive spools.
pub const WARP_SPOOLING: u8 = 1;

/// Value of [`ShipBody::warp`] while the ship moves at warp speed.
pub const WARP_ACTIVE: u8 = 2;

/// What a remote hull needs to look right. Values out of their range are clamped when encoded.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct ShipBody {
    /// Position of the hull centre in the frame of the record, in metres. Kept to the
    /// millimetre on a surface.
    pub position: DVec3,
    /// Velocity in the frame of the record. Used to interpolate between two samples.
    pub velocity: Vec3,
    /// Orientation: turns hull axes into frame axes. Kept within 0.005 rad.
    pub rotation: Quat,
    /// Main engine lever, from -1 to 1: it sets the engine glow.
    pub throttle: f32,
    /// Extension of the landing gear, from 0 (folded) to 1 (down): it sets the length of the legs.
    pub gear: f32,
    /// Damage from hard landings, from 0 to 1.
    pub damage: f32,
    /// Whether the ship rests on its gear or floats at rest.
    pub landed: bool,
    /// State of the warp drive: [`WARP_OFF`], [`WARP_SPOOLING`] or [`WARP_ACTIVE`]. Other
    /// values are kept as they are, for later versions.
    pub warp: u8,
    /// How far the drive is, from 0 to 1: the share of the spool time done while it spools, the
    /// share of its top speed while it runs.
    pub warp_level: f32,
    /// Tilt of the pilot's stick, each axis from -1 to 1: pitch up, then roll left.
    pub stick: [f32; 2],
    /// Boost the pilot holds, from 0 to 1.
    pub boost: f32,
}

impl ShipBody {
    /// Adds the body to `out`, with its position packed for `frame`.
    pub fn encode(&self, frame: &Frame, out: &mut Vec<u8>) {
        put_position(out, frame, self.position);
        put_vec3(out, self.velocity);
        out.extend_from_slice(&pack_rotation(self.rotation).to_le_bytes());
        out.extend_from_slice(&[
            pack_i8(self.throttle, 1.0) as u8,
            pack_unit(self.gear),
            pack_unit(self.damage),
            u8::from(self.landed),
            self.warp,
            pack_unit(self.warp_level),
            pack_i8(self.stick[0], 1.0) as u8,
            pack_i8(self.stick[1], 1.0) as u8,
            pack_unit(self.boost),
        ]);
    }

    /// Returns the body [`ShipBody::encode`] wrote for `frame`, or nothing when `bytes` has the
    /// wrong length or holds a number that is not finite.
    pub fn decode(frame: &Frame, bytes: &[u8]) -> Option<Self> {
        let mut reader = Reader(bytes);
        let position = reader.position(frame)?;
        let velocity = reader.vec3()?;
        let rotation = unpack_rotation(u32::from_le_bytes(reader.take()?));
        let [throttle, gear, damage, state, warp, level, x, y, boost] = reader.take()?;
        reader.end(Self {
            position,
            velocity,
            rotation,
            throttle: unpack_i8(throttle as i8, 1.0),
            gear: unpack_unit(gear),
            damage: unpack_unit(damage),
            landed: state & 1 != 0,
            warp,
            warp_level: unpack_unit(level),
            stick: [unpack_i8(x as i8, 1.0), unpack_i8(y as i8, 1.0)],
            boost: unpack_unit(boost),
        })
    }
}
