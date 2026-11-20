//! The dashboard: the raked panel under the glass with the wedge that carries it, the hood
//! over its top edge and the black face the instruments stand on. Nothing rises above the
//! panel, so the world shows over it.

use bevy::prelude::*;

use crate::ship::cockpit::{RAKE, RAKE_TILT};
use crate::ship::solid::Solid;

/// How far the wedge under the panel goes down, and the height and depth of the hood over its
/// top edge, in metres.
const WEDGE: f32 = 0.14;

/// Returns the point of the raked panel `across` metres from its middle and `along` metres up
/// its slope, lifted `lift` metres off its face.
fn rake(across: f32, along: f32, lift: f32) -> Vec3 {
    RAKE.0 + Quat::from_rotation_x(-RAKE_TILT) * Vec3::new(across, along, lift)
}

/// Returns the corners of the face of the panel, shrunk by `inset` on every side and lifted by
/// `lift`: bottom left, bottom right, top right and top left.
fn face(inset: f32, lift: f32) -> [Vec3; 4] {
    let (across, along) = (RAKE.1 - inset, RAKE.2 - inset);
    [(-1.0, -1.0), (1.0, -1.0), (1.0, 1.0), (-1.0, 1.0)]
        .map(|(x, y)| rake(x * across, y * along, lift))
}

/// Returns the dashboard: the wedge that carries the panel, and the hood over its top edge.
pub fn dash() -> Solid {
    let mut dash = Solid::default();
    dash.plate(face(0.0, 0.0), Vec3::NEG_Y * WEDGE);
    let top = rake(0.0, RAKE.2, 0.0);
    dash.slab(
        Vec3::new(-RAKE.1, top.y, top.z - 0.07),
        Vec3::new(RAKE.1, top.y + 0.02, top.z + 0.01),
    );
    dash
}

/// Returns the black face of the instruments: one quad over the whole panel, which the
/// displays and the lamps stand on.
pub fn faces() -> Solid {
    let mut faces = Solid::default();
    faces.quad(
        face(0.003, 0.001),
        Quat::from_rotation_x(-RAKE_TILT) * Vec3::Z,
    );
    faces
}
