//! Keys: where the grip is and which way the tool points at one instant of a strike, in the torso
//! frame, with the turn and bend of the chest that go with it. Every act blends between keys.

use bevy::prelude::*;

use super::dir::Dir;

/// One instant of a strike, in the frame of the torso: +X right, +Y up, -Z ahead.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Key {
    /// Where the leading hand holds the grip, in metres.
    pub grip: Vec3,
    /// Unit direction from the grip to the head.
    pub axis: Vec3,
    /// Turn of the chest to the left over the hips, in radians.
    pub twist: f32,
    /// Forward bend of the chest, in radians.
    pub spine: f32,
}

impl Key {
    /// Returns the key `k` of the way from this one to `other`, 0 to 1, eased in and out.
    pub fn toward(&self, other: &Key, k: f32) -> Key {
        let k = k.clamp(0.0, 1.0);
        let k = k * k * (3.0 - 2.0 * k);
        Key {
            grip: self.grip.lerp(other.grip, k),
            axis: self.axis.lerp(other.axis, k).normalize_or(other.axis),
            twist: self.twist + (other.twist - self.twist) * k,
            spine: self.spine + (other.spine - self.spine) * k,
        }
    }
}

/// Returns the key of a tool carried in the right hand, low beside the hip, pointing ahead and up.
pub fn carry() -> Key {
    Key {
        grip: Vec3::new(0.26, 0.02, -0.18),
        axis: Vec3::new(0.0, 0.7, -0.7).normalize(),
        twist: 0.0,
        spine: 0.0,
    }
}

/// Returns the key of a guard raised on the side `dir`: the shaft across the blow that would come
/// from there.
pub fn guard(dir: Dir) -> Key {
    let (grip, axis) = match dir {
        Dir::Right => (Vec3::new(0.24, 0.3, -0.3), Vec3::new(0.1, 1.0, -0.1)),
        Dir::Left => (Vec3::new(-0.08, 0.3, -0.32), Vec3::new(-0.1, 1.0, -0.1)),
        Dir::Up => (Vec3::new(0.2, 0.5, -0.28), Vec3::new(-1.0, 0.15, 0.0)),
        Dir::Down => (Vec3::new(0.18, 0.05, -0.32), Vec3::new(-0.6, -0.8, 0.0)),
    };
    Key {
        grip,
        axis: axis.normalize(),
        twist: -0.15 * dir.side(),
        spine: 0.08,
    }
}

/// Returns the key of a spear drawn back over the shoulder for a throw, pointing ahead.
pub fn drawn() -> Key {
    Key {
        grip: Vec3::new(0.28, 0.42, 0.22),
        axis: Vec3::new(-0.05, 0.12, -1.0).normalize(),
        twist: -0.4,
        spine: -0.12,
    }
}
