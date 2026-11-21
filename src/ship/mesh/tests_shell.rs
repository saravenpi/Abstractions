//! Shell tests: seen from the eye of the pilot the cabin has no hole and shows no surface from
//! behind, and the canopy holds to the spine shut and open.

use bevy::prelude::*;

use super::bars::frame;
use super::dome::{arch, glass, spine};
use super::fittings::{accents, furniture, plates};
use super::frame::{CANOPY, GLAZED};
use super::shell::hull;
use super::tests_cabin::outlines;
use super::tests_fit::{corners, inside};
use super::tub::tub;
use crate::ship::SEAT;
use crate::ship::cockpit::{HINGE, Screen};

/// Returns how far along `dir` the ray from `from` meets the triangle `tri`, and whether it
/// meets its front.
fn meet(tri: &[Vec3], from: Vec3, dir: Vec3) -> Option<(f32, bool)> {
    let (ab, ac) = (tri[1] - tri[0], tri[2] - tri[0]);
    let normal = dir.cross(ac);
    let scale = ab.dot(normal);
    if scale.abs() < 1e-9 {
        return None;
    }
    let to = from - tri[0];
    let (u, across) = (to.dot(normal) / scale, to.cross(ab));
    let (v, along) = (dir.dot(across) / scale, ac.dot(across) / scale);
    let hit = u >= 0.0 && v >= 0.0 && u + v <= 1.0 && along > 1e-4;
    hit.then(|| (along, ab.cross(ac).dot(dir) < 0.0))
}

/// Returns the first thing the ray from `from` along `dir` meets among the triangles `shell`.
fn first(shell: &[Vec3], from: Vec3, dir: Vec3) -> Option<(f32, bool)> {
    let hits = shell.chunks(3).filter_map(|tri| meet(tri, from, dir));
    hits.min_by(|a, b| a.0.total_cmp(&b.0))
}

/// Returns everything opaque the pilot can see of the ship from the seat: the hull, the spine,
/// the tub and what is bolted into it.
fn shell() -> Vec<Vec3> {
    let mut solids = vec![tub(), spine(), furniture(), plates(), accents()];
    solids.extend(hull());
    solids.into_iter().flat_map(corners).collect()
}

/// From the eye, every ray toward a point of the walls, the floor and the bulkheads of the tub
/// and toward the displays meets the front of something at that point or before it, every ray
/// that looks steeply down meets something, and no ray anywhere meets a surface from behind.
#[test]
fn the_cabin_has_no_hole_seen_from_the_eye() {
    let (eye, shell) = (SEAT.as_vec3(), shell());
    let mut targets: Vec<Vec3> = Screen::ALL.map(|s| s.placed().translation).into();
    for tri in corners(tub()).chunks(3) {
        let middle = (tri[0] + tri[1] + tri[2]) / 3.0;
        targets.push(middle);
        targets.extend(tri.iter().map(|corner| middle.lerp(*corner, 0.9)));
    }
    for target in targets {
        let reach = target.distance(eye);
        let seen = first(&shell, eye, (target - eye) / reach);
        let closed = seen.is_some_and(|(at, front)| front && at <= reach + 2e-3);
        assert!(closed, "a hole toward {target}: {seen:?}");
    }
    for (yaw, pitch) in (0..60).flat_map(|i| (-17..17).map(move |j| (i, j))) {
        let turn =
            Quat::from_rotation_y(yaw as f32 * 0.105) * Quat::from_rotation_x(pitch as f32 * 0.09);
        let dir = turn * Vec3::NEG_Z;
        let seen = first(&shell, eye, dir);
        assert!(
            seen.is_none_or(|(_, front)| front),
            "a back face along {dir}"
        );
        assert!(dir.y > -0.77 || seen.is_some(), "nothing along {dir}");
    }
}

/// The canopy holds to the ship shut and open: its hinge is the top edge of the hoop, which it
/// shares with the spine, so the two corners of that edge stay where they are at any angle.
/// Shut, the feet of the glass are under the skin of the hull and the hoop is against the spine.
#[test]
fn the_canopy_stays_attached_open_and_shut() {
    let hoop = arch(CANOPY[GLAZED - 1]);
    assert!(HINGE.distance((hoop[1] + hoop[2]) * 0.5) < 1e-5, "{HINGE}");
    let fixed = corners(spine());
    let moving: Vec<Vec3> = [glass(), frame()].into_iter().flat_map(corners).collect();
    for angle in [0.0, 0.5, 1.0, 1.4] {
        let swung = |p: &Vec3| HINGE + Quat::from_rotation_x(angle) * (*p - HINGE);
        let shared = moving
            .iter()
            .map(swung)
            .filter(|p| fixed.iter().any(|q| q.distance(*p) < 1e-4));
        assert!(shared.count() >= 2, "the canopy is loose at {angle} rad");
    }
    let hull = &outlines()[..2];
    let feet = corners(glass()).into_iter().filter(|p| p.y < 0.6);
    for foot in feet {
        assert!(
            hull.iter().any(|part| inside(part, foot)),
            "{foot} is in the air"
        );
    }
}
