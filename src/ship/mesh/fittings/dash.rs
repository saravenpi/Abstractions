//! The dashboard: one dark panel under the rim of the hull that faces the eye of the pilot, a
//! low hood over its top edge that runs forward to the foot of the windscreen, and a light bezel
//! round each of its three displays. Nothing rises into the view of the nose.

use bevy::prelude::*;

use crate::ship::cockpit::{BAYS, RAKE, RAKE_TILT};
use crate::ship::solid::Solid;

/// Thickness of the panel behind its face, how far a bezel stands off the face, and the width
/// of a bezel round its display, in metres.
const PANEL: (f32, f32, f32) = (0.04, 0.006, 0.014);

/// Two lengths in metres that go together: a rear and a front value.
type Pair = (f32, f32);

/// The hood: where its rear and its front edge are along z, their half widths, the height of
/// their top, and its thickness. It is a thin plate whose rear edge is behind the top edge of
/// the panel and over it, so that seen from the eye of the pilot it hides no part of a display.
const HOOD: (Pair, Pair, Pair, f32) = ((-2.155, -2.45), (0.33, 0.2), (0.535, 0.47), 0.05);

/// Returns the point of the raked panel `across` metres from its middle and `along` metres up
/// its slope, lifted `lift` metres off its face.
fn rake(across: f32, along: f32, lift: f32) -> Vec3 {
    RAKE.0 + Quat::from_rotation_x(-RAKE_TILT) * Vec3::new(across, along, lift)
}

/// Returns the corners of the part of the panel `half` metres either way of `centre`, lifted by
/// `lift`: bottom left, bottom right, top right and top left.
fn face(centre: Vec2, half: Vec2, lift: f32) -> [Vec3; 4] {
    [(-1.0, -1.0), (1.0, -1.0), (1.0, 1.0), (-1.0, 1.0)]
        .map(|(x, y)| rake(centre.x + x * half.x, centre.y + y * half.y, lift))
}

/// Returns where the display `index` of [`BAYS`] is on the panel: its centre across and up the
/// slope, and its half size, in metres.
fn bay(index: usize) -> (Vec2, Vec2) {
    let (across, width) = BAYS[index];
    (Vec2::new(across, 0.0), Vec2::new(width, BAYS[1].1) * 0.5)
}

/// Returns the dashboard: the panel and the hood over its top edge.
pub fn dash() -> Solid {
    let mut dash = Solid::default();
    let whole = face(Vec2::ZERO, Vec2::new(RAKE.1, RAKE.2), 0.0);
    dash.plate(
        whole,
        Quat::from_rotation_x(-RAKE_TILT) * Vec3::NEG_Z * PANEL.0,
    );
    let ((rear, front), (wide, narrow), (high, low), thick) = HOOD;
    let top = [
        Vec3::new(-wide, high, rear),
        Vec3::new(wide, high, rear),
        Vec3::new(narrow, low, front),
        Vec3::new(-narrow, low, front),
    ];
    dash.plate(top, Vec3::NEG_Y * thick);
    dash
}

/// Returns the bezels: round each display four light bars that stand off the face of the panel.
pub fn bezels() -> Solid {
    let mut bezels = Solid::default();
    let (_, lift, width) = PANEL;
    let out = Quat::from_rotation_x(-RAKE_TILT) * Vec3::Z * (lift + 0.004);
    for index in 0..BAYS.len() {
        let (centre, half) = bay(index);
        let (grown, bar) = (half + Vec2::splat(width), Vec2::splat(width * 0.5));
        for (dx, dy) in [(1.0, 0.0), (-1.0, 0.0), (0.0, 1.0), (0.0, -1.0)] {
            let at = centre + Vec2::new(dx * (half.x + bar.x), dy * (half.y + bar.y));
            let size = if dx != 0.0 {
                Vec2::new(bar.x, grown.y)
            } else {
                Vec2::new(grown.x, bar.y)
            };
            bezels.plate(face(at, size, -0.004), out);
        }
    }
    bezels
}
