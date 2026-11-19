//! Limb solvers: reachable targets are met, knees bend toward their pole, far targets get a straight limb.

use bevy::math::Vec3;

use super::ik::{fabrik, three_bone, two_bone};
use crate::universe::hash::Rng;

/// Returns a seeded point in a box `size` wide around the origin.
fn point(rng: &mut Rng, size: f32) -> Vec3 {
    let mut axis = || rng.range(-1.0, 1.0) as f32 * size;
    Vec3::new(axis(), axis(), axis())
}

/// Every reachable target is met within a millimetre with the knee on the side of its pole,
/// and every target out of reach gets a straight limb pointing at it.
#[test]
fn two_bone_meets_targets_and_bends_to_its_pole() {
    let mut rng = Rng::new(9);
    let (a, b) = (0.6, 0.5);
    for _ in 0..2000 {
        let (root, target, pole) = (
            point(&mut rng, 1.0),
            point(&mut rng, 1.0),
            point(&mut rng, 2.0),
        );
        let [knee, end] = two_bone(root, target, (a, b), pole);
        assert!((knee.distance(root) - a).abs() < 1e-4 && (end.distance(knee) - b).abs() < 1e-4);
        let dist = target.distance(root);
        let dir = (target - root) / dist;
        let side = (pole - root) - dir * (pole - root).dot(dir);
        if dist < a + b - 1e-3 && dist > (a - b) + 1e-3 {
            assert!(
                end.distance(target) < 1e-3,
                "missed by {}",
                end.distance(target)
            );
            if side.length() > 1e-2 {
                assert!((knee - root).dot(side) > 0.0, "knee on the wrong side");
            }
        } else if dist >= a + b {
            assert!(
                (knee - root).normalize().dot(dir) > 0.9999
                    && (end - knee).normalize().dot(dir) > 0.9999
            );
        }
    }
}

/// A three-bone leg holds its last bone along the given direction and still reaches.
#[test]
fn three_bone_holds_its_last_bone() {
    let (root, pole, last) = (
        Vec3::new(0.0, 1.0, 0.0),
        Vec3::new(0.0, 1.0, 1.0),
        Vec3::new(0.0, -0.8, -0.6),
    );
    let target = Vec3::new(0.1, 0.0, -0.3);
    let [knee, ankle, end] = three_bone(root, target, [0.5, 0.5, 0.2], last, pole);
    assert!(end.distance(target) < 1e-3);
    assert!((end - ankle).normalize().dot(last.normalize()) > 0.9999);
    assert!(knee.z > root.z.min(ankle.z));
}

/// FABRIK reaches a reachable target in three passes and keeps every segment's length.
#[test]
fn fabrik_reaches_and_keeps_lengths() {
    let lengths = [0.3, 0.3, 0.3, 0.3];
    let mut points: Vec<Vec3> = (0..5).map(|i| Vec3::Y * 0.3 * i as f32).collect();
    let target = Vec3::new(0.6, 0.6, 0.2);
    fabrik(&mut points, &lengths, target, 3);
    assert!(
        points[4].distance(target) < 0.01,
        "missed by {}",
        points[4].distance(target)
    );
    assert_eq!(points[0], Vec3::ZERO);
    for (pair, length) in points.windows(2).zip(lengths) {
        assert!((pair[0].distance(pair[1]) - length).abs() < 1e-4);
    }
    fabrik(&mut points, &lengths, Vec3::X * 5.0, 2);
    assert!(points[4].distance(Vec3::X * 1.2) < 1e-3);
}
