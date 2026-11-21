//! Tests that the body is one piece: every shape is closed and faces outward, no triangle is
//! degenerate, the shapes of a segment hold together, and every segment turns inside its parent
//! at every extreme of every joint.

use bevy::prelude::*;

use super::build;
use crate::avatar::pose::{CORE_LIMITS, CORES, LIMB_LIMITS, LIMBS, Pose};
use crate::avatar::rig::{Seg, globals};
use crate::skin::Detail;
use crate::skin::rig::{Tri, floating, inside, open_edges, pieces, volume};

/// How far round a joint its housing must reach, in meters: the tolerance of the attachment.
const SPARE: f32 = 0.005;

/// Every shape of every segment is closed and faces outward, up close and from afar, and no
/// triangle is degenerate.
#[test]
fn every_shape_is_closed_and_faces_outward() {
    for (seg, detail) in Seg::ALL
        .into_iter()
        .flat_map(|s| [(s, Detail::Near), (s, Detail::Far)])
    {
        let soup = build(seg, detail);
        for tri in soup.tris() {
            let area = (tri[1] - tri[0]).cross(tri[2] - tri[0]).length();
            assert!(area > 1e-8, "{seg:?} {detail:?}: degenerate {tri:?}");
        }
        for piece in pieces(&soup) {
            assert_eq!(
                open_edges(&piece),
                0,
                "{seg:?} {detail:?}: open at {:?}",
                piece[0]
            );
            assert!(
                volume(&piece) > 1e-7,
                "{seg:?} {detail:?}: inside out at {:?}",
                piece[0]
            );
        }
    }
}

/// Up close, the shapes of each segment are one piece: each is sunk into another, none floats.
#[test]
fn no_shape_of_a_segment_floats() {
    for seg in Seg::ALL {
        let all = pieces(&build(seg, Detail::Near));
        let lost: Vec<Vec3> = floating(&all).into_iter().map(|i| all[i][0][0]).collect();
        assert!(lost.is_empty(), "{seg:?}: shapes float near {lost:?}");
    }
}

/// Returns the rest pose with the joint `joint` alone, on both sides for a limb, at its value
/// in `whole`. The trunk joints count first, then the joints of a side.
fn alone(whole: &Pose, joint: usize) -> Pose {
    let mut pose = Pose::default();
    match joint.checked_sub(CORES) {
        None => pose.core[joint] = whole.core[joint],
        Some(limb) => {
            for (mine, theirs) in pose.sides.iter_mut().zip(whole.sides) {
                mine[limb] = theirs[limb];
            }
        }
    }
    pose
}

/// Returns the poses that put each joint in turn at its lowest and at its highest, then all at
/// their lowest and all at their highest, and the rest pose.
fn extremes() -> Vec<Pose> {
    let mut all = vec![Pose::default()];
    for high in [false, true] {
        let pick = |(low, top): (f32, f32)| if high { top } else { low };
        let whole = Pose {
            core: CORE_LIMITS.map(pick),
            sides: [LIMB_LIMITS.map(pick); 2],
            ..Pose::default()
        };
        all.push(whole);
        all.extend((0..CORES + LIMBS).map(|joint| alone(&whole, joint)));
    }
    all
}

/// Returns whether one shape of `shapes` holds `point` with [`SPARE`] to spare on every side.
fn holds(shapes: &[Vec<Tri>], point: Vec3) -> bool {
    let around = [Vec3::X, Vec3::Y, Vec3::Z]
        .into_iter()
        .flat_map(|axis| [point + axis * SPARE, point - axis * SPARE]);
    let around: Vec<Vec3> = around.collect();
    shapes
        .iter()
        .any(|shape| around.iter().all(|p| inside(shape, *p)))
}

/// Returns whether a shape of the segment `seg` holds `point`, given in its frame, with
/// [`SPARE`] to spare on every side.
pub fn held(seg: Seg, point: Vec3) -> bool {
    holds(&pieces(&build(seg, Detail::Near)), point)
}

/// At the rest pose and at every extreme of every joint, each joint lies inside a shape of the
/// segment that turns there and inside a shape of its parent, with [`SPARE`] to spare: a housing
/// holds every limb, so no gap can open.
#[test]
fn every_joint_is_held_at_every_extreme() {
    let shapes = Seg::ALL.map(|seg| pieces(&build(seg, Detail::Near)));
    for pose in extremes() {
        let frames = globals(&pose);
        for (seg, parent) in Seg::ALL.into_iter().filter_map(|s| Some((s, s.parent()?))) {
            let joint = frames[seg as usize].translation;
            for part in [seg, parent] {
                let local = frames[part as usize].compute_affine().inverse();
                let held = holds(&shapes[part as usize], local.transform_point3(joint));
                assert!(held, "{part:?} does not hold the joint of {seg:?}");
            }
        }
    }
}
