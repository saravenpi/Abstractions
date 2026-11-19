//! Legs: a pair of two or three bone legs, posed at rest with the foot on the ground under the hip.

use bevy::math::{Quat, Vec3};

use super::frame::{aim, frame};
use super::kit::Kit;
use super::piece::piece;
use super::{Knee, Leg, Part, PartKind, ShapeKind};

/// Hip to rest foot distance over the leg length, inside the 0.6 to 0.95 the motion model can reach.
const REACH: f32 = 0.8;

/// One pair of legs, described by its right leg.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct LegSpec {
    /// Hip socket of the right leg in the frame of the parent part.
    pub hip: Vec3,
    /// Bone lengths: thigh, shin, and foot or 0 for a two bone leg.
    pub bones: [f32; 3],
    /// Thigh radius. The shin and the foot are thinner.
    pub radius: f32,
    /// Which way the knee bends.
    pub knee: Knee,
    /// Offset of the right foot at rest from the ground straight under the hip, in the body frame.
    pub out: Vec3,
    /// Order from front to back, 0 for the front pair.
    pub rank: u8,
    /// Shapes of the thigh and the shin: tapers, or spikes for thin insect legs.
    pub shapes: [ShapeKind; 2],
}

/// Returns the hip height of a leg of length `leg` whose foot rests `splay` metres out from under the hip.
pub fn stance(leg: f32, splay: f32) -> f32 {
    let reach = REACH * leg;
    (reach * reach - splay * splay).max(0.0).sqrt()
}

/// Returns the knee of a two bone limb from `hip` to `target` with bone lengths `bones`, bent toward `pole`.
fn bend(hip: Vec3, target: Vec3, (a, b): (f32, f32), pole: Vec3) -> Vec3 {
    let d = target - hip;
    let dist = d.length().clamp(1e-4, (a + b) * 0.999);
    let dir = d.normalize_or(Vec3::NEG_Y);
    let side = pole
        .reject_from(dir)
        .normalize_or(dir.any_orthonormal_vector());
    let cos = ((a * a + dist * dist - b * b) / (2.0 * a * dist)).clamp(-1.0, 1.0);
    hip + (dir * cos + side * (1.0 - cos * cos).sqrt()) * a
}

/// Returns the direction the knee of `spec` points to.
fn pole(spec: &LegSpec) -> Vec3 {
    match spec.knee {
        Knee::Forward => Vec3::NEG_Z,
        Knee::Back => Vec3::Z,
        Knee::Out => Vec3::Y + spec.out.normalize_or_zero(),
    }
}

/// Returns the rest rotations of the thigh, shin and foot of the right leg, and its foot rest point.
fn pose(kit: &Kit, parent: u8, spec: &LegSpec) -> ([Quat; 3], Vec3) {
    let (at, turn) = frame(&kit.body.parts, usize::from(parent));
    let hip = at + turn * spec.hip;
    let rest = Vec3::new(hip.x, 0.0, hip.z) + spec.out;
    let [a, b, c] = spec.bones;
    let target = rest + Vec3::Y * c;
    let knee = bend(hip, target, (a, b), pole(spec));
    let thigh = aim(turn.inverse() * (knee - hip));
    let shin = aim((turn * thigh).inverse() * (target - knee));
    let foot = aim((turn * thigh * shin).inverse() * Vec3::NEG_Y);
    ([thigh, shin, foot], rest)
}

/// Returns the bones of the right leg of `spec` with the rest rotations `turns`, hip first.
fn bones(spec: &LegSpec, turns: [Quat; 3]) -> Vec<Part> {
    let [a, b, c] = spec.bones;
    let r = spec.radius;
    let bone = |kind, shape, (length, width): (f32, f32)| {
        piece(kind, shape, Vec3::new(width, length, width))
    };
    let mut out = vec![
        bone(PartKind::Thigh, spec.shapes[0], (a, 2.0 * r))
            .at(spec.hip)
            .of(0, 0.75),
        bone(PartKind::Shin, spec.shapes[1], (b, 1.5 * r))
            .at(Vec3::NEG_Y * a)
            .of(0, 0.7),
    ];
    if c > 0.0 {
        out.push(
            bone(PartKind::Foot, ShapeKind::Taper, (c, 1.4 * r))
                .at(Vec3::NEG_Y * b)
                .of(0, 0.8),
        );
    }
    out.iter()
        .zip(turns)
        .map(|(part, rest)| part.turned(rest))
        .collect()
}

/// Adds the pair of legs `spec` under `parent`, left leg first, and lists both legs.
pub fn legs(kit: &mut Kit, parent: u8, spec: &LegSpec) {
    let (turns, rest) = pose(kit, parent, spec);
    let mut above = [parent; 2];
    let mut pairs = Vec::with_capacity(3);
    for part in bones(spec, turns) {
        above = kit.pair(above, part);
        pairs.push(above);
    }
    let lengths: Vec<f32> = spec.bones.into_iter().filter(|&l| l > 0.0).collect();
    for (index, side) in [(0, -1i8), (1, 1)] {
        kit.body.legs.push(Leg {
            bones: pairs.iter().map(|pair| pair[index]).collect(),
            lengths: lengths.clone(),
            rest: rest * Vec3::new(f32::from(side), 1.0, 1.0),
            knee: spec.knee,
            side,
            rank: spec.rank,
        });
    }
}
