//! Dashboard tests: the displays stand on the panel of the dashboard, inside their bezels, and
//! the controls of the right console and the pedals stand inside the cockpit.

use bevy::prelude::*;

use super::dash::{bezels, dash};
use crate::ship::cockpit::{GEAR_LEVER, PEDAL, Screen, WARP_SWITCH};
use crate::ship::mesh::dome::canopy;
use crate::ship::mesh::frame::fuselage;
use crate::ship::mesh::tests_fit::{corners, crosses, inside};

/// Returns true when `point` is inside the hull or the glass.
fn within(point: Vec3) -> bool {
    let closed = fuselage().into_iter().chain([canopy()]);
    closed.map(corners).any(|outline| inside(&outline, point))
}

/// The displays stand on the panel of the dashboard, a hair above its face and parallel to it,
/// so no display floats in the air or sinks into the panel, and no bezel covers a display.
#[test]
fn the_displays_stand_on_the_panel() {
    let (panel, frames) = (corners(dash()), corners(bezels()));
    let unit = [(-0.5, -0.5), (0.5, -0.5), (0.5, 0.5), (-0.5, 0.5), (0.0, 0.0)];
    for placed in Screen::ALL.map(|screen| screen.placed()) {
        let normal = placed.rotation * Vec3::Z;
        for (x, y) in unit {
            let corner = placed * Vec3::new(x * 0.98, y * 0.98, 0.0);
            let from = corner + normal * 0.05;
            let on = panel.chunks(3).any(|tri| crosses(tri, from, -normal));
            assert!(on, "a display sticks out of the panel at {corner}");
            let covered = frames.chunks(3).any(|tri| crosses(tri, from, -normal));
            assert!(!covered, "a bezel covers a display at {corner}");
            assert!(!within_panel(&panel, corner), "a display is sunk at {corner}");
        }
    }
}

/// Returns true when `point` is inside the closed panel whose triangles have the corners `panel`.
fn within_panel(panel: &[Vec3], point: Vec3) -> bool {
    inside(&panel[..36], point)
}

/// The controls of the right console and the pedals stand inside the cockpit, on the console
/// and on the floor.
#[test]
fn the_other_controls_stand_in_the_cockpit() {
    for hinge in [
        GEAR_LEVER,
        WARP_SWITCH,
        PEDAL,
        Vec3::new(-PEDAL.x, PEDAL.y, PEDAL.z),
    ] {
        assert!(
            within(hinge + Vec3::Y * 0.03),
            "{hinge} is outside the hull"
        );
    }
}
