//! The rest pose of a body: where every part sits in the body frame when nothing moves.

use bevy::math::{Quat, Vec3};

use super::{Part, ShapeKind};

impl ShapeKind {
    /// Returns whether the shape hangs along -Y of its frame from its socket, like a limb.
    ///
    /// Tapers, spikes and fans hang from the socket. Blocks and balls are centred on it.
    pub fn hangs(self) -> bool {
        matches!(self, Self::Taper | Self::Spike | Self::Fan)
    }
}

/// Returns the position and rotation of part `index` in the body frame at rest.
pub fn frame(parts: &[Part], index: usize) -> (Vec3, Quat) {
    let part = &parts[index];
    let (at, turn) = match part.parent {
        Some(parent) => frame(parts, usize::from(parent)),
        None => (Vec3::ZERO, Quat::IDENTITY),
    };
    (at + turn * part.socket, turn * part.rest)
}

/// Returns the position and rotation of every part in the body frame at rest, in part order.
pub fn frames(parts: &[Part]) -> Vec<(Vec3, Quat)> {
    let mut out: Vec<(Vec3, Quat)> = Vec::with_capacity(parts.len());
    for part in parts {
        let (at, turn) = part
            .parent
            .map_or((Vec3::ZERO, Quat::IDENTITY), |p| out[usize::from(p)]);
        out.push((at + turn * part.socket, turn * part.rest));
    }
    out
}

/// Returns the two far points of a part along the body axis, in the body frame at rest.
fn ends(part: &Part, (at, turn): (Vec3, Quat)) -> [Vec3; 2] {
    match part.shape.hangs() {
        true => [at, at + turn * Vec3::new(0.0, -part.size.y, 0.0)],
        false => {
            let half = turn * Vec3::new(0.0, 0.0, part.size.z * 0.5);
            [at - half, at + half]
        }
    }
}

/// Returns the nose to tail length of a body at rest: its extent along Z.
pub fn length(parts: &[Part]) -> f32 {
    let zs = frames(parts)
        .into_iter()
        .zip(parts)
        .flat_map(|(frame, part)| ends(part, frame))
        .map(|p| p.z);
    let (low, high) = zs.fold((f32::MAX, f32::MIN), |(l, h), z| (l.min(z), h.max(z)));
    (high - low).max(0.0)
}

/// Returns the mirror image across the body's YZ plane of a part on the right, as its twin on the left.
pub fn mirror(part: Part) -> Part {
    Part {
        socket: part.socket * Vec3::new(-1.0, 1.0, 1.0),
        rest: Quat::from_xyzw(part.rest.x, -part.rest.y, -part.rest.z, part.rest.w),
        side: -part.side,
        ..part
    }
}

/// Returns the rotation of a hanging part that points its -Y axis along `dir`, both in the parent frame.
pub fn aim(dir: Vec3) -> Quat {
    Quat::from_rotation_arc(Vec3::NEG_Y, dir.normalize_or(Vec3::NEG_Y))
}
