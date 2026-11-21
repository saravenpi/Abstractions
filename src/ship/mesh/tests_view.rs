//! What the pilot sees of the cabin: the dashboard leaves the horizon and the nose in view,
//! and the cabin is the same on both sides.

use bevy::prelude::*;

use super::bars::frame;
use super::fittings::{accents, furniture, plates};
use super::shell::hull;
use super::tests_fit::{corners, crosses};
use super::tub::tub;
use crate::ship::SEAT;

/// Half of the default field of view, 75 degrees, as a slope.
const HALF_VIEW: f32 = 0.767_327;

/// Seen from the eyes of the pilot looking level, at the default field of view, nothing of
/// the hull or the cockpit ahead reaches into the upper two thirds of the screen, no bar of the
/// canopy frame crosses the view ahead, and the nose shows over the dashboard.
#[test]
fn the_dashboard_leaves_the_horizon_free() {
    let eye = SEAT.as_vec3();
    let mut solid = vec![tub(), furniture(), plates(), accents()];
    solid.extend(hull());
    let ahead: Vec<Vec3> = solid.into_iter().flat_map(corners).collect();
    for corner in ahead.iter().filter(|p| p.z < eye.z - 0.05) {
        let drop = (eye.y - corner.y) / (eye.z - corner.z);
        assert!(
            drop > HALF_VIEW / 3.0,
            "{corner} is {drop} under the horizon"
        );
    }
    let bars = corners(frame());
    let above = bars.iter().filter(|p| p.z < eye.z - 0.05 && p.y > eye.y);
    assert_eq!(above.count(), 0, "a bar crosses the view ahead");
    let cabin: Vec<Vec3> = [furniture(), plates(), accents()]
        .into_iter()
        .flat_map(corners)
        .collect();
    for nose in [Vec3::new(0.0, 0.25, -3.2), Vec3::new(0.0, -0.1, -4.2)] {
        let hidden = cabin.chunks(3).any(|tri| crosses(tri, eye, nose - eye));
        assert!(!hidden, "the dashboard hides the nose at {nose}");
        let drop = (eye.y - nose.y) / (eye.z - nose.z);
        assert!(drop < HALF_VIEW, "the nose at {nose} is under the screen");
    }
}

/// The interior is the same on both sides, but for the throttle on the left console.
#[test]
fn the_cabin_is_the_same_on_both_sides() {
    for part in [tub(), furniture(), plates(), accents(), frame()] {
        let points = corners(part);
        for point in &points {
            let mirror = Vec3::new(-point.x, point.y, point.z);
            let twin = points.iter().any(|other| other.distance(mirror) < 1.0e-4);
            assert!(twin, "{point} has no mirror image");
        }
    }
}
