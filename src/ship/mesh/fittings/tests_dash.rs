//! Dashboard tests: the displays and the lamps stand on the black faces of the dashboard, and
//! the controls of the right console and the pedals stand inside the cockpit.

use bevy::prelude::*;

use super::faces;
use crate::ship::cockpit::{GEAR_LEVER, Lamp, PEDAL, Screen, WARP_SWITCH};
use crate::ship::mesh::frame::{canopy, fuselage};
use crate::ship::mesh::tests_fit::{corners, crosses, inside};

/// Returns true when `point` is inside the hull or the glass.
fn within(point: Vec3) -> bool {
    let closed = fuselage().into_iter().chain([canopy()]);
    closed.map(corners).any(|outline| inside(&outline, point))
}

/// The displays and the lamps stand on the black faces of the dashboard, a hair above them and
/// parallel to them, so no instrument floats in the air or sinks into the console.
#[test]
fn the_displays_and_the_lamps_stand_on_their_faces() {
    let panel = corners(faces());
    let unit = [(-0.5, -0.5), (0.5, -0.5), (0.5, 0.5), (-0.5, 0.5)];
    let lamps = Lamp::ALL.map(|lamp| lamp.place(true));
    let screens = Screen::ALL.map(|screen| screen.placed());
    for placed in lamps.into_iter().chain(screens) {
        let normal = placed.rotation * Vec3::Z;
        for (x, y) in unit {
            let corner = placed * Vec3::new(x, y, 0.0);
            let on = panel
                .chunks(3)
                .any(|tri| crosses(tri, corner + normal * 0.002, -normal));
            assert!(on, "an instrument sticks out of its face at {corner}");
        }
    }
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
