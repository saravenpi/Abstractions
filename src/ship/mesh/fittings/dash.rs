//! The dashboard: the panel under the glass, the two hooded instrument pods, the raked panel
//! between them and the black faces the instruments stand on.

use bevy::prelude::*;

use super::pair;
use crate::ship::cockpit::{PANEL_Z, RAKE, RAKE_TILT};
use crate::ship::solid::Solid;

/// Inner and outer edge of the instrument pods across the hull, and their height: low enough
/// to leave the horizon free, and apart, so that the nose shows between them.
const POD: (f32, f32, f32) = (0.1, 0.3, 0.6);

/// Returns the point of the raked panel `across` metres from its middle and `along` metres up
/// its slope, lifted `lift` metres off its face.
fn rake(across: f32, along: f32, lift: f32) -> Vec3 {
    RAKE.0 + Quat::from_rotation_x(-RAKE_TILT) * Vec3::new(across, along, lift)
}

/// Returns the dashboard: the panel, the two pods with their hoods, and the raked wedge
/// between the pods, whose top is the plane the ball and the scope stand on.
pub fn dash() -> Solid {
    let mut dash = Solid::default();
    dash.slab(Vec3::new(-0.25, 0.3, -2.2), Vec3::new(0.25, 0.48, -1.99));
    pair(
        &mut dash,
        Vec3::new(POD.0, 0.36, -2.12),
        Vec3::new(POD.1, POD.2, PANEL_Z),
    );
    pair(
        &mut dash,
        Vec3::new(POD.0 - 0.01, POD.2, -2.12),
        Vec3::new(POD.1 + 0.01, POD.2 + 0.02, PANEL_Z + 0.05),
    );
    let top = [(-1.0, -1.0), (1.0, -1.0), (1.0, 1.0), (-1.0, 1.0)];
    let top = top.map(|(x, y)| rake(x * RAKE.1, y * RAKE.2, 0.0));
    dash.plate(top, Vec3::NEG_Y * 0.12);
    dash
}

/// Returns the black faces of the instruments: one on each pod and one on the raked panel.
pub fn faces() -> Solid {
    let mut faces = Solid::default();
    let z = PANEL_Z + 0.003;
    for side in [-1.0, 1.0] {
        let (near, far) = (side * (POD.0 + 0.02), side * (POD.1 - 0.005));
        let face = [
            Vec3::new(near, 0.465, z),
            Vec3::new(far, 0.465, z),
            Vec3::new(far, POD.2 - 0.012, z),
            Vec3::new(near, POD.2 - 0.012, z),
        ];
        faces.quad(face, Vec3::Z);
    }
    let (across, along) = (RAKE.1 - 0.005, RAKE.2 - 0.005);
    let panel = [(-1.0, -1.0), (1.0, -1.0), (1.0, 1.0), (-1.0, 1.0)];
    faces.quad(
        panel.map(|(x, y)| rake(x * across, y * along, 0.001)),
        Quat::from_rotation_x(-RAKE_TILT) * Vec3::Z,
    );
    faces
}
