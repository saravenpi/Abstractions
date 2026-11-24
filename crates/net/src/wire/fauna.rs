//! Compact batch encoding for authoritative creature poses sent on a planet surface.

use glam::{DVec3, Vec3};

use super::bytes::Reader;

/// Most bytes a [`crate::wire::Kind::Fauna`] record body may hold inside a datagram.
pub const FAUNA_BODY_MAX: usize = 960;

/// Byte length of the batch header: three `f64` origin coordinates and one `u8` pose count.
pub const FAUNA_HEADER_LEN: usize = 25;

/// Byte length of one encoded [`FaunaPose`].
pub const FAUNA_POSE_LEN: usize = 33;

/// Maximum creature poses packed into a single [`FAUNA_BODY_MAX`] batch.
pub const FAUNA_BATCH_MAX: usize = (FAUNA_BODY_MAX - FAUNA_HEADER_LEN) / FAUNA_POSE_LEN;

/// Velocity quantization scale in fixed-point centimetres per second.
const VEL_SCALE: f32 = 100.0;

/// Reserve bytes allowed for record framing when sizing a fauna batch inside a datagram.
const DATAGRAM_OVERHEAD: usize = 64;

/// Authoritative snapshot of one drawn living creature on a planet surface.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct FaunaPose {
    /// Herd identity from `Lead::key().0`.
    pub herd: u64,
    /// Slot in the herd or visitor group from `Lead::key().1`.
    pub slot: u8,
    /// Species index on the planet.
    pub species: u16,
    /// Planet-frame position in metres from the planet centre.
    pub pos: DVec3,
    /// Planet-frame linear velocity in metres per second.
    pub vel: Vec3,
    /// Planet-frame unit forward direction of the creature's body.
    pub heading: Vec3,
    /// Encoded stance code (`0..=4`).
    pub stance: u8,
    /// Mood alarm / excitement in `0.0..=1.0`.
    pub excite: f32,
}

/// Decoded batch of creature snapshots relative to a shared planet-frame origin.
#[derive(Debug, Clone, PartialEq)]
pub struct FaunaBatch {
    /// Batch reference position in planet coordinates (metres).
    pub origin: DVec3,
    /// Decoded creature poses with reconstructed planet-frame positions.
    pub poses: Vec<FaunaPose>,
}

/// Returns how many creature poses fit inside a datagram of `max_datagram` bytes.
pub fn max_fauna_poses(max_datagram: usize) -> usize {
    let body_budget = max_datagram
        .saturating_sub(DATAGRAM_OVERHEAD)
        .min(FAUNA_BODY_MAX);
    body_budget
        .saturating_sub(FAUNA_HEADER_LEN)
        .saturating_div(FAUNA_POSE_LEN)
        .min(FAUNA_BATCH_MAX)
}

/// Appends up to [`FAUNA_BATCH_MAX`] finite creature poses relative to `origin` into `out`.
pub fn encode_fauna(origin: DVec3, poses: &[FaunaPose], out: &mut Vec<u8>) {
    if !origin.is_finite() {
        return;
    }
    let count = poses
        .iter()
        .take(FAUNA_BATCH_MAX)
        .filter(|p| p.pos.is_finite() && p.vel.is_finite() && p.heading.is_finite())
        .count();
    for axis in origin.to_array() {
        out.extend_from_slice(&axis.to_le_bytes());
    }
    out.push(count as u8);
    let mut written = 0;
    for pose in poses {
        if written == count {
            break;
        }
        if pose.pos.is_finite() && pose.vel.is_finite() && pose.heading.is_finite() {
            put_pose(origin, pose, out);
            written += 1;
        }
    }
}

/// Encodes one creature pose relative to `origin` into `out`.
fn put_pose(origin: DVec3, pose: &FaunaPose, out: &mut Vec<u8>) {
    out.extend_from_slice(&pose.herd.to_le_bytes());
    out.push(pose.slot);
    out.extend_from_slice(&pose.species.to_le_bytes());
    let rel = (pose.pos - origin).as_vec3();
    for axis in rel.to_array() {
        out.extend_from_slice(&axis.to_le_bytes());
    }
    for axis in pose.vel.to_array() {
        let cm = (axis * VEL_SCALE)
            .round()
            .clamp(f32::from(i16::MIN), f32::from(i16::MAX)) as i16;
        out.extend_from_slice(&cm.to_le_bytes());
    }
    let dir = pose.heading.normalize_or(Vec3::NEG_Z);
    for axis in dir.to_array() {
        let q = (axis * 127.0).round().clamp(-127.0, 127.0) as i8;
        out.push(q as u8);
    }
    let excite = (pose.excite.clamp(0.0, 1.0) * 31.0).round() as u8;
    let flags = (pose.stance & 0x07) | (excite << 3);
    out.push(flags);
}

/// Decodes a [`FaunaBatch`] written by [`encode_fauna`], returning `None` if malformed.
pub fn decode_fauna(bytes: &[u8]) -> Option<FaunaBatch> {
    if bytes.len() < FAUNA_HEADER_LEN || bytes.len() > FAUNA_BODY_MAX {
        return None;
    }
    let mut reader = Reader(bytes);
    let ox = f64::from_le_bytes(reader.take::<8>()?);
    let oy = f64::from_le_bytes(reader.take::<8>()?);
    let oz = f64::from_le_bytes(reader.take::<8>()?);
    let origin = DVec3::new(ox, oy, oz);
    if !origin.is_finite() {
        return None;
    }
    let [count_byte] = reader.take::<1>()?;
    let count = usize::from(count_byte);
    let expected = FAUNA_HEADER_LEN + count * FAUNA_POSE_LEN;
    if count > FAUNA_BATCH_MAX || bytes.len() != expected {
        return None;
    }
    let mut poses = Vec::with_capacity(count);
    for _ in 0..count {
        poses.push(take_pose(&mut reader, origin)?);
    }
    reader.end(FaunaBatch { origin, poses })
}

/// Reads one encoded creature pose from `reader` and reconstructs its planet position.
fn take_pose(reader: &mut Reader<'_>, origin: DVec3) -> Option<FaunaPose> {
    let herd = u64::from_le_bytes(reader.take::<8>()?);
    let [slot] = reader.take::<1>()?;
    let species = u16::from_le_bytes(reader.take::<2>()?);
    let rel = reader.vec3()?;
    let pos = origin + rel.as_dvec3();
    if !pos.is_finite() {
        return None;
    }
    let vx = i16::from_le_bytes(reader.take::<2>()?);
    let vy = i16::from_le_bytes(reader.take::<2>()?);
    let vz = i16::from_le_bytes(reader.take::<2>()?);
    let vel = Vec3::new(f32::from(vx), f32::from(vy), f32::from(vz)) / VEL_SCALE;
    let [hx, hy, hz] = reader.take::<3>()?;
    let heading = Vec3::new(
        f32::from(hx as i8),
        f32::from(hy as i8),
        f32::from(hz as i8),
    )
    .normalize_or(Vec3::NEG_Z);
    let [flags] = reader.take::<1>()?;
    let stance = flags & 0x07;
    let excite = f32::from(flags >> 3) / 31.0;
    Some(FaunaPose {
        herd,
        slot,
        species,
        pos,
        vel,
        heading,
        stance,
        excite,
    })
}
