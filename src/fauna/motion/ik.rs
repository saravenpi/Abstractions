//! Limb solvers: analytic two-bone and three-bone legs with a pole, and FABRIK for chains.
//!
//! FABRIK after Aristidou and Lasenby, as in the Vulkan tutorial
//! (https://docs.vulkan.org/tutorial/latest/Advanced_glTF/Procedural_Animation_IK/03_fabrik.html):
//! a backward pass from the target, then a forward pass from the fixed root, a few times over.

use bevy::math::Vec3;

/// Smallest length used as a direction, in metres.
const TINY: f32 = 1e-6;

/// Returns the unit vector of `v`, or `fallback` when `v` is too short to have one.
pub fn unit(v: Vec3, fallback: Vec3) -> Vec3 {
    let length = v.length();
    match length > TINY {
        true => v / length,
        false => fallback,
    }
}

/// Returns the knee and the end of a two-bone limb from `root` with bones `a` and `b` long that
/// reaches for `target`, its knee bent toward `pole`.
///
/// A target out of reach gives a straight limb pointing at it. One too close gives a folded limb.
pub fn two_bone(root: Vec3, target: Vec3, (a, b): (f32, f32), pole: Vec3) -> [Vec3; 2] {
    let reach = target - root;
    let dir = unit(reach, Vec3::NEG_Y);
    let dist = reach.length().clamp((a - b).abs() + 1e-5, a + b);
    if dist >= a + b {
        return [root + dir * a, root + dir * (a + b)];
    }
    let along = (a * a - b * b + dist * dist) / (2.0 * dist);
    let rise = (a * a - along * along).max(0.0).sqrt();
    let side = pole - root;
    let bend = unit(side - dir * side.dot(dir), dir.any_orthonormal_vector());
    [root + dir * along + bend * rise, root + dir * dist]
}

/// Returns the three joints after the root of a three-bone limb reaching for `target`, its last
/// bone held along `last`, its knee bent toward `pole`.
pub fn three_bone(
    root: Vec3,
    target: Vec3,
    lengths: [f32; 3],
    last: Vec3,
    pole: Vec3,
) -> [Vec3; 3] {
    let last = unit(last, Vec3::NEG_Y);
    let ankle = target - last * lengths[2];
    let [knee, held] = two_bone(root, ankle, (lengths[0], lengths[1]), pole);
    [knee, held, held + last * lengths[2]]
}

/// Moves `points`, the joints of a chain whose segments are `lengths` long, so that its tip
/// reaches for `target` while its first joint stays where it is, in `passes` FABRIK passes.
pub fn fabrik(points: &mut [Vec3], lengths: &[f32], target: Vec3, passes: usize) {
    let Some(&root) = points.first() else {
        return;
    };
    let count = lengths.len().min(points.len().saturating_sub(1));
    for _ in 0..passes {
        points[count] = target;
        for i in (0..count).rev() {
            let back = unit(points[i] - points[i + 1], Vec3::NEG_Y);
            points[i] = points[i + 1] + back * lengths[i];
        }
        points[0] = root;
        for i in 0..count {
            let ahead = unit(points[i + 1] - points[i], Vec3::NEG_Y);
            points[i + 1] = points[i] + ahead * lengths[i];
        }
    }
}
