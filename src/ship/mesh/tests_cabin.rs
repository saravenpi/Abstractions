//! Cabin tests: the hull that is drawn is its outline with a mouth, the glass closes on the
//! hull around that mouth, and everything in the cockpit fits inside.

use bevy::prelude::*;

use super::dome::{arch, canopy, glass};
use super::fittings::{accents, furniture, plates};
use super::frame::{CANOPY, FUSELAGE, GLAZED, fuselage};
use super::shell::{HALF, MOUTH, hull, lip, mouth, station};
use super::tests_fit::{corners, crosses, inside, positions};
use super::tub::tub;
use super::{cabin_meshes, lever_mesh, stick_mesh, tub_mesh};
use crate::ship::cockpit::{LEVER, Levers, STICK};

/// Returns the corners of the closed outlines: the two halves of the fuselage and the canopy.
pub(super) fn outlines() -> Vec<Vec<Vec3>> {
    let closed = fuselage().into_iter().chain([canopy()]);
    closed.map(corners).collect()
}

/// Returns true when `point` is inside one of the `outlines`.
fn within(outlines: &[Vec<Vec3>], point: Vec3) -> bool {
    outlines.iter().any(|outline| inside(outline, point))
}

/// Returns true when `point` is inside the `outlines` or on their skin: a centimetre further
/// in, or a centimetre down, it is inside.
fn held(outlines: &[Vec<Vec3>], point: Vec3) -> bool {
    let inward = (Vec3::new(0.0, 0.4, point.z) - point).normalize_or_zero();
    let near = [point, point + inward * 0.012, point - Vec3::Y * 0.012];
    near.into_iter().any(|at| within(outlines, at))
}

/// Returns the half width of the tub at `z`, between the widths of the mouth.
fn tub_half(z: f32) -> f32 {
    let stations = mouth();
    let i = usize::from(z > stations[1][0]);
    let t = (z - stations[i][0]) / (stations[i + 1][0] - stations[i][0]);
    HALF[i] + (HALF[i + 1] - HALF[i]) * t
}

/// Every corner of the drawn hull is on the skin of the closed outline, nothing closes the
/// joint between the nose and the body, and the roof is open over the seat. A step of 1 cm in from
/// a corner, straight toward the axis of the hull, is inside the outline, and a step out is not.
#[test]
fn the_drawn_hull_is_the_outline_with_a_mouth() {
    let outline = &outlines()[..2];
    let drawn: Vec<Vec3> = hull().into_iter().flat_map(corners).collect();
    let (first, last) = (FUSELAGE[0][0], FUSELAGE[FUSELAGE.len() - 1][0]);
    for corner in &drawn {
        let spoke = *corner - Vec3::new(0.0, station(corner.z.clamp(first, last))[3], corner.z);
        let way = if spoke.length() > 0.03 {
            -spoke.normalize()
        } else {
            Vec3::Z * -corner.z.signum()
        };
        let inward = way * 0.01;
        assert!(
            within(outline, *corner + inward),
            "{corner} is off the skin"
        );
        assert!(
            !within(outline, *corner - inward),
            "{corner} is under the skin"
        );
    }
    let across = |tri: &[Vec3]| tri.iter().all(|p| (p.z + 1.8).abs() < 1.0e-4);
    assert!(!drawn.chunks(3).any(across), "a wall closes the joint");
    for z in [MOUTH.0 + 0.05, -1.8, -1.3, MOUTH.1 - 0.05] {
        let from = Vec3::new(0.0, 0.2, z);
        let open = !drawn.chunks(3).any(|tri| crosses(tri, from, Vec3::Y));
        assert!(open, "the roof is closed over the cockpit at {z}");
    }
    let behind = Vec3::new(0.0, 0.2, MOUTH.1 + 0.1);
    assert!(drawn.chunks(3).any(|tri| crosses(tri, behind, Vec3::Y)));
}

/// The glass covers the whole mouth and its feet go under the skin of the hull outside the
/// walls of the tub: no gap between the two, and no pane inside the cockpit.
#[test]
fn the_glass_closes_on_the_hull_around_the_mouth() {
    let all = outlines();
    let (hull, bubble) = (&all[..2], &all[2..]);
    for (station, half) in mouth().into_iter().zip(HALF) {
        for at in [lip(station, half), lip(station, 0.0), lip(station, -half)] {
            assert!(within(bubble, at), "the lip {at} is outside the glass");
        }
    }
    for station in &CANOPY[..GLAZED] {
        let points = arch(*station);
        for foot in [points[0], points[3]] {
            assert!(
                within(hull, foot),
                "the foot {foot} of the glass is in the air"
            );
            assert!(
                foot.x.abs() > tub_half(foot.z) + 0.01,
                "{foot} is in the tub"
            );
        }
    }
    let panes = corners(glass());
    assert!(panes.iter().all(|p| held(&all, *p)));
    assert_eq!(panes.len() / 3, 15);
}

/// The tub, the frame of the furniture, its plates and its accents are inside the hull or
/// under the glass, and the whole cabin is under two thousand triangles, none of them
/// degenerate.
#[test]
fn the_cabin_fits_inside_the_hull() {
    let all = outlines();
    let parts = [tub(), furniture(), plates(), accents()];
    for (index, part) in parts.into_iter().enumerate() {
        for corner in corners(part) {
            assert!(held(&all, corner), "part {index}: {corner} sticks out");
        }
    }
    let fixed = cabin_meshes().map(|(_, mesh)| mesh);
    let meshes = [tub_mesh(), stick_mesh(), lever_mesh()];
    let mut triangles = 0;
    for mesh in fixed.iter().chain(&meshes) {
        let points = positions(mesh);
        triangles += points.len() / 3;
        for tri in points.chunks(3) {
            let area = (tri[1] - tri[0]).cross(tri[2] - tri[0]).length();
            assert!(area > 1.0e-7, "{tri:?}");
        }
    }
    assert!(triangles > 400 && triangles < 2000, "{triangles} triangles");
}

/// The stick and the lever stay inside the cockpit wherever the input puts them.
#[test]
fn the_moving_parts_stay_inside() {
    let all = outlines();
    let (stick, lever) = (positions(&stick_mesh()), positions(&lever_mesh()));
    for i in 0..27 {
        let axis = |n: usize| (i / n % 3) as f32 - 1.0;
        let held_at = Levers {
            stick: Vec3::new(axis(1), axis(3), axis(9)),
            throttle: axis(1) * 0.5 + 0.5,
            ..default()
        };
        for point in &stick {
            let at = STICK + held_at.stick_turn() * *point;
            assert!(within(&all, at), "the stick leaves the cockpit at {at}");
        }
        for point in &lever {
            let at = LEVER + held_at.lever_turn() * *point;
            assert!(within(&all, at), "the lever leaves the cockpit at {at}");
        }
    }
}
