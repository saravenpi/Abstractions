//! The wingsuit: how it unfolds, and the fabric stretched between the arms and the body and between the legs.

use bevy::prelude::*;

use super::rig::{SEGS, Seg};
use super::tune::{CHEST, FLUTTER, FLUTTER_SPEED, HIPS, UNFOLD_SECONDS};

/// A panel of fabric.
#[derive(Component, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Panel {
    /// Between the left arm and the body.
    ArmL,
    /// Between the right arm and the body.
    ArmR,
    /// Between the legs.
    Legs,
}

impl Panel {
    /// Every panel.
    pub const ALL: [Panel; 3] = [Panel::ArmL, Panel::ArmR, Panel::Legs];
}

/// Number of corners on the rim of a panel, and of triangles in it.
pub const RIM: usize = 5;

/// Returns how far the wingsuit is unfolded after `dt` seconds on the way from `now` to `want`.
///
/// It moves at a steady rate, never past `want`, and covers the whole way in [`UNFOLD_SECONDS`].
pub fn unfold_step(now: f32, want: f32, dt: f32) -> f32 {
    let want = if want.is_finite() {
        want.clamp(0.0, 1.0)
    } else {
        0.0
    };
    let step = dt.max(0.0) / UNFOLD_SECONDS;
    now + (want - now).clamp(-step, step)
}

/// Returns how far the fabric is stretched for an unfold from 0 to 1: eased at both ends, monotone.
pub fn stretch(unfold: f32) -> f32 {
    let t = unfold.clamp(0.0, 1.0);
    t * t * (3.0 - 2.0 * t)
}

/// Returns how far the middle of the fabric bulges at `airspeed`, for a `wave` from -1 to 1, in meters.
pub fn flutter(airspeed: f32, wave: f32, open: f32) -> f32 {
    let wind = (airspeed.abs() / FLUTTER_SPEED).clamp(0.0, 1.0);
    (FLUTTER.0 + (FLUTTER.1 - FLUTTER.0) * wind) * wave * open
}

/// Returns the rim of an arm panel on the side of `sign`: shoulder, elbow, wrist, hip and flank.
///
/// Folded, the arm corners lie on the flank and the hip, so the fabric grows out of the body.
fn arm_rim(joints: &[Transform; SEGS], sign: f32, open: f32) -> [Vec3; RIM] {
    let side = usize::from(sign > 0.0);
    let at = |seg: Seg| joints[seg as usize + side].translation;
    let flank = joints[Seg::Torso as usize] * Vec3::new(sign * CHEST.1[0] * 0.9, 0.12, 0.0);
    let hip = joints[Seg::Pelvis as usize] * Vec3::new(sign * HIPS[0] * 0.95, 0.0, 0.0);
    [
        flank.lerp(at(Seg::UpperArmL), open),
        flank.lerp(at(Seg::ForearmL), open),
        hip.lerp(at(Seg::HandL), open),
        hip,
        flank,
    ]
}

/// Returns the rim of the leg panel: crotch, left knee, left ankle, right ankle, right knee.
///
/// Folded, the corners of both legs meet halfway between them.
fn leg_rim(joints: &[Transform; SEGS], open: f32) -> [Vec3; RIM] {
    let at = |seg: Seg| joints[seg as usize].translation;
    let pull = |a: Seg, b: Seg| ((at(a) + at(b)) * 0.5).lerp(at(a), open);
    [
        joints[Seg::Pelvis as usize] * Vec3::new(0.0, -HIPS[1], 0.0),
        pull(Seg::ShinL, Seg::ShinR),
        pull(Seg::BootL, Seg::BootR),
        pull(Seg::BootR, Seg::BootL),
        pull(Seg::ShinR, Seg::ShinL),
    ]
}

/// Returns the rim of `panel` in the frame of the player body, for segments at `joints` and a
/// fabric stretched by `open` from 0 to 1.
pub fn rim(panel: Panel, joints: &[Transform; SEGS], open: f32) -> [Vec3; RIM] {
    match panel {
        Panel::ArmL => arm_rim(joints, -1.0, open),
        Panel::ArmR => arm_rim(joints, 1.0, open),
        Panel::Legs => leg_rim(joints, open),
    }
}

/// Returns the triangles of a panel: a fan from its middle, pushed out of its plane by `bulge` meters.
pub fn fan(rim: &[Vec3; RIM], bulge: f32) -> [[Vec3; 3]; RIM] {
    let middle = rim.iter().sum::<Vec3>() / RIM as f32;
    let normal = (rim[1] - rim[0]).cross(rim[3] - rim[0]).normalize_or_zero();
    let hub = middle + normal * bulge;
    std::array::from_fn(|i| [hub, rim[i], rim[(i + 1) % RIM]])
}
